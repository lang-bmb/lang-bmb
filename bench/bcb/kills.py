#!/usr/bin/env python3
"""Mutant kill rates of the BMB contracts and the Rust tests (backlog 2A-14b,
ADR-0042 §4).

For a BCB tree (the initial plan, or a history replayed to commit K): every
W0904 body mutant of every executable definition (`bmb mutate`) is applied to
both languages at the same place (rustify.py). A mutant is killed on the BMB
side when its definition no longer proves, on the Rust side when its test
(rtests.py) fails. An oracle test with many more inputs (rtests.strong_cases)
decides whether the mutant really breaks the contract on some valid input.

BMB rejects some mutants that keep the contract: a loop that starts at 1 or
steps by 2 breaks the invariant `k <= a.len()` (E0806, E0807) while the
result still meets `ensures`. No test that checks the contract can tell those
apart, so the equality of ADR-0042 §4 ("the mutants the BMB contract kills")
is over `contract` = killed by BMB and by the oracle; `annotation_only` is
reported separately. An oracle kill that BMB proved would be unsound.

Mutants are judged in rounds: round r applies mutant r of every definition at
once (a body mutation leaves every contract unchanged, so definitions do not
affect each other's verdicts): one `bmb verify` per round, one `cargo test`
for all.

  kills.py --defs N [--history H.jsonl --at K] [--work DIR] [--jobs J]
Prints one JSON line per definition (mutant indices: "bmb", "contract",
"annotation_only", "rust", "gaps" = contract kills the Rust test misses,
"unsound" = oracle kills BMB proved, "extra" = Rust kills BMB proved) and a
summary line; exits 1 when gaps, unsound or extra is not empty. Uses $BMB or
target/debug/bmb.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BMB = os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))
sys.path.insert(0, HERE)
import history  # noqa: E402
import rtests  # noqa: E402
import rustify  # noqa: E402


def tree_at(args):
    if not args.history:
        a = history.gen.parse(["-", "--defs", str(args.defs)])
        return history.Tree(a)
    header, commits = history.load(args.history)
    last = None
    for k, tree, _ in history.replay(header, commits, args.at):
        last = tree
    return last


def mutants(tree, ws):
    """def path -> list of mutated BMB blocks, in `bmb mutate` order."""
    paths = [f"m{u}::{n}" for u in range(tree.n_units) for n, _, _ in rtests.exec_defs(tree.units[u])]
    cmd = [BMB, "mutate"] + [x for p in paths for x in ("--def", p)] + [ws]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit("kills: bmb mutate failed\n" + r.stdout[-2000:] + r.stderr[-2000:])
    out = {p: [] for p in paths}
    for line in r.stdout.splitlines():
        row = json.loads(line)
        if row.get("summary"):
            continue
        name = row["def"].split("::", 1)[1]
        block = [b for b in rustify.blocks(row["text"]) if rustify.def_name(b) == name]
        assert len(block) == 1, row["def"]
        out[row["def"]].append(block[0])
    return out


def bmb_round(tree, ws, cache, muts, r):
    """Definitions that fail with mutant r of every definition applied."""
    applied = {p: m[r] for p, m in muts.items() if len(m) > r}
    by_unit = {}
    for p, block in applied.items():
        u = int(p.split("::")[0][1:])
        by_unit.setdefault(u, []).append((p.split("::", 1)[1], block))
    for u, items in by_unit.items():
        text = tree.text[("bmb", u)]
        for name, block in items:
            old = [b for b in rustify.blocks(text) if rustify.def_name(b) == name][0]
            assert text.count(old) == 1
            text = text.replace(old, block)
        open(os.path.join(ws, "src", f"m{u}.bmb"), "w", encoding="utf-8").write(text)
    v = subprocess.run([BMB, "verify", "--cache", cache, ws], capture_output=True, text=True)
    rows = [json.loads(x) for x in v.stdout.splitlines() if x.startswith('{"def"')]
    if not rows:
        sys.exit(f"kills: round {r} does not check\n{v.stdout[-3000:]}")
    for u in by_unit:  # restore
        open(os.path.join(ws, "src", f"m{u}.bmb"), "w", encoding="utf-8").write(tree.text[("bmb", u)])
    failed = {x["def"] for x in rows if x["status"] != "verified"}
    stray = failed - set(applied)
    assert not stray, f"round {r}: unmutated definitions failed: {sorted(stray)}"
    return failed


def rust_kills(tree, rs, muts, jobs):
    """(def, r) pairs whose Rust test fails; also checks that every original passes."""
    for u in range(tree.n_units):
        fns, tests = [], []
        kinds = {n: (k, info) for n, k, info in tree.units[u].defs}
        for name, kind, info in rtests.exec_defs(tree.units[u]):
            for r, block in enumerate(muts[f"m{u}::{name}"]):
                target = f"{name}_mut{r}"
                t = rustify.translate(block)
                assert t.count(f"pub fn {name}(") == 1
                fns.append(t.replace(f"pub fn {name}(", f"pub fn {target}(", 1))
                tests.append((name, kinds[name][0], kinds[name][1], target, False))
                tests.append((name, kinds[name][0], kinds[name][1], target, True))
        text = tree.rust(u, fns, tests)
        open(os.path.join(rs, "src", f"m{u}.rs"), "w", encoding="utf-8").write(text)
    env = dict(os.environ, RUSTFLAGS="-D warnings")
    r = subprocess.run(["cargo", "test", "--offline", "--quiet", "--no-run"], cwd=rs, env=env,
                       capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit("kills: cargo test --no-run failed\n" + r.stderr[-4000:])
    r = subprocess.run(["cargo", "test", "--offline", "--", "--test-threads", str(jobs)],
                       cwd=rs, env=env, capture_output=True, text=True, timeout=1800)
    result = {}
    for line in r.stdout.splitlines():
        m = re.match(r"^test (m\d+)::tests::([ts])_(\w+) \.\.\. (ok|FAILED)$", line)
        if m:
            result[(m.group(1), m.group(2), m.group(3))] = m.group(4) == "ok"
    killed, oracle = set(), set()
    for u in range(tree.n_units):
        for name, _, _ in rtests.exec_defs(tree.units[u]):
            ok = result.get((f"m{u}", "t", name))
            assert ok is True, f"original test m{u}::{name}: {ok}\n{r.stdout[-3000:]}"
            for i in range(len(muts[f"m{u}::{name}"])):
                for kind, out in (("t", killed), ("s", oracle)):
                    passed = result.get((f"m{u}", kind, f"{name}_mut{i}"))
                    assert passed is not None, f"no result for m{u}::{kind}_{name}_mut{i}"
                    if not passed:
                        out.add((f"m{u}::{name}", i))
    return killed, oracle


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--defs", type=int, default=100)
    p.add_argument("--history")
    p.add_argument("--at", type=int)
    p.add_argument("--work")
    p.add_argument("--jobs", type=int, default=4)
    args = p.parse_args(argv)
    work = args.work or tempfile.mkdtemp(prefix="bcb-kills-")
    try:
        tree = tree_at(args)
        ws, rs, cache = (os.path.join(work, x) for x in ("bmb", "rust", "cache"))
        history.write_tree(tree, ws, rs)
        bad = [(u, rustify.check_unit(tree.text[("bmb", u)], tree.text[("rust", u)])) for u in range(tree.n_units)]
        bad = [x for x in bad if x[1]]
        assert not bad, f"translation differs from the twin: {bad}"
        base = subprocess.run([BMB, "verify", "--cache", cache, ws], capture_output=True, text=True)
        assert base.returncode == 0, base.stdout[-2000:]
        muts = mutants(tree, ws)
        bmb_killed = set()
        for r in range(max((len(m) for m in muts.values()), default=0)):
            bmb_killed |= {(d, r) for d in bmb_round(tree, ws, cache, muts, r)}
        rust_killed, oracle = rust_kills(tree, rs, muts, args.jobs)
        keys = ("bmb", "contract", "annotation_only", "rust", "gaps", "unsound", "extra")
        totals = {"defs": 0, "mutants": 0, **{k: 0 for k in keys}}
        for u in range(tree.n_units):
            for name, kind, _ in rtests.exec_defs(tree.units[u]):
                d = f"m{u}::{name}"
                b = {r for x, r in bmb_killed if x == d}
                o = {r for x, r in oracle if x == d}
                t = {r for x, r in rust_killed if x == d}
                row = {"def": d, "kind": kind, "mutants": len(muts[d]),
                       "bmb": b, "contract": b & o, "annotation_only": b - o, "rust": t,
                       "gaps": (b & o) - t, "unsound": o - b, "extra": t - b}
                print(json.dumps({k: sorted(v) if isinstance(v, set) else v for k, v in row.items()}))
                totals["defs"] += 1
                totals["mutants"] += row["mutants"]
                for k in keys:
                    totals[k] += len(row[k])
        print(json.dumps({"summary": True, **totals}))
        return 1 if totals["gaps"] or totals["unsound"] or totals["extra"] else 0
    finally:
        if not args.work:
            shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
