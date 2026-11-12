"""The body-changing changes of H-REVIEW's set: one-site body mutants of the
BVB programs, made and judged by the compiler (`bmb mutate`, `bmb verify`, the
real back end). Standard library and `bmb` ($BMB), `clang`, `stdbuf`; used by
`gen.py`, which keeps the committed set as the frozen artifact.

For each BVB task, in the order `bmb mutate` prints its rows (every W0904
change to each executable function of the task's unit, functions in source
order), the first K = `K` mutants that qualify, per kind:

  body-clean    the mutant of the `reference` program still passes `bmb verify`
                under the unchanged reference contract (an equivalent or
                contract-allowed rewrite). Label `clean`.
  body-defect   the same site and replacement in the `weak_reference` program
                passes `bmb verify` under the weak contract, and there is a
                witness: an input on which the hidden precondition is proved,
                the mutant terminates, its output differs from the reference
                program's, and `bmb verify` proves that the hidden reference
                contract rejects that output (the kill check of
                `bench/m1/bsb.py`, against a stub with the hidden contract).
                The change is reference -> weak contract + mutant. Label
                `defect`.

The mutant's text is the task-order program with the row's replacement on the
row's line; `bmb fmt` of it must equal the mutant program the compiler printed
(so nothing here decides what a mutant is), and the recorded column is the
position that gave that text. The compiler-free check (`gen.py --check`)
re-applies the recorded replacement.

Inputs are not written per task. `inputs` draws `N_INPUTS` values per task
from the parameter types with a generator seeded by the task id (small
integers, arrays of length 0..5, sometimes sorted, sometimes of equal
length), and only those whose hidden precondition `bmb verify` proves are
used. A mutant with a difference on no such input is not a `body-defect`
change (it may still differ elsewhere: the set takes only decided cases).
"""
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.path.append(os.path.join(BENCH, "m1"))
import bsb  # noqa: E402
import bsb_gen  # noqa: E402

K = 2
N_INPUTS = 60
MAX_CANDIDATES = 6
RUN_TIMEOUT = 10
INTS = {"i64": [-3, -2, -1, 0, 1, 2, 3, 4, 5, 7, 10, 12, 30, 100],
        "u64": [0, 1, 2, 3, 4, 5, 7, 10, 12, 30, 100],
        "usize": [0, 1, 2, 3, 4, 5, 7, 10, 12, 30, 100]}
ELEMS = {"i64": [-3, -2, -1, 0, 1, 2, 3, 4, 5],
         "u64": [0, 1, 2, 3, 5, 9, 48, 50, 57, 65, 90, 97, 122],
         "bool": [False, True]}
ARRAY = re.compile(r"&(mut )?\[(i64|u64|bool)\]$")


def run(bmb, *args, cwd=None):
    r = subprocess.run([bmb, *args], capture_output=True, text=True, cwd=cwd)
    return r.returncode, r.stdout


# ---------- inputs ----------

def inputs(task):
    """Deterministic candidate inputs of a task, from its signature's types."""
    params, _ = bsb.params_of(task["signature"])
    r = random.Random(f"review:{task['id']}")
    out, seen = [], set()
    for _ in range(N_INPUTS):
        same_len = r.random() < 0.5
        length = r.randint(0, 5)
        inp = {}
        for name, ty in params:
            m = ARRAY.match(ty)
            if m:
                n = length if same_len else r.randint(0, 5)
                vals = [r.choice(ELEMS[m.group(2)]) for _ in range(n)]
                if m.group(2) != "bool" and r.random() < 1 / 3:
                    vals.sort()
                inp[name] = vals
            elif ty == "bool":
                inp[name] = r.choice([False, True])
            elif ty in INTS:
                inp[name] = r.choice(INTS[ty])
            else:
                raise SystemExit(f"{task['id']}: parameter type {ty} has no input generator")
        key = json.dumps(inp, sort_keys=True)
        if key not in seen:
            seen.add(key)
            out.append(inp)
    return out


def call_lines(target, signature, inp, result, suffix):
    """Statements binding `inp` and calling `target` (as `bsb.call_lines`, for
    every element type BVB uses)."""
    params, ret = bsb.params_of(signature)
    lines, args = [], []
    for name, ty in params:
        m = ARRAY.match(ty)
        if m:
            mut = "mut " if m.group(1) else ""
            vals = inp[name]
            lines.append(f"let {mut}{name}{suffix}: [{m.group(2)}; {len(vals)}] = "
                         f"[{', '.join(bsb.lit(x) for x in vals)}];")
            args.append(("&mut " if mut else "&") + name + suffix)
        else:
            args.append(bsb.lit(inp[name]))
    call = f"{target}({', '.join(args)})"
    lines.append(call + ";" if ret == "()" else f"let {result}: {ret} = {call};")
    return lines


def out_arrays(signature):
    """[(name, element type)] of the `&mut` array parameters."""
    params, _ = bsb.params_of(signature)
    return [(n, ARRAY.match(ty).group(2)) for n, ty in params if ARRAY.match(ty) and ARRAY.match(ty).group(1)]


# ---------- the hidden contract ----------

def hidden_unit(task, seed_text, fns):
    """A unit with the hidden reference contract on a stub and `fns`, every
    one of which sees the hidden predicates and the seed's spec fns."""
    names = re.findall(r"spec fn (\w+)", task["accept"]["specs"])
    if not task["accept"]["pre"]:
        names = [n for n in names if n != "ref_pre"]
    uses = ", ".join(names + ["given::" + n for n in re.findall(r"spec fn (\w+)", seed_text)])
    body = "\n".join(f"#[effects(pure)]\n#[uses({uses})]\nfn {name}() -> () {{\n"
                     + "".join(f"    {l}\n" for l in lines) + "}\n" for name, lines in fns)
    return '#[summary("hidden check")]\nmod bmb_review {\n' + bsb_gen.stub_prelude(task) + body + "}\n"


def proved(bmb, task, seed, fns, tmp):
    """{fn name: verified} for `fns` checked against the stub."""
    with open(seed) as f:
        seed_text = f.read()
    path = os.path.join(tmp, "hidden.bmb")
    with open(path, "w") as f:
        f.write(hidden_unit(task, seed_text, fns))
    _, out = run(bmb, "verify", seed, path)
    return bsb.statuses(out, "bmb_review")


def allowed_inputs(bmb, task, seed, tmp):
    """The candidate inputs whose hidden precondition is proved: a call of
    the stub on the input verifies."""
    cand = inputs(task)
    fns = [(f"p{i}", call_lines("stub", task["signature"], inp, "r", "")) for i, inp in enumerate(cand)]
    st = proved(bmb, task, seed, fns, tmp)
    return [inp for i, inp in enumerate(cand) if st.get(f"p{i}")]


def rejected(bmb, task, seed, witnesses, tmp):
    """For each witness: is its output proved impossible under the hidden contract?"""
    _, ret = bsb.params_of(task["signature"])
    fns = []
    for i, w in enumerate(witnesses):
        lines = call_lines("stub", task["signature"], w["input"], "r", "")
        terms = []
        if ret != "()":
            v = w["output"]["r"]
            terms.append(("r" if v else "!r") if ret == "bool" else f"r == {bsb.lit(v)}")
        for name, _ in out_arrays(task["signature"]):
            terms += [f"{name}[{k}] == {bsb.lit(x)}" for k, x in enumerate(w["output"][name])]
        fns.append((f"k{i}", lines + [f"assert(!({' && '.join(terms)}));"]))
    st = proved(bmb, task, seed, fns, tmp)
    return [bool(st.get(f"k{i}")) for i in range(len(witnesses))]


# ---------- running ----------

def driver(target, signature, ins):
    _, ret = bsb.params_of(signature)
    show = {"bool": "show_bool", "i64": "print_i64", "u64": "show_u64", "usize": "show_usize"}
    body = []
    for i, inp in enumerate(ins):
        body += call_lines(target, signature, inp, f"r_{i}", f"_{i}")
        if ret != "()":
            body.append(f"{show[ret]}(r_{i});")
        for name, elem in out_arrays(signature):
            body += [f"{show[elem]}({name}_{i}[{k}]);" for k in range(len(inp[name]))]
    return ('#[summary("review driver")]\nmod review_driver {\n' + bsb_gen.SHOW
            + "\n    #[effects(io)]\n    fn main() -> () {\n" + "".join(f"        {l}\n" for l in body) + "    }\n}\n")


def group(signature, inp, nums):
    """(numbers used, output) of one input's printed results."""
    _, ret = bsb.params_of(signature)
    out, k = {}, 0
    if ret != "()":
        if not nums:
            return 0, None
        out["r"] = bool(nums[0]) if ret == "bool" else nums[0]
        k = 1
    for name, _ in out_arrays(signature):
        n = len(inp[name])
        if k + n > len(nums):
            return 0, None
        out[name] = nums[k:k + n]
        k += n
    return k, out


def run_program(bmb, files, target, signature, ins, tmp):
    """Per input {"ok": output} | {"trap": true} | {"hang": true}, running
    `files` through `emit-llvm --all-checks` and clang (no contract is
    trusted); None when the program does not pass the front end."""
    d = tempfile.mkdtemp(dir=tmp)
    results, start = [], 0
    while start < len(ins):
        rest = ins[start:]
        drv, ll, exe = (os.path.join(d, n) for n in ("review_driver.bmb", "p.ll", "p"))
        with open(drv, "w") as f:
            f.write(driver(target, signature, rest))
        code, _ = run(bmb, "emit-llvm", "--all-checks", *files, drv, "-o", ll)
        if code != 0:
            return None
        c = subprocess.run(["clang", "-O0", "-Wno-override-module", "-o", exe, ll], capture_output=True, text=True)
        if c.returncode != 0:
            raise SystemExit(f"clang failed: {c.stderr}")
        try:
            p = subprocess.run(["stdbuf", "-oL", exe], capture_output=True, text=True, timeout=RUN_TIMEOUT)
            rc, stdout, hung = p.returncode, p.stdout, False
        except subprocess.TimeoutExpired as e:
            rc, hung = None, True
            stdout = (e.stdout or b"").decode() if isinstance(e.stdout, bytes) else (e.stdout or "")
        nums = [int(x) for x in stdout.split()]
        k = done = 0
        for inp in rest:
            # An input with nothing to print cannot be told from one not reached:
            # it counts as done only when the program ended normally.
            n, out = group(signature, inp, nums[k:])
            if out is None or (n == 0 and rc != 0):
                break
            results.append({"ok": out})
            k += n
            done += 1
        if done == len(rest):
            break
        if rc == 0:
            raise SystemExit(f"driver output is short: {stdout!r}")
        results.append({"hang": True} if hung else {"trap": True})
        start = len(results)
    return results


# ---------- mutants ----------

def apply_mutation(text, m):
    """`text` with the one-site replacement `m` (line and column from 1), or
    None when the site does not hold `m["before"]` on that line."""
    lines = text.split("\n")
    i, j = m["line"] - 1, m["col"] - 1
    if i >= len(lines) or not lines[i][j:].startswith(m["before"]):
        return None
    lines[i] = lines[i][:j] + m["after"] + lines[i][j + len(m["before"]):]
    return "\n".join(lines)


def same_unit(ws_fmt, seed_fmt, unit_text):
    """Is the workspace's normal form the seed's followed by `unit_text`?"""
    return ws_fmt.endswith(unit_text) and ws_fmt[:-len(unit_text)].rstrip("\n") == seed_fmt.rstrip("\n")


def exec_defs(text, unit):
    """The executable functions of the unit, in source order."""
    return [f"{unit}::{n}" for n in re.findall(r"^    (?:proof )?fn (\w+)", text, re.M)]


def mutant_rows(bmb, seed, file, text, tmp):
    """`bmb mutate` rows of every executable function of the program, each
    with `program`: the task-order text with the row's replacement, checked
    against the compiler's own mutant text (None when it cannot be placed)."""
    ws = os.path.join(tmp, "ws")
    if os.path.isdir(ws):
        shutil.rmtree(ws)
    os.makedirs(os.path.join(ws, "src"))
    shutil.copy(seed, os.path.join(ws, "src", os.path.basename(seed)))
    path = os.path.join(ws, file)
    with open(path, "w") as f:
        f.write(text)
    unit = os.path.basename(file)[:-4]
    args = [a for d in exec_defs(text, unit) for a in ("--def", d)]
    code, out = run(bmb, "mutate", *args, ws)
    if code != 0:
        raise SystemExit(f"bmb mutate failed: {out}")
    seed_fmt = run(bmb, "fmt", seed)[1]
    rows = []
    for line in out.splitlines():
        r = json.loads(line)
        if r.get("summary"):
            continue
        # The row's column is where the changed construct starts; the replaced
        # expression may start later on the line. The first position on the
        # row's line whose replacement formats to the compiler's mutant is it.
        m = {k: r[k] for k in ("def", "site", "line", "col", "before", "after")}
        prog, src = None, text.split("\n")
        line = src[r["line"] - 1] if r["line"] <= len(src) else ""
        at = [j for j in range(len(line)) if line.startswith(r["before"], j)]
        for j in [j for j in at if j >= r["col"] - 1] + [j for j in at if j < r["col"] - 1]:
            cand = apply_mutation(text, dict(m, col=j + 1))
            with open(path, "w") as f:
                f.write(cand)
            if same_unit(run(bmb, "fmt", ws)[1], seed_fmt, r["text"]):
                prog, m["col"] = cand, j + 1
                break
        rows.append({"mutation": m, "program": prog})
    with open(path, "w") as f:
        f.write(text)
    return ws, path, rows


def verifies(bmb, ws, path, row):
    """Does the mutant pass the proof? Only the mutated function is proved
    again (`--def`; the whole program is still checked): the other
    definitions are the proved program's and see this one by its contract."""
    with open(path, "w") as f:
        f.write(row["program"])
    return run(bmb, "verify", "--def", row["mutation"]["def"], ws)[0] == 0


def for_task(bmb, task, seed, file, reference, weak):
    """(body-clean, body-defect, counts) of one task: lists of
    {"mutation", "program"[, "witness"]}, at most K each."""
    counts = {"mutants": 0, "unplaced": 0, "clean_proved": 0, "weak_proved": 0, "inputs": 0, "defect_witnessed": 0}
    tmp = tempfile.mkdtemp(prefix="bmb-review-mut-")
    try:
        ws, path, rows = mutant_rows(bmb, seed, file, reference, tmp)
        counts["mutants"] = len(rows)
        counts["unplaced"] = sum(1 for r in rows if r["program"] is None)
        clean = []
        for r in rows:
            if len(clean) == K:
                break
            if r["program"] is not None and verifies(bmb, ws, path, r):
                clean.append(r)
        counts["clean_proved"] = len(clean)

        # The same rows on the weak program: its bodies are the reference's, so
        # a site may sit on another line; the compiler says where.
        ws, path, rows = mutant_rows(bmb, seed, file, weak, tmp)
        defect, allowed, ref_out = [], None, None
        target = task["def"]
        for r in rows:
            if len(defect) == K:
                break
            if r["program"] is None or not verifies(bmb, ws, path, r):
                continue
            counts["weak_proved"] += 1
            if allowed is None:
                allowed = allowed_inputs(bmb, task, seed, tmp)
                with open(path, "w") as f:
                    f.write(reference)
                ref_out = run_program(bmb, [seed, path], target, task["signature"], allowed, tmp)
                keep = [i for i, x in enumerate(ref_out or []) if "ok" in x]
                allowed, ref_out = [allowed[i] for i in keep], [ref_out[i] for i in keep]
                counts["inputs"] = len(allowed)
            if not allowed:
                continue
            with open(path, "w") as f:
                f.write(r["program"])
            got = run_program(bmb, [seed, path], target, task["signature"], allowed, tmp)
            if got is None:
                continue
            cand = [{"input": allowed[i], "expected": ref_out[i]["ok"], "output": got[i]["ok"]}
                    for i in range(len(allowed)) if "ok" in got[i] and got[i] != ref_out[i]][:MAX_CANDIDATES]
            verdicts = rejected(bmb, task, seed, cand, tmp) if cand else []
            witness = next((w for w, v in zip(cand, verdicts) if v), None)
            if witness:
                defect.append(dict(r, witness=witness))
        counts["defect_witnessed"] = len(defect)
        return clean, defect, counts
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
