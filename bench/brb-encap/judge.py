"""Hidden invariant judge of BRB-ENCAP (ADR-0060 result 6, backlog T0-09).

The judge sees only a session's final data structure. It runs hidden
operation sequences (scripts) on it, starting from the type's `new()`, and
after every operation reads the operation's result and the structure's raw
fields. The agent never sees the scripts, the model or the invariant check.
Both arms are judged the same way:

- the result of every operation is compared with the task's Python model;
- the raw fields are checked against the task's invariant (`invariant`, the
  hidden invariant judge): a state that breaks it is a violation, whatever
  the results say;
- while the invariant holds, the abstract value of the raw fields (`view`)
  must equal the model's;
- a panic, trap, crash or hang fails the script.

A script's first failure is its kind (`violation`, `wrong`, `panic`, ..).
`violation_cases` counts the scripts where any recorded state breaks the
invariant, even after an earlier wrong result.

BMB arm:
1. Gate: `bmb verify` of the workspace. Only a proved workspace is
   delivered.
2. Invariant kept: the type's identity (`id`, which covers its fields and its
   invariant predicate's body, ADR-0059/0060) equals the seed's.
3. A hidden unit drives the scripts through the type's operations and the
   operation under test, and copies the raw fields out. It assumes nothing
   about the invariant. It must type-check against the workspace (else the
   target is missing or has another signature); proving it checks that the
   operation can be called on any state (a `requires` stronger than the
   statement fails this: `domain_ok`).
4. `bmb emit-llvm` of workspace + hidden unit, an LLVM entry and a C driver.
   Proved code has no runtime checks, so the run shows the states the proof
   allowed. An unproved workspace still runs (its unproved obligations are
   runtime checks; the invariant has none), so the judge can show what the
   gate rejected; it is not delivered either way.

Rust arm: the judge copies the agent's crate, appends a hidden child module
of the crate root that reads the private fields (`__brb_encap_probe`), and
builds a judge binary against it. A crate whose type no longer has the
seed's fields at the crate root does not build: not delivered.

Case files hold one script per line: the number of operations, then
`op a b` per operation. Drivers print `K ok TOKENS..` with `2 + raw` tokens
per finished operation (the result as two numbers, then the raw fields), and
a final `panic` token when the script stopped abnormally (Rust: a panic; BMB:
the forked child that runs it trapped, crashed or hung, see `c_driver`).

Standard library only.
"""
import importlib.util
import json
import os
import re
import shutil
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BRB = os.path.join(ROOT, "bench", "brb")


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# Frozen with BRB (bench/brb/MANIFEST): the seeded case generator, and
# `run_cases` and the judge crate's manifest. Loaded by path, under their own
# names, so that this module and BRB's (both `judge`) do not shadow each other.
brb_judge = _load("brb_judge", os.path.join(BRB, "judge.py"))
casegen = brb_judge.casegen

RET_KINDS = ("bool", "usize", "opt", "unit")
ARG_TYPES = ("i64", "usize")
FIELD_TYPES = ("usize", "[i64; 8]")
PROBE = "__brb_encap_probe"


# ---------- tasks ----------

def load_task(path):
    mod = _load("brb_encap_task_" + re.sub(r"\W", "_", os.path.basename(path)[:-3]), path)
    for _, ty in mod.RAW:
        assert ty in FIELD_TYPES, f"{mod.ID}: unsupported field type {ty}"
    names = [op[0] for op in mod.OPS]
    assert mod.NAME in names and len(set(names)) == len(names), f"{mod.ID}: operations"
    for _, args, ret, _, _ in mod.OPS:
        assert ret in RET_KINDS and len(args) <= 2 and all(a in ARG_TYPES for a in args), f"{mod.ID}: {args} {ret}"
    return mod


def load_tasks(tdir):
    return [load_task(os.path.join(tdir, f)) for f in sorted(os.listdir(tdir)) if f.endswith(".py")]


def seed_dir(task):
    return os.path.join(HERE, "seed", task.SEED)


def seed_files(task, arm):
    """{path in the workspace or crate: text} of the task's starting point."""
    d = seed_dir(task)

    def text(name):
        with open(os.path.join(d, name)) as f:
            return f.read()

    if arm == "bmb":
        return {f"src/{task.UNIT}.bmb": text(f"{task.UNIT}.bmb")}
    return {"src/lib.rs": text("lib.rs"), "tests/seed.rs": text("seed.rs")}


def write_files(root, files):
    for rel, content in files.items():
        path = os.path.join(root, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as f:
            f.write(content)


def task_prompt(task, arm):
    """The task message of one arm. The statement is the same text in both
    arms; where the type lives and the signature line differ."""
    if arm == "bmb":
        where = (f"The {task.KIND} is the type `{task.UNIT}::{task.BMB_TYPE}` in unit `{task.UNIT}`, with its "
                 f"existing operations; its invariant is the predicate named in its `#[invariant(..)]` attribute.\n\n"
                 f"Required signature (a function in unit `{task.UNIT}`):\n{task.SIG_BMB}")
    else:
        where = (f"The {task.KIND} is the type `{task.RUST_TYPE}` in `src/lib.rs`, with its existing methods; "
                 f"its invariant is stated in its documentation.\n\n"
                 f"Required signature (a method of `{task.RUST_TYPE}` in `src/lib.rs`):\n{task.SIG_RUST}")
    return f"Change the {task.KIND}. {task.STATEMENT}\n\n{where}"


def rust_ref_source(task, method):
    """The seed's `src/lib.rs` with `method` in a new `impl` block."""
    return seed_files(task, "rust")["src/lib.rs"] + f"\nimpl {task.RUST_TYPE} {{\n{method}}}\n"


# ---------- cases ----------

def hidden_cases(task):
    return list(task.cases(casegen.Gen(task.ID)))


def raw_size(task):
    return sum(8 if ty.startswith("[") else 1 for _, ty in task.RAW)


def width(task):
    return 2 + raw_size(task)


def encode(task, script):
    codes = {op[0]: i for i, op in enumerate(task.OPS)}
    out = [str(len(script))]
    for op, a, b in script:
        out += [str(codes[op]), str(a), str(b)]
    return " ".join(out)


def write_case_file(task, cs, path):
    with open(path, "w") as f:
        for script in cs:
            f.write(encode(task, script) + "\n")


def unpack(task, vals):
    raw, i = {}, 0
    for name, ty in task.RAW:
        if ty.startswith("["):
            raw[name] = vals[i:i + 8]
            i += 8
        else:
            raw[name] = vals[i]
            i += 1
    return raw


def score_case(task, script, got):
    """(kind of the first failure or "pass", index of the first state that
    breaks the invariant or None, detail)."""
    if got[0] != "ok":
        return got[0], None, {}
    toks = list(got[1])
    panicked = bool(toks) and toks[-1] == "panic"
    if panicked:
        toks.pop()
    try:
        vals = [int(t) for t in toks]
    except ValueError:
        return "wrong", None, {"got": " ".join(toks)[:120]}
    w = width(task)
    recs = [vals[i:i + w] for i in range(0, len(vals), w)]
    if recs and len(recs[-1]) != w:
        return "wrong", None, {"got": "record of the wrong size"}
    viol = next((t for t, r in enumerate(recs) if not task.invariant(unpack(task, r[2:]))), None)
    st = task.init()
    for t, (op, a, b) in enumerate(script):
        if t >= len(recs):
            return ("panic" if panicked else "crash"), viol, {"op": t}
        want = task.step(st, op, a, b)
        raw = unpack(task, recs[t][2:])
        if not task.invariant(raw):
            return "violation", viol, {"op": t, "call": f"{op}({a}, {b})", "raw": raw}
        got_view = task.view(raw)
        if tuple(recs[t][:2]) != tuple(want) or got_view != task.model_view(st):
            return "wrong", viol, {"op": t, "call": f"{op}({a}, {b})", "want": [list(want), task.model_view(st)],
                                   "got": [recs[t][:2], got_view]}
    if len(recs) != len(script):
        return "wrong", viol, {"got": "more records than operations"}
    return "pass", viol, {}


def score(task, cs, got):
    kinds = {}
    failing = []
    violation_cases = 0
    for k, script in enumerate(cs):
        kind, viol, detail = score_case(task, script, got.get(k, ("not run",)))
        kinds[kind] = kinds.get(kind, 0) + 1
        violation_cases += viol is not None
        if kind != "pass" and len(failing) < 3:
            failing.append({"case": k, "kind": kind, "ops": len(script), **json.loads(json.dumps(detail))})
    passed = kinds.pop("pass", 0)
    return {"cases": len(cs), "passed": passed, "failed": len(cs) - passed, "fail_kinds": kinds,
            "violation_cases": violation_cases, "failing": failing}


def not_delivered(cs, why, run=None):
    """Every case fails; `run` is the score of a run that happened anyway
    (BMB: an unproved workspace), kept for its `violation_cases`."""
    return {"delivered": False, "why": why, "cases": len(cs), "passed": 0, "failed": len(cs),
            "fail_kinds": {"not delivered": len(cs)},
            "violation_cases": run["violation_cases"] if run else None,
            "failing": run["failing"] if run else [], "run_fail_kinds": run["fail_kinds"] if run else None}


# ---------- BMB arm ----------

def bmb_rows(text):
    return [json.loads(line) for line in text.splitlines() if line.strip().startswith("{")]


def _bmb_arg(ty, v):
    return v if ty == "i64" else f"ix({v})"


def bmb_unit(task, hidden):
    """The hidden driver unit: `run(ops, out)` applies the script and copies
    each result and the raw fields to `out`."""
    w = width(task)
    ty = f"{task.UNIT}::{task.BMB_TYPE}"
    arms = []
    for code, (name, args, ret, call, _) in enumerate(task.OPS):
        call = call.format(name=task.NAME, **{v: _bmb_arg(t, v) for v, t in zip("ab", args)})
        if ret == "bool":
            body = f"r0 = flag({call});"
        elif ret == "usize":
            body = f"r0 = num({call});"
        elif ret == "opt":
            body = (f"match {call} {{\n                    None => {{}}\n"
                    f"                    Some(v{code}) => {{\n                        r0 = 1;\n"
                    f"                        r1 = v{code};\n                    }}\n                }}")
        else:
            body = f"{call};"
        head = "if" if code == 0 else "} else if"
        arms.append(f"            {head} op == {code} {{\n                {body}")
    copies, off = [], 2
    for i, (name, fty) in enumerate(task.RAW):
        if fty == "usize":
            copies.append(f"                out[w + {off}] = num(s.{name});")
            off += 1
        else:
            copies.append(f"                for k{i} in z..8 {{\n                    out[w + {off} + k{i}] = s.{name}[k{i}];\n"
                          "                }")
            off += 8
    return f"""#[summary("hidden judge")]
mod {hidden} {{
    #[effects(pure)]
    fn num(x: usize) -> i64 {{
        match i64::try_from(x).ok() {{
            None => -1,
            Some(v) => v,
        }}
    }}

    #[effects(pure)]
    fn ix(x: i64) -> usize {{
        match usize::try_from(x).ok() {{
            None => 0,
            Some(v) => v,
        }}
    }}

    #[effects(pure)]
    fn flag(b: bool) -> i64 {{
        if b {{
            1
        }} else {{
            0
        }}
    }}

    #[effects(io)]
    fn run(ops: &[i64], out: &mut [i64]) -> usize {{
        let mut s: {ty} = {ty}::new();
        let z: usize = 0;
        let mut w: usize = 0;
        let mut i: usize = 0;
        while i < ops.len() && ops.len() - i >= 3
            decreases ops.len() - i,
        {{
            let op: i64 = ops[i];
            let a: i64 = ops[i + 1];
            let b: i64 = ops[i + 2];
            let mut r0: i64 = 0;
            let mut r1: i64 = 0;
{chr(10).join(arms)}
            }}
            if w < out.len() && out.len() - w >= {w} {{
                out[w] = r0;
                out[w + 1] = r1;
{chr(10).join(copies)}
                w = w + {w};
            }}
            i = i + 3;
        }}
        w
    }}
}}
"""


def bmb_wrapper(hidden):
    return (f'\ndefine i64 @brb_entry(ptr %p, i64 %n, ptr %o, i64 %m) {{\nentry:\n'
            f'  %r = call i64 @"{hidden}::run"(ptr %p, i64 %n, ptr %o, i64 %m)\n  ret i64 %r\n}}\n')


def c_driver(task):
    """Runs each script in a forked child that replays its prefixes (1, 2, ..
    operations) and sends each new record through a pipe, so the records
    before a trap, crash or hang (`alarm`) still arrive; the parent then
    prints a final `panic` token. Proved code has no runtime checks, so a
    state the proof did not foresee can make it trap."""
    w = width(task)
    return (
        "#include <inttypes.h>\n#include <signal.h>\n#include <stdint.h>\n#include <stdio.h>\n"
        "#include <stdlib.h>\n#include <sys/wait.h>\n#include <unistd.h>\n"
        "extern int64_t brb_entry(int64_t *, int64_t, int64_t *, int64_t);\n"
        "int main(int argc, char **argv) {\n"
        "  FILE *f = fopen(argv[1], \"r\");\n  long start = atol(argv[2]);\n"
        "  char *line = NULL; size_t cap = 0; long k = 0;\n"
        "  while (getline(&line, &cap, f) > 0) {\n"
        "    if (k < start) { k++; continue; }\n"
        "    char *p = line, *e;\n"
        "    int64_t n = strtoll(p, &e, 10); p = e;\n"
        "    int64_t *ops = malloc(8 * (size_t)(3 * n + 1));\n"
        "    for (int64_t j = 0; j < 3 * n; j++) { ops[j] = strtoll(p, &e, 10); p = e; }\n"
        f"    int64_t m = {w} * n;\n"
        "    int64_t *out = malloc(8 * (size_t)(m + 1));\n"
        "    int fd[2];\n    if (pipe(fd) != 0) return 1;\n"
        "    fflush(stdout);\n"
        "    pid_t pid = fork();\n"
        "    if (pid == 0) {\n"
        "      close(fd[0]);\n      alarm(5);\n"
        "      for (int64_t j = 1; j <= n; j++) {\n"
        f"        int64_t r = brb_entry(ops, 3 * j, out, {w} * j);\n"
        f"        if (r != {w} * j) _exit(3);\n"
        f"        if (write(fd[1], out + {w} * (j - 1), 8 * {w}) != 8 * {w}) _exit(4);\n"
        "      }\n"
        "      _exit(0);\n"
        "    }\n"
        "    close(fd[1]);\n"
        "    int64_t got = 0; ssize_t len;\n"
        "    while (got < m && (len = read(fd[0], (char *)out + 8 * got, 8 * (size_t)(m - got))) > 0) got += len / 8;\n"
        "    close(fd[0]);\n"
        "    int status = 0;\n    waitpid(pid, &status, 0);\n"
        "    printf(\"%ld ok\", k);\n"
        "    for (int64_t j = 0; j < got; j++) printf(\" %\" PRId64, out[j]);\n"
        "    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) printf(\" panic\");\n"
        "    printf(\"\\n\");\n    fflush(stdout);\n"
        "    free(ops); free(out);\n"
        "    k++;\n  }\n  return 0;\n}\n"
    )


_seed_ids = {}


def type_id(bmb, cargs, ws, task):
    """The `id` of the task's type in workspace `ws`, or None."""
    _, out = bmb("query", *cargs, "surface", f"{task.UNIT}::{task.BMB_TYPE}", "--", ws)
    row = next((r for r in bmb_rows(out) if r.get("level") == "L1"), None)
    return row.get("id") if row else None


def seed_type_id(bmb, task, tmp):
    if task.ID not in _seed_ids:
        ws = os.path.join(tmp, "seed-ws")
        write_files(ws, seed_files(task, "bmb"))
        _seed_ids[task.ID] = type_id(bmb, [], ws, task)
        shutil.rmtree(ws, ignore_errors=True)
    return _seed_ids[task.ID]


def invariant_diagnostics(verify_out):
    """Error codes of a `verify` output, and whether one is at a type
    invariant boundary (E0823, or E0811 when the solver could not decide)."""
    rows = [r for r in bmb_rows(verify_out) if r.get("severity") == "error"]
    codes = sorted({r["code"] for r in rows})
    return codes, any("type invariant of" in r.get("message", "") for r in rows)


def judge_bmb(task, ws, bmb_bin, tmp, cache=None):
    """Judges the BMB workspace `ws` (see the module doc)."""
    cs = hidden_cases(task)
    cargs = ["--cache", cache] if cache else []

    def bmb(*a):
        r = subprocess.run([bmb_bin, *a], capture_output=True, text=True)
        return r.returncode, r.stdout

    vcode, vout = bmb("verify", *cargs, ws)
    codes, at_invariant = invariant_diagnostics(vout)
    head = {"verified": vcode == 0, "gate_codes": codes, "invariant_rejected": at_invariant,
            "invariant_kept": type_id(bmb, cargs, ws, task) == seed_type_id(bmb, task, tmp)}
    _, l0 = bmb("map", *cargs, "L0", ws)
    units = {r["path"] for r in bmb_rows(l0) if r.get("kind") == "unit"}
    hidden = "brb_encap_judge"
    while hidden in units:
        hidden += "_"
    jpath = os.path.join(tmp, "judge.bmb")
    with open(jpath, "w") as f:
        f.write(bmb_unit(task, hidden))
    ccode, _ = bmb("check", ws, jpath)
    if ccode != 0:
        return {**not_delivered(cs, "target missing or wrong signature"), **head, "domain_ok": None}
    dcode, _ = bmb("verify", *cargs, ws, jpath)
    head["domain_ok"] = (dcode == 0) if vcode == 0 else None
    ll = os.path.join(tmp, "judge.ll")
    bmb("emit-llvm", *cargs, ws, jpath, "-o", ll)
    if not os.path.exists(ll):
        return {**not_delivered(cs, "no LLVM IR"), **head}
    with open(ll) as f:
        ir = f.read()
    ir = re.sub(r"define i32 @main\(\)", "define i32 @brb_agent_main()", ir)
    with open(ll, "w") as f:
        f.write(ir + bmb_wrapper(hidden))
    cpath = os.path.join(tmp, "driver.c")
    with open(cpath, "w") as f:
        f.write(c_driver(task))
    exe = os.path.join(tmp, "judge-bmb")
    b = subprocess.run(["clang", "-O1", "-w", "-Wno-override-module", "-o", exe, cpath, ll],
                       capture_output=True, text=True)
    if b.returncode != 0:
        return {**not_delivered(cs, "driver build: " + b.stderr[-400:]), **head}
    cfile = os.path.join(tmp, "cases.txt")
    write_case_file(task, cs, cfile)
    run = score(task, cs, brb_judge.run_cases(lambda s: [exe, cfile, str(s)], len(cs)))
    if vcode != 0:
        return {**not_delivered(cs, "not verified", run), **head}
    return {"delivered": True, **head, **run}


# ---------- Rust arm ----------

def probe_module(task):
    lines = []
    for name, ty in task.RAW:
        if ty == "usize":
            lines.append(f"        let {name}: usize = s.{name};\n"
                         f"        v.push(i64::try_from({name}).unwrap_or(-1));")
        else:
            lines.append(f"        let {name}: &[i64; 8] = &s.{name};\n        v.extend_from_slice({name});")
    return ("\n#[doc(hidden)]\n#[allow(warnings, clippy::all)]\n"
            f"pub mod {PROBE} {{\n    pub fn raw(s: &super::{task.RUST_TYPE}) -> Vec<i64> {{\n"
            "        let mut v = Vec::new();\n" + "\n".join(lines) + "\n        v\n    }\n}\n")


def _rust_arg(ty, v):
    return v if ty == "i64" else f"({v} as usize)"


def rust_main(task, callees):
    """Judge `main.rs`: the operation under test is `callees[which]`."""
    arms = []
    for code, (name, args, ret, _, call) in enumerate(task.OPS):
        argmap = {v: _rust_arg(t, v) for v, t in zip("ab", args)}
        variants = [call.format(name=c, **argmap) for c in callees] if name == task.NAME \
            else [call.format(name=task.NAME, **argmap)]

        def conv(c):
            if ret == "bool":
                return f"(flag({c}), 0)"
            if ret == "usize":
                return f"(num({c}), 0)"
            if ret == "opt":
                return f"match {c} {{ None => (0, 0), Some(v) => (1, v) }}"
            return f"{{ {c}; (0, 0) }}"

        if len(variants) == 1:
            arms.append(f"            {code} => {conv(variants[0])},")
        else:
            inner = "\n".join(f"                {j} => {conv(v)}," for j, v in enumerate(variants))
            arms.append(f"            {code} => match which {{\n{inner}\n                _ => unreachable!(),\n            }},")
    return (
        "#![allow(warnings, clippy::all)]\n"
        "use std::io::Write;\n\n"
        "fn flag(b: bool) -> i64 {\n    b as i64\n}\n\n"
        "fn num(x: usize) -> i64 {\n    i64::try_from(x).unwrap_or(-1)\n}\n\n"
        "fn run(ops: &[i64], which: usize, rec: &mut Vec<i64>) {\n"
        f"    let mut s = solution::{task.RUST_TYPE}::new();\n"
        "    for c in ops.chunks(3) {\n"
        "        let (op, a, b) = (c[0], c[1], c[2]);\n"
        "        let (r0, r1): (i64, i64) = match op {\n"
        + "\n".join(arms) + "\n"
        "            _ => unreachable!(),\n"
        "        };\n"
        "        rec.push(r0);\n        rec.push(r1);\n"
        f"        rec.extend(solution::{PROBE}::raw(&s));\n"
        "    }\n}\n\n"
        "fn main() {\n"
        "    std::panic::set_hook(Box::new(|_| {}));\n"
        "    let args: Vec<String> = std::env::args().collect();\n"
        "    let text = std::fs::read_to_string(&args[1]).unwrap();\n"
        "    let start: usize = args[2].parse().unwrap();\n"
        "    let which: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(0);\n"
        "    let out = std::io::stdout();\n"
        "    for (k, line) in text.lines().enumerate().skip(start) {\n"
        "        let nums: Vec<i64> = line.split_whitespace().map(|t| t.parse().unwrap()).collect();\n"
        "        let ops = &nums[1..];\n"
        "        let mut rec = Vec::new();\n"
        "        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(ops, which, &mut rec)));\n"
        "        let mut o = out.lock();\n"
        "        write!(o, \"{} ok\", k).unwrap();\n"
        "        for x in &rec {\n            write!(o, \" {}\", x).unwrap();\n        }\n"
        "        if res.is_err() {\n            write!(o, \" panic\").unwrap();\n        }\n"
        "        writeln!(o).unwrap();\n"
        "        o.flush().unwrap();\n"
        "    }\n}\n"
    )


def probed_crate(task, crate, dest, lib=None):
    """A copy of `crate` (manifest and `src/`) whose `src/lib.rs` (or `lib`)
    ends with the hidden probe module."""
    os.makedirs(dest)
    shutil.copy(os.path.join(crate, "Cargo.toml"), dest)
    if lib is None:
        shutil.copytree(os.path.join(crate, "src"), os.path.join(dest, "src"))
        with open(os.path.join(dest, "src", "lib.rs")) as f:
            lib = f.read()
    write_files(dest, {"src/lib.rs": lib + probe_module(task)})


def build_rust_judge(task, crate, tmp, callees):
    """Builds the judge binary against `crate` (see brb_judge.build_rust_judge:
    same manifest and profile, own target directory). Returns (exe, error)."""
    jdir = os.path.join(tmp, "judge")
    target_dir = os.path.join(jdir, "target")
    os.makedirs(os.path.join(jdir, "src"), exist_ok=True)
    with open(os.path.join(jdir, "Cargo.toml"), "w") as f:
        f.write(brb_judge.JUDGE_TOML.format(path=os.path.abspath(crate)))
    with open(os.path.join(jdir, "src", "main.rs"), "w") as f:
        f.write(rust_main(task, callees))
    env = {**os.environ, "CARGO_TARGET_DIR": target_dir, "CARGO_TERM_COLOR": "never"}
    b = subprocess.run(["cargo", "build", "--offline", "--quiet", "--manifest-path",
                        os.path.join(jdir, "Cargo.toml")], capture_output=True, text=True, env=env)
    if b.returncode != 0:
        return None, b.stderr[-600:]
    exe = os.path.join(tmp, "judge-rust")
    shutil.copy(os.path.join(target_dir, "debug", "brb_judge"), exe)
    return exe, None


def judge_rust(task, crate, tmp):
    cs = hidden_cases(task)
    probed = os.path.join(tmp, "agent")
    probed_crate(task, crate, probed)
    exe, err = build_rust_judge(task, probed, tmp, [task.NAME])
    if exe is None:
        return not_delivered(cs, "judge build (missing method, or the type's fields changed): " + err)
    cfile = os.path.join(tmp, "cases.txt")
    write_case_file(task, cs, cfile)
    run = score(task, cs, brb_judge.run_cases(lambda s: [exe, cfile, str(s)], len(cs)))
    return {"delivered": True, **run}


def judge_mutants(task, tmp):
    """Each Rust mutant's hidden-case result: the scripts must catch every
    mutant (judge adequacy, run in CI)."""
    cs = hidden_cases(task)
    names = [f"{task.NAME}_m{i}" for i in range(len(task.RUST_MUTANTS))]
    lib = seed_files(task, "rust")["src/lib.rs"]
    for name, m in zip(names, task.RUST_MUTANTS):
        renamed = m.replace(f"fn {task.NAME}(", f"fn {name}(", 1)
        lib += f"\n#[allow(warnings, clippy::all)]\nimpl {task.RUST_TYPE} {{\n{renamed}}}\n"
    crate = os.path.join(tmp, "mutants")
    os.makedirs(crate)
    with open(os.path.join(crate, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "solution"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n\n[workspace]\n')
    probed = os.path.join(tmp, "probed")
    probed_crate(task, crate, probed, lib)
    exe, err = build_rust_judge(task, probed, tmp, names)
    if exe is None:
        return [{"mutant": i, "killed": None, "error": err} for i in range(len(names))]
    cfile = os.path.join(tmp, "cases.txt")
    write_case_file(task, cs, cfile)
    out = []
    for i in range(len(names)):
        s = score(task, cs, brb_judge.run_cases(lambda st, i=i: [exe, cfile, str(st), str(i)], len(cs)))
        out.append({"mutant": i, "killed": s["failed"] > 0, "failed": s["failed"], "fail_kinds": s["fail_kinds"],
                    "violation_cases": s["violation_cases"]})
    return out
