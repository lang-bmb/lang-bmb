#!/usr/bin/env python3
"""Contract weakening on a commit history (backlog 2D-19, `bmb weaken`).

  python3 dev-docs/experiments/weaken/run.py --work DIR [--bmb PATH] [--profile P]
          [--defs N] [--commits K] > section.md

Replays the BCB history (bench/bcb/history.py) and compares every commit with
the one before it (`bmb weaken before after`). The generator knows what each
edit did to a contract, which is the oracle:

  body      the interface stays: `same`
  contract  a leaf's ensures gains its `_ok` predicate: `refined`; loses it: `weakened`
  spec      a `_ok` predicate's bound rises (weaker): `weakened` for its leaf,
            falls: `refined`; `same` when the leaf does not use it
  add       `added`;  remove: `removed`

Every other definition (its `id` changed only through what it calls, or not
at all) must be `same`. A definition two edits of one commit touch is
`mixed` and not judged. Standard library only.
"""
import argparse
import collections
import contextlib
import copy
import json
import os
import shutil
import subprocess
import sys
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "bench", "bcb"))
import history  # noqa: E402


def write(tree, d):
    src = os.path.join(d, "src")
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(src)
    for u in range(tree.n_units):
        with open(os.path.join(src, f"m{u}.bmb"), "w", encoding="utf-8") as f:
            f.write(tree.text[("bmb", u)])
    return d


def expect(tree, e):
    """The verdicts edit `e` should give, on the tree before it."""
    u, op = e["unit"], e["op"]
    path = lambda n: f"m{u}::{n}"  # noqa: E731
    if op == "body":
        return {path(e["def"]): "same"}
    if op == "add":
        return {path(e["def"]): "added"}
    if op == "remove":
        return {path(d): "removed" for d in e["defs"] if not d.endswith("_ok")}
    if op == "contract":
        return {path(e["def"]): "refined" if e["set"]["post"] else "weakened"}
    if op == "spec":
        _, _, info = tree.find(u, e["def"])
        _, _, leaf = tree.find(u, info["of"])
        if not leaf["post"]:
            return {path(info["of"]): "same"}
        return {path(info["of"]): "weakened" if e["set"]["c"] > info["c"] else "refined"}
    raise ValueError(op)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    ap.add_argument("--profile", default="contract-heavy")
    ap.add_argument("--defs", type=int, default=200)
    ap.add_argument("--commits", type=int, default=200)
    args = ap.parse_args()
    os.makedirs(args.work, exist_ok=True)
    hist = os.path.join(args.work, f"{args.profile}-{args.defs}-{args.commits}.jsonl")
    if not os.path.exists(hist):
        with contextlib.redirect_stdout(sys.stderr):
            history.main(["gen", hist, "--defs", str(args.defs), "--commits", str(args.commits),
                          "--profile", args.profile])
    header, commits = history.load(hist)
    by_k = {c["k"]: c for c in commits}
    # (edit kind, expected, got) for edited definitions; got for the others.
    judged = collections.Counter()
    others = collections.Counter()
    verdicts = collections.Counter()
    wrong = []
    queries = 0
    cpu = 0.0
    prev = None
    for k, tree, _ in history.replay(header, commits):
        if prev is None:
            prev = copy.deepcopy(tree)
            continue
        want, kinds = {}, {}
        run = copy.deepcopy(prev)
        for e in by_k[k]["edits"]:
            for d, v in expect(run, e).items():
                if d in want:
                    want[d] = "mixed"
                else:
                    want[d], kinds[d] = v, e["op"]
            run.apply(copy.deepcopy(e))
        a = write(prev, os.path.join(args.work, "a"))
        b = write(tree, os.path.join(args.work, "b"))
        t0 = os.times()
        r = subprocess.run([args.bmb, "weaken", a, b], capture_output=True, text=True)
        t1 = os.times()
        cpu += (t1.children_user - t0.children_user) + (t1.children_system - t0.children_system)
        rows = [json.loads(x) for x in r.stdout.splitlines() if x.strip()]
        for row in rows:
            if row.get("summary"):
                queries += row["queries"]
                continue
            if "verdict" not in row:
                continue
            d = row["def"] or row["old"]
            got = row["verdict"]
            verdicts[got] += 1
            w = want.get(d)
            if w is None:
                others[got] += 1
                if got != "same":
                    wrong.append((k, d, "same", got))
            elif w != "mixed":
                judged[(kinds[d], w, got)] += 1
                if got != w:
                    wrong.append((k, d, w, got))
        prev = copy.deepcopy(tree)
        print(json.dumps({"k": k, "edits": [e["op"] for e in by_k[k]["edits"]]}), file=sys.stderr, flush=True)
    report(args, judged, others, verdicts, wrong, queries, cpu)


def report(args, judged, others, verdicts, wrong, queries, cpu):
    print(f"## 프로필 `{args.profile}`, 정의 {args.defs}개, 커밋 {args.commits}개\n")
    print(f"생성: `python3 dev-docs/experiments/weaken/run.py --work W --profile {args.profile} "
          f"--defs {args.defs} --commits {args.commits}`. 커밋마다 `bmb weaken 직전 직후`.\n")
    print("| 편집 | 기대 | 판정 | 정의 수 |\n| --- | --- | --- | ---: |")
    for (kind, w, got), n in sorted(judged.items()):
        print(f"| {kind} | {w} | {got} | {n} |")
    ok = sum(n for (_, w, got), n in judged.items() if w == got)
    total = sum(judged.values())
    print(f"\n- 편집한 정의: {total}개 중 기대와 같은 판정 {ok}개.")
    print(f"- 편집하지 않은 정의(행 {sum(others.values())}개): " +
          ", ".join(f"{v} {n}" for v, n in sorted(others.items())) + ".")
    print("- 전체 판정: " + ", ".join(f"{v} {n}" for v, n in sorted(verdicts.items())) + ".")
    print(f"- 해법기 의무 {queries}개, `bmb weaken` CPU {cpu:.1f}초(커밋 {args.commits}개).")
    if wrong:
        print("\n| 커밋 | 정의 | 기대 | 판정 |\n| ---: | --- | --- | --- |")
        for k, d, w, got in wrong[:50]:
            print(f"| {k} | `{d}` | {w} | {got} |")


if __name__ == "__main__":
    t = time.time()
    main()
    print(f"\n(벽시계 {time.time() - t:.0f}초)", file=sys.stderr)
