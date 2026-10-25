#!/usr/bin/env python3
"""Gate test for BNB (backlog 3A-05): frozen tasks, the Rust twin (cargo check,
locality), the file tools, and at N=100 the same ground truth from the BMB
maps and from the twin's text, with both scripted oracles answering every task
exactly within the budget. Uses $BMB or target/debug/bmb."""

import filecmp
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import bnb  # noqa: E402


def twin(out, n):
    subprocess.run([sys.executable, os.path.join(HERE, "twin.py"), out, "--defs", str(n)], check=True,
                   capture_output=True)


def test_tasks_frozen():
    bnb.check_manifest()
    assert bnb.make_tasks() == bnb.load_tasks(), "tasks/ is not what `bnb.py tasks` derives"
    assert 20 <= len(bnb.load_tasks()) <= 30


def test_twin_local():
    with tempfile.TemporaryDirectory() as t:
        a, b, c = (os.path.join(t, x) for x in "abc")
        twin(a, 100)
        twin(b, 100)
        twin(c, 200)
        for u in range(5):
            f = f"src/m{u}.rs"
            assert filecmp.cmp(os.path.join(a, f), os.path.join(b, f), shallow=False), f"not deterministic: {f}"
            assert filecmp.cmp(os.path.join(a, f), os.path.join(c, f), shallow=False), f"not local: {f}"


def test_rust_tools():
    with tempfile.TemporaryDirectory() as t:
        os.makedirs(os.path.join(t, "src"))
        with open(os.path.join(t, "src", "a.rs"), "w") as f:
            f.write("one\ntwo\nthree\nfour\nfive\n")
        rt = bnb.RustTools(t)
        assert rt.call("list_dir", {}) == ("src/\n", False)
        assert rt.call("read_file", {"path": "src/a.rs", "start_line": 2, "end_line": 3}) == ("2\ttwo\n3\tthree\n", False)
        out, _ = rt.call("grep", {"pattern": "^t", "context_before": 1})
        assert out == "src/a.rs-1-one\nsrc/a.rs:2:two\nsrc/a.rs:3:three\n", out
        out, _ = rt.call("grep", {"pattern": "five|one", "path": ["src/a.rs"], "context_after": 1})
        assert out == "src/a.rs:1:one\nsrc/a.rs-2-two\n--\nsrc/a.rs:5:five\n", out
        assert rt.call("grep", {"pattern": "zzz"}) == ("(no matches)\n", False)
        assert rt.call("read_file", {"path": "../x"})[1], "path outside the crate must be an error"


def test_oracles_n100():
    with tempfile.TemporaryDirectory() as t:
        out = os.path.join(t, "r.jsonl")
        p = subprocess.run([sys.executable, os.path.join(HERE, "bnb.py"), "run", "--sizes", "100", "--work", t,
                            "--check-truth", "--cargo-check", "--expect", "all-pass", "--out", out],
                           capture_output=True, text=True)
        assert p.returncode == 0, p.stdout[-3000:] + p.stderr[-3000:]
        rows = [json.loads(x) for x in open(out)]
        build = next(r for r in rows if r.get("kind") == "build")
        assert build["cargo_check_exit"] == 0 and build["truth_diffs"] == [], build
        runs = [r for r in rows if "arm" in r]
        n = len(bnb.load_tasks())
        for arm in ("bmb", "rust"):
            xs = [r for r in runs if r["arm"] == arm]
            assert len(xs) == n and all(r["success"] and not r["limit_hit"] for r in xs), arm


def main():
    test_tasks_frozen()
    test_twin_local()
    test_rust_tools()
    test_oracles_n100()
    print("bench/bnb: ok")


if __name__ == "__main__":
    main()
