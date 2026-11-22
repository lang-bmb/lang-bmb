#!/usr/bin/env python3
"""The shape of real edits (backlog T1-51, `dev-docs/reviews/practices-2026-11-22.md` §6).

For every commit that touched DIR (first parent, oldest first) the program's
definitions and what their proofs read (`bmb reads`), then per pair of
consecutive commits each definition is put in one class:

    added      not in the older version
    removed    not in the newer version
    body       its own text changed, its contract did not: the callers'
               proofs read nothing that changed (the contract firewall)
    contract   its own text and its contract changed
    reads      its own text is the same and its proof key changed: it is
               proved again because of what it reads
    same       nothing its proof reads changed

    run.py CACHE [--dir selfhost] [--bmb target/release/bmb] [--max N]

CACHE keeps the `reads` rows per commit (about 20 s a commit for the port).
Prints the table as Markdown. A commit whose sources `bmb reads` refuses is
left out with its neighbours' pair. Standard library only.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
CLASSES = ("added", "removed", "body", "contract", "reads", "same")


def rows_at(commit, cache, d, bmb):
    f = os.path.join(cache, commit + ".jsonl")
    if not os.path.exists(f):
        tmp = tempfile.mkdtemp()
        tar = subprocess.run(["git", "archive", commit, d], cwd=ROOT, capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", tmp], input=tar, check=True)
        files = sorted(os.path.join(tmp, d, x) for x in os.listdir(os.path.join(tmp, d)) if x.endswith(".bmb"))
        r = subprocess.run([bmb, "reads", *files], capture_output=True, text=True)
        with open(f, "w") as h:
            h.write(r.stdout if r.returncode == 0 else "")
        shutil.rmtree(tmp)
    out = {}
    with open(f) as h:
        for line in h:
            if line.startswith('{"def"'):
                r = json.loads(line)
                own = next((x["digest"] for x in r["reads"] if x["kind"] == "self"), None)
                out[r["def"]] = (r["kind"], own, r["contract"], r["key"])
    return out


def classify(a, b):
    c = Counter()
    kinds = Counter()
    for d in set(a) | set(b):
        if d not in a:
            k = "added"
        elif d not in b:
            k = "removed"
        elif a[d][1] != b[d][1]:
            k = "body" if a[d][2] == b[d][2] else "contract"
        elif a[d][3] != b[d][3]:
            k = "reads"
        else:
            k = "same"
        c[k] += 1
        kinds[(k, (b.get(d) or a[d])[0])] += 1
    return c, kinds


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("cache")
    ap.add_argument("--dir", default="selfhost")
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "release", "bmb"))
    ap.add_argument("--max", type=int, default=1000)
    args = ap.parse_args()
    os.makedirs(args.cache, exist_ok=True)
    commits = subprocess.run(["git", "log", "--format=%H", "--first-parent", "-n", str(args.max), "--", args.dir],
                             cwd=ROOT, capture_output=True, text=True, check=True).stdout.split()
    commits.reverse()
    versions = [(c, rows_at(c, args.cache, args.dir, args.bmb)) for c in commits]
    versions = [(c, v) for c, v in versions if v]
    total = Counter()
    kinds = Counter()
    print("| 커밋 | 정의 | 더함 | 지움 | 본문만 | 계약도 | 읽는 것만 | 그대로 |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- |")
    for (_, a), (c, b) in zip(versions, versions[1:]):
        n, k = classify(a, b)
        if not any(n[x] for x in CLASSES[:5]):
            continue
        total += n
        kinds += k
        print(f"| `{c[:8]}` | {len(b)} | " + " | ".join(str(n[x]) for x in CLASSES) + " |")
    print("| 합 | | " + " | ".join(str(total[x]) for x in CLASSES) + " |")
    edited = total["body"] + total["contract"]
    print()
    print(f"판 {len(versions)}개, 쌍 {len(versions) - 1}개. 고친 정의(자기 글이 바뀐 것) {edited}개 가운데 "
          f"계약이 그대로인 것 {total['body']}개({total['body'] / edited:.1%})." if edited else "고친 정의 없음")
    by = {x: {k[1]: v for k, v in kinds.items() if k[0] == x} for x in ("body", "contract", "reads")}
    print(f"종류별: 본문만 {by['body']}, 계약도 {by['contract']}, 읽는 것만 {by['reads']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
