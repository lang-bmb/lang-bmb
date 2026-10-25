#!/usr/bin/env python3
"""Gate test for BCB (backlog 2A-14, ADR-0042): histories are deterministic,
replay reproduces every recorded digest, every commit of every profile keeps
the BMB workspace fully proved and the Rust twin with its tests compiling
(`cargo check --tests`, warnings as errors), each edit kind changes what it
claims to (a body edit keeps signature and contract, a contract edit changes an
interface), and the Rust tests kill every mutant whose contract violation the
BMB gate rejects (kills.py).

Small sizes (N=100, K=COMMITS); uses $BMB or target/debug/bmb.
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BMB = os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))
sys.path.insert(0, HERE)
import history  # noqa: E402
import kills  # noqa: E402
import bcb  # noqa: E402

bcb.BMB = kills.BMB = BMB

COMMITS = 40


def gen(out, profile, commits=COMMITS):
    history.main(["gen", out, "--defs", "100", "--commits", str(commits), "--profile", profile])


def hashes(ws):
    r = subprocess.run([BMB, "hash", ws], capture_output=True, text=True, check=True)
    rows = [json.loads(x) for x in r.stdout.splitlines() if x.startswith("{")]
    return {x["def"]: (x["body"], x["iface"]) for x in rows}


def headers(tree):
    """Per definition: its header in the BMB text without `#[uses]` (effects,
    signature, requires, ensures), and its `#[uses]` line."""
    out = {}
    for (lang, u), text in tree.text.items():
        if lang != "bmb":
            continue
        for block in text.split("\n\n"):
            lines = [x.strip() for x in block.splitlines()]
            sig = next((i for i, x in enumerate(lines) if x.startswith(("fn ", "spec fn "))), None)
            if sig is None:
                continue
            name = lines[sig].split()[1 if lines[sig].startswith("fn ") else 2].split("(")[0]
            end = next(i for i, x in enumerate(lines) if i >= sig and x.endswith("{"))
            head = [x for x in lines[:end + 1] if not x.startswith("#[uses(")]
            uses = [x for x in lines[:sig] if x.startswith("#[uses(")]
            out[f"m{u}::{name}"] = ("\n".join(head), uses)
    return out


def verify(ws, cache):
    r = subprocess.run([BMB, "verify", "--cache", cache, ws], capture_output=True, text=True)
    rows = [json.loads(x) for x in r.stdout.splitlines() if x.startswith('{"def"')]
    bad = [x for x in rows if x["status"] != "verified"]
    assert r.returncode == 0 and not bad and rows, r.stdout[-3000:]
    return len(rows)


def check_profile(t, profile):
    path = os.path.join(t, f"{profile}.jsonl")
    gen(path, profile)
    again = os.path.join(t, f"{profile}-again.jsonl")
    gen(again, profile)
    assert open(path, "rb").read() == open(again, "rb").read(), f"{profile}: history not deterministic"
    header, commits = history.load(path)
    assert len(commits) == COMMITS and header["events"]["four"] == [8, 16, 24, 32], header["events"]
    ws, rs, cache = (os.path.join(t, f"{profile}-{x}") for x in ("bmb", "rust", "cache"))
    before = None
    body_edits = uses_changed = 0
    env = dict(os.environ, RUSTFLAGS="-D warnings", CARGO_TARGET_DIR=os.path.join(t, f"{profile}-target"))
    for k, tree, changed in history.replay(header, commits):
        history.write_tree(tree, ws, rs, None if k == 0 else changed)
        verify(ws, cache)
        r = subprocess.run(["cargo", "check", "--quiet", "--offline", "--tests"], cwd=rs, env=env,
                           capture_output=True, text=True)
        assert r.returncode == 0, f"{profile} commit {k}: cargo check\n{r.stderr[-3000:]}"
        now = hashes(ws)
        heads = headers(tree)
        assert set(heads) == set(now), f"commit {k}: header parse"
        if before is not None:
            ops = {e["op"] for e in commits[k - 1]["edits"]}
            if ops == {"body"}:
                # Signature and contract stay; the interface hash changes only with `#[uses]`
                # (ADR-0019 puts attributes in it; the incremental keys leave it out, ADR-0043).
                for d in now:
                    assert heads[d][0] == before_heads[d][0], f"commit {k}: a body edit changed {d}'s header"
                    if heads[d][1] == before_heads[d][1]:
                        assert now[d][1] == before[d][1], f"commit {k}: {d}'s interface changed"
                for e in commits[k - 1]["edits"]:
                    d = f"m{e['unit']}::{e['def']}"
                    body_edits += 1
                    uses_changed += heads[d][1] != before_heads[d][1]
            if ops == {"contract"}:
                assert any(now[d][1] != before[d][1] for d in now if d in before), f"commit {k}: no interface changed"
            if "add" not in ops and "remove" not in ops and "contract" not in ops:
                assert set(now) == set(before), f"commit {k}: definitions appeared or vanished"
        before, before_heads = now, heads
    # The written tree equals a fresh replay to the end.
    fresh_b, fresh_r = os.path.join(t, f"{profile}-fb"), os.path.join(t, f"{profile}-fr")
    history.main(["replay", path, "--bmb", fresh_b, "--rust", fresh_r])
    for a, b in ((ws, fresh_b), (rs, fresh_r)):
        d = subprocess.run(["diff", "-r", os.path.join(a, "src"), os.path.join(b, "src")], capture_output=True, text=True)
        assert d.returncode == 0, d.stdout[-2000:]
    return history.stats(header, commits), body_edits, uses_changed


def check_harness(t):
    """Replay harness (2A-14c): every arm measures every commit it should, the
    selections nest (R2 within R1 within R0), events add cold rows, B0 proves
    everything, and the table reads the result."""
    path = os.path.join(t, "harness.jsonl")
    history.main(["gen", path, "--defs", "100", "--commits", "5", "--profile", "body-heavy"])
    out = os.path.join(t, "harness-results.jsonl")
    bcb.main(["run", path, "--work", os.path.join(t, "harness"), "--out", out, "--b0-every", "5"])
    head, rows = bcb.load_rows(out)
    assert head["events"]["four"] == [1, 2, 3, 4], head
    by = {}
    for r in rows:
        by.setdefault((r["arm"], r["cold"]), []).append(r["k"])
    assert by[("rust", False)] == list(range(6)) and by[("b1", False)] == list(range(6)), by
    assert by[("rust", True)] == [1, 2, 3, 4] and by[("b1", True)] == [1, 2, 3, 4], by
    assert by[("b0", True)] == [0, 5], by
    for r in rows:
        if r["arm"] == "rust":
            assert r["R2"]["tests"] <= r["R1"]["tests"] <= r["R0"]["tests"], r
            assert r["R2"]["covered_defs"] >= r["R2"]["tests"], r
        if r["arm"] == "b1" and r["k"] > 0 and not r["cold"]:
            assert len(r["encoded_defs"]) < 20, r  # a few definitions, not the program
    cold = [r for r in rows if r["arm"] == "b1" and r["cold"]]
    warm = {r["k"]: r for r in rows if r["arm"] == "b1" and not r["cold"]}
    assert all(r["encoded"] > warm[r["k"]]["encoded"] for r in cold), "a cold build re-proves everything"
    b0 = [r for r in rows if r["arm"] == "b0"]
    assert b0[0]["encoded"] == [r for r in rows if r["arm"] == "b1" and r["k"] == 0][0]["encoded"], b0
    # Planted defects (2A-14d): a Rust catch is always a proof failure too (Rust
    # tests check only the contract), the selections nest, B0 sees what B1 sees.
    dpath = os.path.join(t, "defects.jsonl")
    bcb.main(["defects", path, "--work", os.path.join(t, "defects"), "--out", dpath])
    _, drows = bcb.load_rows(dpath)
    planted = [r for r in drows if "def" in r]
    assert len(drows) == 5 and planted, drows
    for r in planted:
        assert r["R2"] <= r["R1"] <= r["R0"], r
        assert not r["R0"] or r["proof"], f"Rust caught what BMB proved: {r}"
        assert r["B1"] == (r["proof"] or r["B1_tests"]) and r["B1"] <= r["B0"], r
    text = bcb.table([out, dpath])
    assert "C_B1/C_R1" in text and "| body-heavy | 100 | 5 | B1 |" in text and "결함 검출(전체)" in text, text


def selections(path):
    out = []
    for line in open(path):
        r = json.loads(line)
        if r.get("header"):
            continue
        if r.get("arm") == "rust":
            out.append((r["k"], r["cold"], r["R1"]["tests"], r["R2"]["tests"], r["R2"]["covered_defs"]))
        elif r.get("arm") == "b1":
            out.append((r["k"], r["cold"], tuple(r["encoded_defs"]), r["tests"]["tests"]))
        else:
            out.append(tuple(sorted((k, str(v)) for k, v in r.items())))
    return out


class Stop(Exception):
    pass


def check_suite(t):
    """`suite` continues runs that were killed: rust between an event commit's
    warm and cold rows, b1 the same, defects mid-history, each with a line cut
    off; the result rows equal one uninterrupted run."""
    for w in ("w1", "w2"):
        hdir = os.path.join(t, "suite", w, "histories")
        os.makedirs(hdir)
        history.main(["gen", os.path.join(hdir, "mini.jsonl"), "--defs", "100", "--commits", "6",
                      "--profile", "body-heavy"])
    saved = bcb.SUITE, bcb.manifest, bcb.measure, bcb.bmb_measure, bcb.tests_fail
    bcb.manifest = lambda args: 0
    os.environ["BCB_ALLOW_DIRTY"] = "1"  # the gate's tree may be a working copy
    one, two = os.path.join(t, "suite", "one"), os.path.join(t, "suite", "two")
    try:
        bcb.SUITE = [("mini", "rust", 0), ("mini", "b1", 0), ("mini", "defects", 0)]
        bcb.suite(argparse.Namespace(work=os.path.join(t, "suite", "w1"), results=one))
        cleans, fails = [0], [0]

        def measure(cmd, *a, **kw):
            if cmd[:2] == ["cargo", "clean"]:
                cleans[0] += 1
                if cleans[0] == 2:  # the cold half of event commit 2
                    raise Stop()
            return saved[2](cmd, *a, **kw)

        def bmb_measure(tree, ws, cache, selected, k, arm, cold, out, n):
            if k == 3 and cold:
                raise Stop()
            return saved[3](tree, ws, cache, selected, k, arm, cold, out, n)

        def tests_fail(*a):
            fails[0] += 1
            if fails[0] == 7:  # third planted commit
                raise Stop()
            return saved[4](*a)

        bcb.measure, bcb.bmb_measure, bcb.tests_fail = measure, bmb_measure, tests_fail
        for arm in ("rust", "b1", "defects"):
            bcb.SUITE = [("mini", arm, 0)]
            try:
                bcb.suite(argparse.Namespace(work=os.path.join(t, "suite", "w2"), results=two))
                raise AssertionError(f"{arm}: the stop did not happen")
            except Stop:
                pass
            with open(os.path.join(two, f"mini.{arm}.jsonl"), "a") as f:
                f.write('{"arm": "cut off')
        bcb.measure, bcb.bmb_measure, bcb.tests_fail = saved[2:]
        bcb.SUITE = [("mini", "rust", 0), ("mini", "b1", 0), ("mini", "defects", 0)]
        bcb.suite(argparse.Namespace(work=os.path.join(t, "suite", "w2"), results=two))
        for arm in ("rust", "b1", "defects"):
            a, b = (selections(os.path.join(d, f"mini.{arm}.jsonl")) for d in (one, two))
            assert a == b, f"suite resume differs for {arm}:\n{a}\n{b}"
        assert bcb.resume_point(os.path.join(two, "mini.rust.jsonl"), "rust", {1, 2, 3, 4}, 6) is None
    finally:
        bcb.SUITE, bcb.manifest, bcb.measure, bcb.bmb_measure, bcb.tests_fail = saved
        bcb.BMB = BMB


def main():
    with tempfile.TemporaryDirectory() as t:
        out = {}
        for profile in sorted(history.PROFILES):
            st, body, uses = check_profile(t, profile)
            out[profile] = st["edits"]
            out[profile + " body edits changing #[uses]"] = f"{uses}/{body}"
        assert out["body-only"]["body"] > 0 and sum(out["body-only"].values()) == out["body-only"]["body"], out
        assert all(out["contract-heavy"][k] > 0 for k in history.KINDS), out
        # A tampered history is refused.
        path = os.path.join(t, "body-heavy.jsonl")
        lines = open(path, encoding="utf-8").read().splitlines()
        c = json.loads(lines[3])
        c["digest"] = "0" * 64
        lines[3] = json.dumps(c)
        open(path, "w", encoding="utf-8").write("\n".join(lines) + "\n")
        r = subprocess.run([sys.executable, os.path.join(HERE, "history.py"), "replay", path, "--bmb",
                            os.path.join(t, "x"), "--rust", os.path.join(t, "y")], capture_output=True, text=True)
        assert r.returncode != 0 and "differs from the recorded digest" in r.stderr, r.stderr
        # Test equality (2A-14b): after the contract-heavy history, the Rust tests kill
        # every mutant whose contract violation the BMB gate rejects, and nothing it proves.
        code = kills.main(["--history", os.path.join(t, "contract-heavy.jsonl"), "--at", str(COMMITS),
                           "--work", os.path.join(t, "kills")])
        assert code == 0, "kills: gaps, unsound or extra mutants"
        check_harness(t)
        check_suite(t)
        assert bcb.manifest(argparse.Namespace(write=False, histories=None)) == 0, "bench/bcb/MANIFEST is stale"
    print("bench/bcb: ok " + json.dumps(out))


if __name__ == "__main__":
    main()
