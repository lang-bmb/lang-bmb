#!/usr/bin/env python3
"""H-INTENT's item set (ADR-0078, backlog T1-44e), derived from the frozen BVB
tasks (`bench/bvb/tasks`). A second set of the review harness
(`bench/review/harness.py --set bench/intent`). Standard library only; the
examples need the compiler.

  python3 bench/intent/gen.py            writes bench/intent/changes/ (needs `bmb`: $BMB or target/debug/bmb)
  python3 bench/intent/gen.py --check    no compiler: the committed set is this generator's, but for the examples' text
  python3 bench/intent/gen.py --regen    needs `bmb`: a fresh build equals the committed set byte for byte
  python3 bench/intent/gen.py --stats    what the two labels' views differ in (no compiler, no model)

An item is one proved program and the intent it was written for. The reviewer
says whether the contract of the program states the intent. A BVB task has a
`reference` (code whose contract states the task's intent) and a
`weak_reference` (the same task, a contract that is consistent and proved but
does not state the intent). Each task gives two items with the same intent:

  <task>.reference   the `reference` program        label `clean`    kind `reference-contract`
  <task>.weak        the `weak_reference` program   label `defect`   kind `weak-contract`

Nothing is written by hand: intent and program text are made as
`bench/review/gen.py` makes them (the task's description; the task's `put`
code inside its unit, next to the BVB seed). `examples` is the standard output
of

    bmb examples <the task's `def`> -- <seed>/src/given.bmb src/tasks.bmb

as the tool prints it (JSONL rows and the summary row, the tool's default
count). The tool reads the contract and not the body.

The directory is named `changes/` and the program is the item's `after`
because that is where the review harness reads a set from; an item has no
`before`.

The committed `changes/` is the frozen artifact (ADR-0021 §1): the examples
are the compiler's and the solver's, so the set is not regenerated for a run.
The reviewer is shown the committed text. It was generated with the compiler
commit recorded in `set.json` (`generated_with`). `--check` re-derives
everything but `examples` byte for byte and checks that `examples` has the
tool's shape for the item's definition. `--regen` runs the tool again.

The label is hidden from the reviewer (`bench/review/views.py` never shows
it). The gate (`test_intent.py`) checks it against the compiler: every
program is proved, and its contract implies BVB's hidden reference contract
exactly for `clean` items.
"""
import concurrent.futures
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.dont_write_bytecode = True


def _review(name):
    """A module of bench/review, loaded by path (it has a `gen.py` too)."""
    import importlib.util
    spec = importlib.util.spec_from_file_location("review_" + name, os.path.join(BENCH, "review", name + ".py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


review_gen = _review("gen")
SOURCE = review_gen.SOURCE
SEED = review_gen.SEED
# (suffix, the task's key, label, kind)
KINDS = [("reference", "reference", "clean", "reference-contract"),
         ("weak", "weak_reference", "defect", "weak-contract")]
ROW_KINDS = ("allowed", "rejected", "excluded")
ROW_KEYS = {"def", "kind", "input", "result", "after"}


def bmb_bin():
    return os.environ.get("BMB", os.path.join(os.path.dirname(BENCH), "target", "debug", "bmb"))


def bare_items(task):
    """The two items of a task without their examples (no compiler)."""
    out, texts = [], set()
    for suffix, key, label, kind in KINDS:
        file, text = review_gen.program(task, key)
        texts.add(text)
        out.append({"id": f"{task['id']}.{suffix}",
                    "source": {"set": SOURCE, "task": task["id"], "program": key},
                    "intent": review_gen.intent(task), "seed": SEED, "file": file, "def": task["def"],
                    "after": text, "label": label, "kind": kind})
    if len(texts) != len(KINDS):
        raise SystemExit(f"{task['id']}: the two references are one program")
    return out


def examples(bmb, item):
    """stdout of `bmb examples DEF -- given.bmb tasks.bmb` for the item."""
    tmp = tempfile.mkdtemp(prefix="bmb-intent-")
    try:
        path = os.path.join(tmp, os.path.basename(item["file"]))
        with open(path, "w") as f:
            f.write(item["after"])
        seed = os.path.join(HERE, item["seed"], "src", "given.bmb")
        r = subprocess.run([bmb, "examples", item["def"], "--", seed, path], capture_output=True, text=True)
        if r.returncode != 0 or r.stderr or examples_problem(item, r.stdout):
            raise RuntimeError(f"{item['id']}: bmb examples failed ({r.returncode}): {r.stdout}{r.stderr}")
        return r.stdout
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def parse_examples(text):
    """(rows, summary) of an `examples` text."""
    lines = [json.loads(line) for line in text.split("\n") if line]
    return lines[:-1], lines[-1]


def examples_problem(item, text):
    """Why `text` is not the tool's output for the item's definition, or None."""
    try:
        if not text.endswith("\n") or "\n\n" in text:
            return "not lines"
        rows, summary = parse_examples(text)
    except (ValueError, IndexError):
        return "not JSONL"
    if summary.get("summary") is not True or summary.get("def") != item["def"]:
        return "no summary row of the definition last"
    for r in rows:
        if r.get("def") != item["def"] or r.get("kind") not in ROW_KINDS or not set(r) <= ROW_KEYS:
            return "a row that is not an example of the definition"
    for k in ROW_KINDS:
        if summary.get(k) != sum(1 for r in rows if r["kind"] == k):
            return f"the summary does not count the `{k}` rows"
    if "/" in text.replace("\\/", ""):
        return "a path"
    return None


def items(bmb, jobs=2):
    bare = [i for t in review_gen.load_tasks() for i in bare_items(t)]
    with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
        texts = list(pool.map(lambda i: examples(bmb, i), bare))
    return sorted((dict(i, examples=t) for i, t in zip(bare, texts)), key=lambda i: i["id"])


def dump(item):
    return json.dumps(item, ensure_ascii=False, indent=1) + "\n"


def build(root, bmb, jobs=2):
    out = os.path.join(root, "changes")
    if os.path.isdir(out):
        shutil.rmtree(out)
    os.makedirs(out)
    for i in items(bmb, jobs):
        with open(os.path.join(out, i["id"] + ".json"), "w") as f:
            f.write(dump(i))
    return out


def tree(d):
    out = {}
    for name in sorted(os.listdir(d)):
        with open(os.path.join(d, name), "rb") as f:
            out[name] = f.read()
    return out


def structure_problems(d):
    """What is wrong with the committed set `d`, checked without a compiler."""
    bad, got = [], tree(d)
    want = {i["id"] + ".json": i for t in review_gen.load_tasks() for i in bare_items(t)}
    for name in sorted(set(got) - set(want)):
        bad.append(f"{name}: not an item of a task")
    for name, bare in want.items():
        if name not in got:
            bad.append(f"{name}: missing")
            continue
        try:
            item = json.loads(got[name])
            text = item["examples"]
        except (ValueError, KeyError, TypeError):
            bad.append(f"{name}: not an item with examples")
            continue
        if not isinstance(text, str) or got[name] != dump(dict(bare, examples=text)).encode():
            bad.append(f"{name}: not the generator's item")
        elif why := examples_problem(bare, text):
            bad.append(f"{name}: examples: {why}")
    return bad


# ---------- what the labels' views differ in ----------

def features(item):
    """Counts read off an item's examples: the summary's numbers, the inputs
    shown with an `allowed` row, and those shown with two."""
    rows, summary = parse_examples(item["examples"])
    per_input = {}
    for r in rows:
        if r["kind"] == "allowed":
            key = json.dumps(r["input"], sort_keys=True)
            per_input[key] = per_input.get(key, 0) + 1
    return {"rows": len(rows), "allowed": summary["allowed"], "rejected": summary["rejected"],
            "excluded": summary["excluded"], "undecided": summary["undecided"], "reason": "reason" in summary,
            "allowed_inputs": len(per_input), "two_allowed_inputs": sum(1 for n in per_input.values() if n > 1)}


TESTS = [("행이 하나도 없음", lambda f: f["rows"] == 0),
         ("`allowed` 행 없음", lambda f: f["allowed"] == 0),
         ("`rejected` 행 없음", lambda f: f["rejected"] == 0),
         ("`excluded` 행 없음", lambda f: f["excluded"] == 0),
         ("`undecided` > 0", lambda f: f["undecided"] > 0),
         ("요약에 `reason`(질의 없음)", lambda f: f["reason"]),
         ("한 입력에 `allowed` 결과가 둘", lambda f: f["two_allowed_inputs"] > 0),
         ("`allowed` 행이 있고 입력마다 결과 하나", lambda f: f["allowed"] > 0 and f["two_allowed_inputs"] == 0)]


def stats(all_items):
    """{"counts": {test: {label: items}}, "same_examples_tasks": tasks whose
    two items have the same examples text, "mean_rows": {label: rows}}."""
    by = {}
    for i in all_items:
        by.setdefault(i["label"], []).append(i)
    counts = {name: {label: sum(1 for i in group if test(features(i))) for label, group in sorted(by.items())}
              for name, test in TESTS}
    tasks = {}
    for i in all_items:
        tasks.setdefault(i["source"]["task"], set()).add(i["examples"])
    return {"counts": counts,
            "same_examples_tasks": sorted(t for t, texts in tasks.items() if len(texts) == 1),
            "mean_rows": {label: sum(features(i)["rows"] for i in group) / len(group)
                          for label, group in sorted(by.items())}}


def load(d):
    return [json.loads(data) for _, data in sorted(tree(d).items())]


def stats_text(all_items):
    s = stats(all_items)
    labels = sorted(s["mean_rows"])
    n = {label: sum(1 for i in all_items if i["label"] == label) for label in labels}
    out = ["| 예시 출력의 성질 | " + " | ".join(f"`{label}` ({n[label]})" for label in labels) + " |",
           "| --- |" + " --- |" * len(labels)]
    for name, _ in TESTS:
        out.append(f"| {name} | " + " | ".join(str(s["counts"][name][label]) for label in labels) + " |")
    out.append("| 평균 행 수 | " + " | ".join(f"{s['mean_rows'][label]:.2f}" for label in labels) + " |")
    out += ["", f"두 항목의 예시 텍스트가 같은 과제: {len(s['same_examples_tasks'])} / {len(all_items) // 2}" + "".join(f"\n  {t}" for t in s["same_examples_tasks"])]
    return "\n".join(out) + "\n"


def main():
    committed = os.path.join(HERE, "changes")
    if "--check" in sys.argv:
        if bad := structure_problems(committed):
            sys.exit("bench/intent/changes: " + "; ".join(bad[:5]) + f" ({len(bad)} problems)")
        print("bench/intent/changes ok (structure)")
    elif "--regen" in sys.argv:
        tmp = tempfile.mkdtemp(prefix="intent-")
        try:
            if tree(build(tmp, bmb_bin())) != tree(committed):
                sys.exit("bench/intent/changes differs from bench/intent/gen.py output with this compiler")
            print("bench/intent/changes ok (regenerated)")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    elif "--stats" in sys.argv:
        sys.stdout.write(stats_text(load(committed)))
    else:
        print(build(HERE, bmb_bin()))


if __name__ == "__main__":
    main()
