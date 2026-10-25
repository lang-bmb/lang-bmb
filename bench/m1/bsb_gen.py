#!/usr/bin/env python3
"""Generates the hidden BSB mutant sets of a task set (backlog 2A-10, ADR-0034 G3).

  bsb_gen.py [--set DIR] [--tasks ID,..] [--jobs N] [--check]

For every write task of the set (default `bench/m1b`) and every target
(`def` and each `also_check` entry):

1. The reference solution is applied to a copy of the seed.
2. Mutants: every W0904 change (`bmb mutate`) to the target and to each
   executable function it calls, directly or transitively, in the reference
   workspace. The compiler makes the mutant program; nothing here edits text.
3. Each mutant and the reference run on the task's fixed inputs (below; all
   satisfy the hidden `ref_pre`) through a small driver compiled by the real
   back end: `bmb emit-llvm --all-checks` (no proof, so no contract is
   trusted and every overflow, bounds or division check stays a trap) and
   clang. A trap ends a run; the driver is rebuilt from the next input.
4. Classification, in order:
   - `no_check`   the mutant program does not pass the front end;
   - `equivalent` same output (or same trap) on every input: no
                  distinguishing input among the fixed inputs;
   - `trap_only`  differs only by trapping or hanging (a contract states
                  results, so it cannot rule this out; the proof gate does);
   - `allowed`    terminates with a different output, but every such output
                  satisfies the hidden reference contract (an allowed variant,
                  e.g. another index of an equal minimum);
   - kept         otherwise. Witnesses: up to `MAX_WITNESSES` inputs, in input
                  order, whose mutant output the hidden reference contract
                  rejects. Rejection is proved by `bmb verify` with the same
                  kill check the harness uses (`bsb.py`), against a stub
                  that has the hidden contract.

Output: `<set>/mutants/<task-id>.json` and `<set>/mutants/MANIFEST`. The task
set's own MANIFEST is not touched. `--check` regenerates in memory and
compares with the files (determinism). Standard library only.
"""
import argparse
import concurrent.futures
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.dont_write_bytecode = True
import bsb  # noqa: E402

MAX_CANDIDATES = 6
MAX_WITNESSES = 2
RUN_TIMEOUT = 10


def bmb_bin():
    return os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


def bmb(*args):
    r = subprocess.run([bmb_bin(), *args], capture_output=True, text=True)
    return r.returncode, r.stdout


# ---------- fixed inputs (each satisfies the task's hidden ref_pre) ----------

CAP = 1000000000


def dedupe(xs):
    seen, out = set(), []
    for x in xs:
        k = json.dumps(x, sort_keys=True)
        if k not in seen:
            seen.add(k)
            out.append(x)
    return out


def rand_arrays(r, n, lens, lo, hi, extra=()):
    vals = list(range(lo, hi + 1)) if hi - lo <= 20 else None
    out = []
    for _ in range(n):
        k = r.choice(lens)
        if vals is not None:
            out.append([r.choice(vals) for _ in range(k)])
        else:
            out.append([r.choice(list(extra)) if extra and r.random() < 0.4 else r.randint(lo, hi)
                        for _ in range(k)])
    return out


def inputs_for(task_id, target):
    r = random.Random(f"bsb:{task_id}:{target}")
    t = task_id[:3]
    if t == "w01":
        fixed = [([], 0, 1), ([1], 1, 1), ([0], 1, 2), ([3], 1, 2), ([1, 2], 1, 2), ([2, 1, 0], 1, 2),
                 ([1, 2, 3], 1, 2), ([1], 2, 1), ([-1, 0], -1, 0)]
        rnd = [(a, lo, lo + r.randint(-1, 3)) for a in rand_arrays(r, 30, [0, 1, 2, 3, 4], -1, 4)
               for lo in [r.randint(-1, 2)]]
        return [{"a": a, "lo": lo, "hi": hi} for a, lo, hi in fixed + rnd]
    if t == "w02":
        fixed = [([], 0), ([1], 1), ([1], 2), ([1, 1], 1), ([1, 2, 1], 1), ([2, 1, 1], 2), ([0, 1, 2], 2)]
        rnd = [(a, r.randint(0, 2)) for a in rand_arrays(r, 30, [0, 1, 2, 3, 4, 5], 0, 2)]
        return [{"a": a, "v": v} for a, v in fixed + rnd]
    if t in ("w03", "w24"):
        fixed = [(1, 2, 3), (1, 3, 2), (2, 1, 3), (2, 3, 1), (3, 1, 2), (3, 2, 1), (1, 1, 1),
                 (1, 1, 2), (1, 2, 1), (2, 1, 1), (2, 2, 1), (2, 1, 2), (1, 2, 2), (-5, 0, 5)]
        rnd = [tuple(r.randint(-3, 3) for _ in range(3)) for _ in range(20)]
        return [{"a": a, "b": b, "c": c} for a, b, c in fixed + rnd]
    if t in ("w04", "w25"):
        fixed = [[5], [3, 1, 2], [1, 1], [2, 1, 1], [1, 2, 3], [3, 2, 1], [0, 3, 3, 0]]
        return [{"a": a} for a in fixed + rand_arrays(r, 30, [1, 2, 3, 4, 5], -2, 3)]
    if t == "w05":
        out = []
        for src in [[], [1], [1, 2], [1, 2, 3], [4, 4, 5]] + rand_arrays(r, 25, [0, 1, 2, 3, 4], -5, 5):
            out.append({"dst": [9] * len(src), "src": src})
            out.append({"dst": [r.randint(-5, 5) for _ in src], "src": src})
        return out
    if t == "w06":
        fixed = [([], 0), ([1], 1), ([1, 2], -1), ([1000], 1000), ([-1000], -1000), ([1000, -1000], 0),
                 ([5, 6, 7], 3)]
        rnd = [(a, r.choice([-1000, -7, -1, 0, 1, 2, 7, 1000]))
               for a in rand_arrays(r, 25, [0, 1, 2, 3, 4], -1000, 1000, extra=(-1000, -1, 0, 1, 1000))]
        return [{"a": a, "d": d} for a, d in fixed + rnd]
    if t == "w07":
        ns = [0, 1, 5, 9, 10, 11, 19, 20, 50, 99, 100, 101, 999, 1000, 1001, 9999, 10000, 12345,
              99999, 100000, 999999, 1000000, 1000000000, 1000000000000]
        return [{"n": n} for n in ns]
    if t == "w08":
        return [{"n": n} for n in list(range(13)) + [50, 99, 100, 1000, 9999, 100000, 999999, 1000000]]
    if t == "w09":
        fixed = [([], 0), ([1], 1), ([1], 0), ([1], 2), ([1, 2], 2), ([1, 1, 2], 1), ([0, 2, 4], 3),
                 ([0, 2, 4], 4), ([0, 2, 4], 5), ([0, 2, 4], -1)]
        rnd = [(sorted(a), r.randint(-1, 5)) for a in rand_arrays(r, 30, [0, 1, 2, 3, 4, 5], 0, 4)]
        return [{"a": a, "t": x} for a, x in fixed + rnd]
    if t == "w10":
        fixed = [[], [2], [3], [1], [2, 3], [2, 1], [3, 0, 2], [4], [2, 3, 2, 3]]
        return [{"a": a} for a in fixed + rand_arrays(r, 30, [0, 1, 2, 3, 4], 0, 4)]
    if t == "w11":
        fixed = [([], 0, 1), ([5], 0, 1), ([-5], 0, 1), ([0, 1], 0, 1), ([2, -2, 0], -1, 1), ([3], 3, 3)]
        rnd = []
        for a in rand_arrays(r, 30, [0, 1, 2, 3, 4], -3, 3):
            lo = r.randint(-2, 2)
            rnd.append((a, lo, lo + r.randint(0, 2)))
        return [{"a": a, "lo": lo, "hi": hi} for a, lo, hi in fixed + rnd]
    if t == "w12":
        fixed = [[0], [5], [1, 2], [2, 1], [-1000, 1000], [1000, -1000], [-1000], [1000], [3, -4, 7]]
        rnd = rand_arrays(r, 25, [1, 2, 3, 4], -1000, 1000, extra=(-1000, -1, 0, 1, 1000))
        return [{"a": a} for a in fixed + rnd]
    if t in ("w13", "w18"):
        fixed = [[1], [1, 2], [1, 2, 3], [1, 2, 3, 4], [5, 5], [4, 3, 2, 1]]
        return [{"a": a} for a in fixed + rand_arrays(r, 25, [1, 2, 3, 4, 5], -4, 6)]
    if t == "w14":
        vals = [0, 1, 5, 10, CAP - 1, CAP]
        fixed = [([10, 0], 0, 1, 5), ([10, 0], 1, 0, 5), ([10, 0], 0, 0, 5), ([3, 0], 0, 1, 5),
                 ([10, CAP], 0, 1, 1), ([10, CAP - 5], 0, 1, 5), ([5, 0, 7], 0, 1, 5), ([5, 0, 7], 2, 0, 0),
                 ([1, 2, 3], 1, 2, 1)]
        rnd = []
        for _ in range(30):
            a = [r.choice(vals) for _ in range(r.randint(1, 4))]
            rnd.append((a, r.randrange(len(a)), r.randrange(len(a)), r.choice([0, 1, 5, 10, CAP])))
        return [{"a": a, "i": i, "j": j, "x": x} for a, i, j, x in fixed + rnd]
    if t == "w15":
        vals = [0, 1, 5, 10, CAP - 1, CAP]
        fixed = [([10, 0], 0, 5), ([10, 0, 0], 0, 5), ([10, 0, 0], 0, 4), ([3, 0], 0, 5), ([10, CAP], 0, 1),
                 ([7], 0, 1), ([0, 10, 0], 1, 5)]
        rnd = []
        for _ in range(30):
            a = [r.choice(vals) for _ in range(r.randint(1, 4))]
            rnd.append((a, r.randrange(len(a)), r.choice([0, 1, 5, 10, CAP])))
        return [{"a": a, "from": f, "x": x} for a, f, x in fixed + rnd]
    if t == "w16":
        return [{"s": s, "ev": ev} for s in range(4) for ev in range(-2, 4)]
    if t == "w17":
        fixed = [([], 0), ([], 2), ([1], 0), ([1, 2], 0), ([1, -1], 0), ([1, -1, 1], 0), ([0, 2], 1)]
        rnd = [(e, r.randint(0, 3)) for e in rand_arrays(r, 30, [0, 1, 2, 3, 4], -1, 3)]
        return [{"events": e, "s0": s} for e, s in fixed + rnd]
    if t == "w19":
        fixed = [([1], 0), ([2, 1], 0), ([1, 2], 0), ([3, 1, 2], 0), ([3, 1, 2], 1), ([3, 2, 1], 2),
                 ([1, 3, 2, 0], 1)]
        rnd = [(a, r.randrange(len(a))) for a in rand_arrays(r, 30, [1, 2, 3, 4, 5], -3, 3)]
        return [{"a": a, "i": i} for a, i in fixed + rnd]
    if t == "w20":
        if target == "num::abs_diff":
            vs = [-CAP, -7, -1, 0, 1, 7, CAP]
            return [{"a": a, "b": b} for a in vs for b in vs]
        fixed = [[], [5], [1, 3], [3, 1], [1, 5, 2], [-1000, 1000], [1000, -1000, 1000], [0, 0, 0],
                 [1, 2, 4, 7]]
        rnd = rand_arrays(r, 25, [0, 1, 2, 3, 4, 5], -1000, 1000, extra=(-1000, -1, 0, 1, 1000))
        return [{"a": a} for a in fixed + rnd]
    if t == "w21":
        vals = [0, 1, 5, 10, CAP - 1, CAP]
        fixed = [([], 5), ([0], 5), ([CAP], 1), ([CAP - 1, 0], 1), ([CAP - 5, 0, CAP], 5), ([1, 2], 0)]
        rnd = [([r.choice(vals) for _ in range(r.randint(0, 4))], r.choice([0, 1, 5, 10, CAP]))
               for _ in range(30)]
        return [{"a": a, "x": x} for a, x in fixed + rnd]
    if t == "w22":
        out = []
        for a in [[], [1], [1, 2], [1, 2, 3], [-1000, 1000, -1000], [1000, 1000]] + rand_arrays(
                r, 25, [0, 1, 2, 3, 4], -1000, 1000, extra=(-1000, -1, 0, 1, 1000)):
            out.append({"a": a, "out": [r.randint(-9, 9) for _ in a]})
        return out
    if t == "w23":
        avs = [-CAP, -8, -7, -6, -1, 0, 1, 6, 7, 8, CAP]
        bvs = [1, 2, 3, 7, CAP]
        return [{"a": a, "b": b} for a in avs for b in bvs]
    raise SystemExit(f"no inputs for {task_id}")


# ---------- reference workspace ----------

def reference_workspace(set_dir, task, tmp):
    ws = os.path.join(tmp, "ws")
    shutil.copytree(os.path.join(set_dir, "seed"), ws)
    ops = os.path.join(tmp, "ops.json")
    with open(ops, "w") as f:
        json.dump(task["reference"], f)
    code, out = bmb("apply", ws, ops)
    if code != 0:
        raise SystemExit(f"{task['id']}: reference does not apply: {out}")
    return ws


def callees(ws, d):
    """`d` and the executable functions it calls, directly or transitively."""
    found, todo = [d], [d]
    while todo:
        x = todo.pop(0)
        _, out = bmb("query", "edges", x, "out", "1", "--", ws)
        for line in out.splitlines():
            if line.startswith("{"):
                r = json.loads(line)
                if r.get("kind") == "calls" and r["to"] not in found:
                    found.append(r["to"])
                    todo.append(r["to"])
    return [found[0]] + sorted(found[1:])


def unit_texts(ws):
    src = os.path.join(ws, "src")
    return {f[:-4]: open(os.path.join(src, f)).read() for f in sorted(os.listdir(src)) if f.endswith(".bmb")}


# ---------- running ----------

SHOW = """    #[effects(io)]
    fn show_bool(b: bool) -> () {
        if b {
            print_i64(1);
        } else {
            print_i64(0);
        }
    }

    #[effects(io)]
    fn show_u64(n: u64) -> () {
        let mut x: i64 = 0;
        let mut p: i64 = 1;
        let mut m: u64 = n;
        while m > 0
            decreases m,
        {
            if m % 2 == 1 {
                x = x + p;
            }
            m = m / 2;
            if m > 0 {
                p = p * 2;
            }
        }
        print_i64(x);
    }

    #[effects(io)]
    fn show_usize(n: usize) -> () {
        let mut x: i64 = 0;
        let mut p: i64 = 1;
        let mut m: usize = n;
        while m > 0
            decreases m,
        {
            if m % 2 == 1 {
                x = x + p;
            }
            m = m / 2;
            if m > 0 {
                p = p * 2;
            }
        }
        print_i64(x);
    }
"""


def driver(target, signature, inputs):
    params, ret = bsb.params_of(signature)
    body = []
    for i, inp in enumerate(inputs):
        res = f"r_{i}"
        body += bsb.call_lines(target, signature, inp, result=res, suffix=f"_{i}")
        if ret == "bool":
            body.append(f"show_bool({res});")
        elif ret == "i64":
            body.append(f"print_i64({res});")
        elif ret == "u64":
            body.append(f"show_u64({res});")
        elif ret == "usize":
            body.append(f"show_usize({res});")
        for name, ty in params:
            if ty == "&mut [i64]":
                body += [f"print_i64({name}_{i}[{k}]);" for k in range(len(inp[name]))]
    return ('#[summary("bsb driver")]\nmod bsb_driver {\n' + SHOW
            + "\n    #[effects(io)]\n    fn main() -> () {\n"
            + "".join(f"        {l}\n" for l in body) + "    }\n}\n")


def group_size(signature, inp):
    params, ret = bsb.params_of(signature)
    return (0 if ret == "()" else 1) + sum(len(inp[n]) for n, ty in params if ty == "&mut [i64]")


def parse_group(signature, inp, nums):
    params, ret = bsb.params_of(signature)
    out, k = {}, 0
    if ret != "()":
        out["r"] = bool(nums[0]) if ret == "bool" else nums[0]
        k = 1
    for name, ty in params:
        if ty == "&mut [i64]":
            out[name] = nums[k:k + len(inp[name])]
            k += len(inp[name])
    return out


def run_program(units, target, signature, inputs, tmp):
    """Results per input: {"ok": output} | {"trap": true} | {"hang": true};
    None when the program does not pass the front end."""
    d = tempfile.mkdtemp(dir=tmp)
    files = []
    for name, text in units.items():
        if name == "app":  # the seed's entry point; the driver has its own `main`
            continue
        p = os.path.join(d, name + ".bmb")
        with open(p, "w") as f:
            f.write(text)
        files.append(p)
    results, start = [], 0
    try:
        while start < len(inputs):
            rest = inputs[start:]
            drv = os.path.join(d, "bsb_driver.bmb")
            with open(drv, "w") as f:
                f.write(driver(target, signature, rest))
            ll, exe = os.path.join(d, "p.ll"), os.path.join(d, "p")
            code, _ = bmb("emit-llvm", "--all-checks", *files, drv, "-o", ll)
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
                n = group_size(signature, inp)
                if k + n > len(nums):
                    break
                results.append({"ok": parse_group(signature, inp, nums[k:k + n])})
                k += n
                done += 1
            if done == len(rest):
                break
            if rc == 0:
                raise SystemExit(f"driver output is short: {stdout!r}")
            # The input after the last complete one trapped (or hung).
            results.append({"hang": True} if hung else {"trap": True})
            start = len(results)
    finally:
        shutil.rmtree(d, ignore_errors=True)
    return results


# ---------- classification ----------

def stub_prelude(t):
    """The hidden reference contract on a stub with the target's signature."""
    a = t["accept"]
    params, ret_ty = raw_signature(t["signature"])
    ret = f"(r: {ret_ty})" if ret_ty != "()" else "()"
    dummy = {"()": "", "bool": "    false\n"}.get(ret_ty, "    0\n")
    req = f"    requires {a['pre']},\n" if a["pre"] else ""
    return (a["specs"] + "\n\n#[effects(pure)]\n"
            + f"fn stub({params}) -> {ret}\n{req}    ensures {a['post']},\n{{\n{dummy}}}\n\n")


def raw_signature(sig):
    """The parameter list text and the result type of a signature."""
    m = re.match(r"fn \w+\((.*)\) -> (.*)$", sig.strip())
    ret = m.group(2)
    m2 = re.match(r"\((\w+): (.*)\)$", ret)
    return m.group(1), (m2.group(2) if m2 else ret)


def rejected_by_reference(t, candidates):
    """For each (mutant index, witness) candidate: does the hidden reference
    contract reject the mutant's output there? One `bmb verify` run."""
    uses = ["ref_post"] + (["ref_pre"] if t["accept"]["pre"] else [])
    by_m = {}
    for i, w in candidates:
        by_m.setdefault(i, []).append(w)
    order = sorted(by_m)
    # One pseudo-mutant per candidate so each witness gets its own verdict.
    flat = [{"witnesses": [w]} for i in order for w in by_m[i]]
    keys = [(i, j) for i in order for j in range(len(by_m[i]))]
    verdict = bsb.kill_check([("stub", t["signature"], flat)], uses, set(),
                             lambda f: bmb("verify", f)[1], prelude=stub_prelude(t))[0]
    out = {}
    for (i, j), v in zip(keys, verdict):
        out.setdefault(i, []).append((by_m[i][j], v))
    return out


def gen_target(set_dir, task, t, jobs):
    inputs = dedupe(inputs_for(task["id"], t["def"]))
    with tempfile.TemporaryDirectory(prefix="bsb-gen-") as tmp:
        ws = reference_workspace(set_dir, task, tmp)
        units = unit_texts(ws)
        ref = run_program(units, t["def"], t["signature"], inputs, tmp)
        if ref is None or any("ok" not in x for x in ref):
            raise SystemExit(f"{task['id']}: the reference does not run cleanly on every input")
        defs = callees(ws, t["def"])
        args = [a for d in defs for a in ("--def", d)]
        code, out = bmb("mutate", *args, ws)
        if code != 0:
            raise SystemExit(f"{task['id']}: bmb mutate failed: {out}")
        rows = [json.loads(l) for l in out.splitlines() if l.startswith("{")]
        muts = [r for r in rows if not r.get("summary")]

        def run_one(m):
            u = dict(units)
            u[m["unit"]] = m["text"]
            return run_program(u, t["def"], t["signature"], inputs, tmp)

        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
            runs = list(pool.map(run_one, muts))

    records, candidates = [], []
    for i, (m, res) in enumerate(zip(muts, runs)):
        rec = {k: m[k] for k in ("def", "site", "line", "col", "before", "after")}
        if res is None:
            rec["reason"] = "no_check"
        else:
            diff = [k for k in range(len(inputs)) if res[k] != ref[k]]
            okdiff = [k for k in diff if "ok" in res[k]]
            if not diff:
                rec["reason"] = "equivalent"
            elif not okdiff:
                rec["reason"] = "trap_only"
            else:
                for k in okdiff[:MAX_CANDIDATES]:
                    candidates.append((i, {"input": inputs[k], "expected": ref[k]["ok"], "output": res[k]["ok"]}))
        records.append(rec)
    verdicts = rejected_by_reference(t, candidates) if candidates else {}
    kept, discarded = [], []
    for i, rec in enumerate(records):
        if "reason" not in rec:
            ws_ = [w for w, v in verdicts.get(i, []) if v][:MAX_WITNESSES]
            if ws_:
                rec["witnesses"] = ws_
            else:
                rec["reason"] = "allowed"
        (discarded if "reason" in rec else kept).append(rec)
    for n, rec in enumerate(kept):
        rec["id"] = f"m{n:02d}"
    counts = {"generated": len(records), "kept": len(kept)}
    for reason in ("equivalent", "trap_only", "allowed", "no_check"):
        counts[reason] = sum(1 for r in discarded if r["reason"] == reason)
    return {"def": t["def"], "signature": t["signature"], "mutated": defs, "inputs": inputs,
            "counts": counts, "mutants": kept, "discarded": discarded}


def gen_task(set_dir, task, jobs):
    targets = [task] + task.get("also_check", [])
    return {"task": task["id"], "generator": "bench/m1/bsb_gen.py", "max_witnesses": MAX_WITNESSES,
            "targets": [gen_target(set_dir, task, t, jobs) for t in targets]}


def dump(obj):
    return json.dumps(obj, ensure_ascii=False, indent=1) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--set", default=os.path.join(ROOT, "bench", "m1b"))
    ap.add_argument("--tasks")
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--check", action="store_true", help="regenerate and compare with the files")
    args = ap.parse_args()
    set_dir = os.path.abspath(args.set)
    tdir = os.path.join(set_dir, "tasks")
    tasks = [json.load(open(os.path.join(tdir, f))) for f in sorted(os.listdir(tdir))]
    tasks = [t for t in tasks if t["kind"] == "write"]
    if args.tasks:
        wanted = set(args.tasks.split(","))
        tasks = [t for t in tasks if t["id"] in wanted]
    out_dir = bsb.mutants_dir(set_dir)
    os.makedirs(out_dir, exist_ok=True)
    bad = []
    for task in tasks:
        text = dump(gen_task(set_dir, task, args.jobs))
        path = os.path.join(out_dir, task["id"] + ".json")
        c = json.loads(text)
        summary = " ".join(f"{t['def']}: " + ", ".join(f"{k} {v}" for k, v in t["counts"].items())
                           for t in c["targets"])
        print(f"{task['id']}: {summary}", flush=True)
        if args.check:
            if not os.path.exists(path) or open(path).read() != text:
                bad.append(task["id"])
        else:
            with open(path, "w") as f:
                f.write(text)
    if args.check:
        if bad:
            sys.exit("mutant files differ from regeneration: " + ", ".join(bad))
        print("mutant files reproduce")
    else:
        with open(os.path.join(out_dir, "MANIFEST"), "w") as f:
            f.write(bsb.manifest_text(set_dir))


if __name__ == "__main__":
    main()
