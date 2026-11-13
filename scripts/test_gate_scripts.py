#!/usr/bin/env python3
"""The gate's own scripts (INF-34, INF-35, INF-36): scripts/fresh.sh fails on
a stale generated document within seconds and names it, the gate lock lets
one holder run at a time, and scripts/tree_state.py sees what a run left."""
import fcntl
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import tree_state  # noqa: E402


def copy_of_tree(dst):
    """The files of the tree as they are now (not the ignored ones), with this tree's build."""
    files = subprocess.run(["git", "ls-files", "-z", "-co", "--exclude-standard"], cwd=ROOT, capture_output=True, text=True,
                           check=True).stdout.split("\0")
    for rel in filter(None, files):
        src = ROOT / rel
        if not src.is_file():  # deleted and not yet committed
            continue
        out = dst / rel
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_bytes(src.read_bytes())
        out.chmod(src.stat().st_mode)
    (dst / "target").symlink_to(ROOT / "target")


def fresh(root):
    env = dict(os.environ, BMB_FRESH_NO_BUILD="1")
    start = time.monotonic()
    r = subprocess.run([str(root / "scripts" / "fresh.sh")], capture_output=True, text=True, env=env)
    return r, time.monotonic() - start


def test_fresh_names_each_stale_document():
    with tempfile.TemporaryDirectory() as t:
        root = Path(t)
        copy_of_tree(root)
        r, _ = fresh(root)
        assert r.returncode == 0, r.stderr
        # one stale document, one stale generated block, one health ratchet
        with open(root / "docs" / "diagnostics.md", "a") as f:
            f.write("\nstale\n")
        text = (root / "docs" / "ROADMAP.md").read_text()
        mark = "<!-- roadmap:status begin"
        assert mark in text
        i = text.index("\n", text.index(mark)) + 1
        (root / "docs" / "ROADMAP.md").write_text(text[:i] + "stale\n" + text[i:])
        (root / "compiler" / "tests" / "bound.rs").write_text("use bmb::map;\n")
        r, took = fresh(root)
        assert r.returncode == 1, r.stdout + r.stderr
        for name in ("diagnostic codes document", "roadmap status block", "health (quick)"):
            assert f"fresh: {name}" in r.stderr, r.stderr
        assert "S5_tests_bound_to_rust_api" in r.stderr, r.stderr
        assert "fresh: backlog" not in r.stderr, r.stderr
        assert took < 120, took


def test_one_gate_at_a_time():
    with tempfile.TemporaryDirectory() as t:
        lock, out = os.path.join(t, "gate.lock"), os.path.join(t, "ran")
        env = dict(os.environ, BMB_GATE_LOCK=lock)
        env.pop("BMB_GATE_LOCK_HELD", None)
        cmd = [str(ROOT / "scripts" / "with_gate_lock.sh"), "sh", "-c", f'echo "$BMB_GATE_LOCK_HELD" > {out}']
        with open(lock, "w") as held:
            fcntl.flock(held, fcntl.LOCK_EX)
            waiting = subprocess.Popen(cmd, env=env, stderr=subprocess.PIPE, text=True)
            time.sleep(1.0)
            assert waiting.poll() is None and not os.path.exists(out), "ran under a held lock"
            # a gate started by the holder (gate_power.py) does not wait for itself
            inner = os.path.join(t, "inner")
            subprocess.run([cmd[0], "sh", "-c", f"echo in > {inner}"], env=dict(env, BMB_GATE_LOCK_HELD="1"),
                           check=True, timeout=10)
            assert os.path.exists(inner)
        assert waiting.wait(timeout=10) == 0
        assert "waiting" in waiting.stderr.read()
        assert open(out).read().strip() == "1"


def test_a_run_that_leaves_a_file_shows():
    with tempfile.TemporaryDirectory() as t:
        root = Path(t)
        git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=root,
                                        check=True, capture_output=True)
        git("init", "-q")
        (root / ".gitignore").write_text("target/\n")
        (root / "kept.txt").write_text("a\n")
        (root / "edited.txt").write_text("a\n")
        git("add", ".")
        git("commit", "-qm", "c")
        (root / "edited.txt").write_text("already uncommitted\n")
        (root / "mine.txt").write_text("already there\n")
        before = tree_state.snapshot(root)
        assert tree_state.left(before, root) == []
        (root / "target").mkdir()
        (root / "target" / "x").write_text("ignored\n")
        assert tree_state.left(before, root) == []
        (root / "x.bmb").write_text("fn main() {}\n")
        (root / "kept.txt").write_text("changed by a test\n")
        assert tree_state.left(before, root) == [" M kept.txt", "?? x.bmb"]


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_"):
            fn()
    print("ok")
