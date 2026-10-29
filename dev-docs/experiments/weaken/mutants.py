#!/usr/bin/env python3
"""Alignment sweep for `bmb weaken` (backlog 2D-19): body mutants of real
programs must compare `same` everywhere.

  python3 dev-docs/experiments/weaken/mutants.py [--bmb PATH] [--per-file N] FILE.bmb...

For each program: import it as a workspace, and for its first N definitions
that `bmb mutate` accepts, apply the middle mutant (one body change, no
contract change) and run `bmb weaken original mutant`. The mutated
definition's `id` changes and so do its callers' (content addressing), so
both versions go into the aligned program and are compared by the solver.
Every row must be `same`; anything else (a non-`same` verdict, an alignment
failure, a diagnostic) is printed. A mutant that does not check (some
literal mutants leave their type's range) is skipped and counted.
Standard library only; deterministic.
"""
import argparse
import json
import os
import shutil
import subprocess
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    ap.add_argument("--per-file", type=int, default=3)
    ap.add_argument("files", nargs="+")
    args = ap.parse_args()
    work = tempfile.mkdtemp(prefix="weaken-mutants-")
    stats, bad, skipped, compared, queries = {}, [], 0, 0, 0
    for f in args.files:
        ws = os.path.join(work, "a")
        shutil.rmtree(ws, ignore_errors=True)
        if subprocess.run([args.bmb, "import", ws, f], capture_output=True).returncode != 0:
            continue
        h = subprocess.run([args.bmb, "hash", ws], capture_output=True, text=True)
        defs = [json.loads(x)["def"] for x in h.stdout.splitlines() if x.strip()]
        tried = 0
        for d in defs:
            if tried >= args.per_file:
                break
            m = subprocess.run([args.bmb, "mutate", "--def", d, ws], capture_output=True, text=True)
            if m.returncode != 0:
                continue  # not an executable function
            rows = [json.loads(x) for x in m.stdout.splitlines() if x.strip()]
            rows = [x for x in rows if "text" in x]
            if not rows:
                continue
            tried += 1
            mu = rows[len(rows) // 2]
            wb = os.path.join(work, "b")
            shutil.rmtree(wb, ignore_errors=True)
            shutil.copytree(ws, wb)
            with open(os.path.join(wb, "src", mu["unit"] + ".bmb"), "w", encoding="utf-8") as o:
                o.write(mu["text"])
            w = subprocess.run([args.bmb, "weaken", ws, wb], capture_output=True, text=True)
            out = [json.loads(x) for x in w.stdout.splitlines() if x.strip()]
            if any(x.get("severity") == "error" for x in out):
                skipped += 1
                continue
            compared += 1
            for x in out:
                if "verdict" in x:
                    stats[x["verdict"]] = stats.get(x["verdict"], 0) + 1
                    if x["verdict"] != "same":
                        bad.append((f, d, x))
                elif x.get("summary"):
                    queries += x["queries"]
    shutil.rmtree(work, ignore_errors=True)
    print(json.dumps({"files": len(args.files), "mutants": compared, "mutants_not_checking": skipped,
                      "rows": stats, "queries": queries}))
    for b in bad:
        print(json.dumps(b, ensure_ascii=False))


if __name__ == "__main__":
    main()
