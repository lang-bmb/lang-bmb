#!/usr/bin/env python3
"""H-REVIEW's change set (ADR-0078, backlog T1-44), derived from the frozen BVB
tasks (`bench/bvb/tasks`). Standard library only; the body-changing changes
need the compiler (`mutants.py`).

  python3 bench/review/gen.py            writes bench/review/changes/ (needs `bmb`: $BMB or target/debug/bmb)
  python3 bench/review/gen.py --check    no compiler: the committed set has the structure this generator gives
  python3 bench/review/gen.py --regen    needs `bmb`: a fresh build equals the committed set byte for byte

A BVB task has a `reference` (code whose contract states the task's intent)
and a `weak_reference` (the same code with a contract that is consistent and
proved but does not state the intent). Each task gives:

  <task>.weaken           reference -> weak_reference                label `defect`  contract-weakened
  <task>.restore          weak_reference -> reference                label `clean`   contract-restored
  <task>.mutant-defect-N  reference -> weak contract + body mutant   label `defect`  contract-weakened+body
  <task>.mutant-clean-N   reference -> reference + body mutant       label `clean`   body-only

The first two need no compiler. The mutant changes are the first
`mutants.K` qualifying one-site body mutants (`bmb mutate`) per task and kind,
in the compiler's row order, N from 1: `mutant-clean` still proves under the
unchanged reference contract; `mutant-defect` proves under the weak contract
and has a witness input on which its output differs from the reference's and
is rejected by the hidden reference contract (`mutants.py` says how each is
decided). A task with no qualifying mutant has none.

Nothing is written by hand: the intent is the task's description (its prompt
without the "Create unit" line and the "Required signature" lines), and each
program is the task's `put` code, indented, inside the unit its `unit` op
declares, with at most one replacement the compiler named. The text keeps the
task's definition order (it is not the compiler's normal form).
A program is that one file next to the BVB seed (`given`).

The committed `changes/` is the frozen artifact (ADR-0021 §1): which mutants
qualify depends on the compiler, so the set is not regenerated for a run. It
was generated with the compiler commit recorded in `set.json`
(`generated_with`). `--check` re-derives what needs no compiler (the first
two kinds byte for byte; for a mutant change, that `before` is the reference
program and `after` is the named program with the recorded replacement).
`--regen` runs the whole generation again. The gate runs both.

The label is hidden from the reviewer (`views.py` never shows it). The gate
(`test_review.py`) checks it against the compiler: both programs of every
change are proved, and the `after` program's contract implies BVB's hidden
reference contract exactly for `clean` changes.
"""
import concurrent.futures
import json
import os
import shutil
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
BENCH = os.path.dirname(HERE)
SOURCE = "bvb"
SEED = "../bvb/seed"
HEAD = "Create unit `tasks` for this task.\n\n"
TAIL = "\n\nRequired signature (in unit `tasks`):\n"
# (suffix, before, after, label, kind)
MUTANT_KINDS = [("mutant-defect", None, "weak_reference", "defect", "contract-weakened+body"),
                ("mutant-clean", None, "reference", "clean", "body-only")]
KINDS = [("weaken", "reference", "weak_reference", "defect", "contract-weakened"),
         ("restore", "weak_reference", "reference", "clean", "contract-restored")]


def intent(task):
    """The task's description: what the function must do and what its
    contract must state (and the given spec fns, when the task shows them)."""
    p, tail = task["prompt"], TAIL + task["signature"]
    if not (p.startswith(HEAD) and p.endswith(tail)):
        raise SystemExit(f"{task['id']}: prompt is not `Create unit` + description + `Required signature`")
    return p[len(HEAD):-len(tail)]


def program(task, key):
    """(file, text) of the unit the ops of `task[key]` create."""
    ops = task[key]
    if [o["op"] for o in ops] != ["unit", "put"] or ops[1]["unit"] != ops[0]["path"]:
        raise SystemExit(f"{task['id']}: {key} is not one `unit` and one `put` into it")
    unit, summary = ops[0]["path"], ops[0]["summary"]
    if '"' in summary or "\\" in summary:
        raise SystemExit(f"{task['id']}: unit summary needs escaping")
    body = "\n".join("    " + line if line else "" for line in ops[1]["code"].split("\n"))
    return f"src/{unit}.bmb", f'#[summary("{summary}")]\nmod {unit} {{\n{body}\n}}\n'


def load_tasks():
    tdir = os.path.join(BENCH, SOURCE, "tasks")
    out = []
    for name in sorted(os.listdir(tdir)):
        with open(os.path.join(tdir, name)) as f:
            out.append(json.load(f))
    return out


def base_changes(task):
    """The two contract-only changes of a task (no compiler)."""
    out = []
    for suffix, a, b, label, kind in KINDS:
        (file, before), (file_b, after) = program(task, a), program(task, b)
        if file != file_b or before == after:
            raise SystemExit(f"{task['id']}: the two references do not make a change to one file")
        out.append({"id": f"{task['id']}.{suffix}",
                    "source": {"set": SOURCE, "task": task["id"], "before": a, "after": b},
                    "intent": intent(task), "seed": SEED, "file": file,
                    "before": before, "after": after, "label": label, "kind": kind})
    return out


def mutant_change(task, suffix, n, m):
    """The change of one qualifying mutant `m` (`mutants.for_task`)."""
    _, _, key, label, kind = next(k for k in MUTANT_KINDS if k[0] == suffix)
    file, before = program(task, "reference")
    c = {"id": f"{task['id']}.{suffix}-{n}",
         "source": {"set": SOURCE, "task": task["id"], "before": "reference", "after": key,
                    "mutation": m["mutation"]},
         "intent": intent(task), "seed": SEED, "file": file,
         "before": before, "after": m["program"], "label": label, "kind": kind}
    if "witness" in m:
        c["witness"] = m["witness"]
    return c


def body_changes(task, bmb):
    """The mutant changes of a task (needs the compiler) and the counts."""
    import mutants
    file, ref = program(task, "reference")
    _, weak = program(task, "weak_reference")
    seed = os.path.join(HERE, SEED, "src", "given.bmb")
    clean, defect, counts = mutants.for_task(bmb, task, seed, file, ref, weak)
    return ([mutant_change(task, "mutant-defect", i + 1, m) for i, m in enumerate(defect)]
            + [mutant_change(task, "mutant-clean", i + 1, m) for i, m in enumerate(clean)]), counts


def changes(bmb, jobs=2, report=None):
    tasks = load_tasks()
    with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
        def one(task):
            start = time.monotonic()
            body, counts = body_changes(task, bmb)
            if report:  # as tasks finish; the seconds are not part of the set
                report(task["id"], dict(counts, seconds=round(time.monotonic() - start, 1)))
            return body
        bodies = list(pool.map(one, tasks))
    out = []
    for task, body in zip(tasks, bodies):
        out += base_changes(task) + body
    return sorted(out, key=lambda c: c["id"])


def dump(c):
    return json.dumps(c, ensure_ascii=False, indent=1) + "\n"


def build(root, bmb, jobs=2, report=None):
    out = os.path.join(root, "changes")
    if os.path.isdir(out):
        shutil.rmtree(out)
    os.makedirs(out)
    for c in changes(bmb, jobs, report):
        with open(os.path.join(out, c["id"] + ".json"), "w") as f:
            f.write(dump(c))
    return out


def tree(d):
    out = {}
    for name in sorted(os.listdir(d)):
        with open(os.path.join(d, name), "rb") as f:
            out[name] = f.read()
    return out


def structure_problems(d):
    """What is wrong with the committed set `d`, checked without a compiler."""
    import mutants
    bad, got, want = [], tree(d), {}
    tasks = {t["id"]: t for t in load_tasks()}
    for t in tasks.values():
        for c in base_changes(t):
            want[c["id"] + ".json"] = dump(c).encode()
    for name, data in want.items():
        if got.get(name) != data:
            bad.append(f"{name}: not the generator's contract-only change")
    seen = {}
    for name in sorted(set(got) - set(want)):
        try:
            c = json.loads(got[name])
            task = tasks[c["source"]["task"]]
            suffix, n = c["id"][len(task["id"]) + 1:].rsplit("-", 1)
            want_c = mutant_change(task, suffix, int(n), {
                "mutation": c["source"]["mutation"], **({"witness": c["witness"]} if "witness" in c else {}),
                "program": mutants.apply_mutation(program(task, c["source"]["after"])[1], c["source"]["mutation"])})
        except (KeyError, ValueError, StopIteration, TypeError) as e:
            bad.append(f"{name}: not a mutant change ({type(e).__name__})")
            continue
        if name != c["id"] + ".json" or got[name] != dump(want_c).encode() or want_c["after"] in (None, c["before"]):
            bad.append(f"{name}: not the reference program with the recorded one-site replacement")
        if (suffix == "mutant-defect") != ("witness" in c):
            bad.append(f"{name}: a witness goes with a mutant-defect change and only with it")
        seen.setdefault((task["id"], suffix), []).append(int(n))
    for (task_id, suffix), ns in seen.items():
        if sorted(ns) != list(range(1, len(ns) + 1)) or len(ns) > mutants.K:
            bad.append(f"{task_id}: {suffix} changes are not 1..n with n <= {mutants.K}")
    return bad


def bmb_bin():
    return os.environ.get("BMB", os.path.join(os.path.dirname(BENCH), "target", "debug", "bmb"))


def main():
    committed = os.path.join(HERE, "changes")
    if "--check" in sys.argv:
        if bad := structure_problems(committed):
            sys.exit("bench/review/changes: " + "; ".join(bad[:5]) + f" ({len(bad)} problems)")
        print("bench/review/changes ok (structure)")
    elif "--regen" in sys.argv:
        tmp = tempfile.mkdtemp(prefix="review-")
        try:
            if tree(build(tmp, bmb_bin())) != tree(committed):
                sys.exit("bench/review/changes differs from bench/review/gen.py output: regenerate it")
            print("bench/review/changes ok (regenerated)")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    else:
        print(build(HERE, bmb_bin(), report=lambda task, counts: print(task, json.dumps(counts), flush=True)))


if __name__ == "__main__":
    main()
