#!/usr/bin/env python3
"""T2-10: totals of measure.sh runs. usage: summarize.py OUT [fail]"""
import json, sys, collections
d = sys.argv[1]
for chk in (["vacfail", "prefail"] if len(sys.argv) > 2 else ["vac", "pre"]):
    try: rows = [json.loads(l) for l in open(f"{d}/{chk}.log")]
    except FileNotFoundError: continue
    tot = sum(r["rlimit"] for r in rows)
    by = collections.Counter((r["limit"], r["answer"]) for r in rows)
    cost = collections.Counter()
    for r in rows: cost[(r["limit"], r["answer"])] += r["rlimit"]
    out = open(f"{d}/{chk}.out").read().splitlines()
    if chk.startswith("vac"):
        und = by[(1000000, "unknown")]
        refuted = sum(1 for l in out if l.startswith("VAC"))
        print(chk, "queries", len(rows), "rlimit", f"{tot:,}", "undecided points", und, "refuted", refuted)
    else:
        none = sum(1 for l in out if l.startswith("PRE") and l.endswith("None"))
        unsat = sum(1 for l in out if l.startswith("PRE") and l.endswith("Some(false)"))
        print(chk, "queries", len(rows), "rlimit", f"{tot:,}", "undecided", none, "unsat", unsat)
    for k in sorted(by, key=lambda k: (k[0] or 0, k[1])): print("   ", k, by[k], f"{cost[k]:,}")
