#!/usr/bin/env python3
"""Gate test for BCB (backlog 2A-14, ADR-0042): histories are deterministic,
replay reproduces every recorded digest, every commit of every profile keeps
the BMB workspace fully proved and the Rust twin compiling (`cargo check`,
warnings as errors), and each edit kind changes what it claims to (a body edit
keeps every interface hash, a contract edit changes one).

Small sizes (N=100, K=COMMITS); uses $BMB or target/debug/bmb.
"""

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
        r = subprocess.run(["cargo", "check", "--quiet", "--offline"], cwd=rs, env=env, capture_output=True, text=True)
        assert r.returncode == 0, f"{profile} commit {k}: cargo check\n{r.stderr[-3000:]}"
        now = hashes(ws)
        heads = headers(tree)
        assert set(heads) == set(now), f"commit {k}: header parse"
        if before is not None:
            ops = {e["op"] for e in commits[k - 1]["edits"]}
            if ops == {"body"}:
                # Signature and contract stay; the interface hash changes only with `#[uses]`
                # (ADR-0019 puts attributes in it; backlog 2D-22).
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
    print("bench/bcb: ok " + json.dumps(out))


if __name__ == "__main__":
    main()
