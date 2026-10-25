#!/usr/bin/env python3
"""Edit localization (backlog 1B-41, H-MAINT, ADR-0040 §4).

  localize.py HISTORY.jsonl --work DIR [--upto K] [--out FILE.jsonl]

Replays a BCB commit history (history.py) on the BMB workspace and, at every
commit, checks that the definitions the incremental build encoded again
(`bmb verify --cache --stats`) lie inside the union of `bmb query impact` over
the definitions the commit edited: a body edit as `body`, a contract or spec
edit as `iface`; added definitions count as changed. H-MAINT is falsified
when a re-proved definition falls outside that set. Each commit's re-proof
cost is the `rlimit` sum of the encoded definitions. Prints one summary line
(and writes one row per commit with --out); exits 1 on a definition outside
the impact set. Not part of the frozen BCB harness (MANIFEST): it reads the
histories, it does not measure the arms. Standard library only.
"""
import argparse
import json
import os
import statistics
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import history  # noqa: E402

BMB = os.environ.get("BMB", os.path.join(ROOT, "target", "release", "bmb"))


def jsonl(text):
    return [json.loads(x) for x in text.splitlines() if x.startswith("{")]


def def_paths(ws):
    out = subprocess.run([BMB, "hash", os.path.join(ws, "src")], capture_output=True, text=True).stdout
    return {r["def"] for r in jsonl(out)}


def impact(ws, d, change):
    out = subprocess.run([BMB, "query", "impact", d, change, "--", ws], capture_output=True, text=True)
    rows = jsonl(out.stdout)
    if out.returncode != 0 or not rows:
        sys.exit(f"localize: impact {d} {change} failed: {out.stderr[-500:]}")
    return {r["path"] for r in rows if "path" in r}


def edited(commit, before, now):
    """(definition, change) per edit of the commit; added definitions are
    `body` changes of themselves (nothing referred to them before)."""
    out = set()
    for e in commit["edits"]:
        u = e["unit"]
        if e["op"] == "body":
            out.add((f"m{u}::{e['def']}", "body"))
        elif e["op"] in ("contract", "spec"):
            out.add((f"m{u}::{e['def']}", "iface"))
    for d in now - before:
        out.add((d, "body"))
    return {(d, c) for d, c in out if d in now}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("history")
    ap.add_argument("--work", required=True)
    ap.add_argument("--upto", type=int)
    ap.add_argument("--out")
    args = ap.parse_args(argv)
    header, commits = history.load(args.history)
    ws, rust, cache, stats = (os.path.join(args.work, x) for x in ("bmb", "rust", "cache", "stats.json"))
    by_k = {c["k"]: c for c in commits}
    rows, outside, before = [], [], set()
    for k, tree, changed in history.replay(header, commits, args.upto):
        history.write_tree(tree, ws, rust, None if k == 0 else changed)
        r = subprocess.run([BMB, "verify", "--cache", cache, "--stats", stats, ws], capture_output=True, text=True)
        status = {x["def"]: x for x in jsonl(r.stdout) if "def" in x}
        if r.returncode != 0:
            sys.exit(f"localize: commit {k} does not verify")
        encoded = set(json.load(open(stats))["encoded"])
        now = def_paths(ws)
        if k > 0:
            allowed = set()
            for d, change in sorted(edited(by_k[k], before, now)):
                allowed |= impact(ws, d, change)
            out = sorted(encoded - allowed)
            if out:
                outside.append({"k": k, "outside": out})
            cost = sum(status[d]["rlimit"] for d in encoded if d in status)
            rows.append({"k": k, "encoded": len(encoded), "impact": len(allowed), "outside": out, "rlimit": cost})
        before = now
    if args.out:
        with open(args.out, "w") as f:
            for row in rows:
                f.write(json.dumps(row) + "\n")
    costs = sorted(r["rlimit"] for r in rows)

    def q(p):
        return costs[min(len(costs) - 1, int(p * len(costs)))] if costs else 0
    print(json.dumps({"history": os.path.basename(args.history), "commits": len(rows),
                      "encoded_mean": round(statistics.mean(r["encoded"] for r in rows), 2) if rows else 0,
                      "impact_mean": round(statistics.mean(r["impact"] for r in rows), 2) if rows else 0,
                      "outside_commits": len(outside),
                      "rlimit_median": q(0.5), "rlimit_p90": q(0.9), "rlimit_max": costs[-1] if costs else 0}))
    if outside:
        sys.exit(f"localize: re-proved outside the impact set: {outside[:5]}")


if __name__ == "__main__":
    main()
