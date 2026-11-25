#!/usr/bin/env python3
"""The gate's own scripts: scripts/fresh.sh fails on
a stale generated document within seconds and names it, the gate lock lets
one holder run at a time, scripts/tree_state.py sees what a run left, and
the nightly workflow runs every case of the port's nightly set."""
import fcntl
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import difftest  # noqa: E402
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
        # one stale document
        with open(root / "docs" / "diagnostics.md", "a") as f:
            f.write("\nstale\n")
        r, took = fresh(root)
        assert r.returncode == 1, r.stdout + r.stderr
        assert "fresh: diagnostic codes document" in r.stderr, r.stderr
        assert "fresh: language reference index" not in r.stderr, r.stderr
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


def test_the_nightly_workflow_runs_every_nightly_case():
    """.github/workflows/nightly.yml runs tests/port/nightly/cases.json every
    night as one job per part, and the parts hold every case once: the set
    cannot drop out of CI by an edit of the workflow or of the cases."""
    text = (ROOT / ".github" / "workflows" / "nightly.yml").read_text()
    assert re.search(r"(?m)^on:\n(?:  .*\n)*  schedule:\n    - cron: \"[^\"]+\"$", text), "nightly.yml runs on a schedule"
    jobs = re.split(r"(?m)^  (?=[a-z][a-z-]*:\n)", text.split("\njobs:\n", 1)[1])
    port = [j for j in jobs if re.search(r"(?m)^        run: scripts/port_nightly\.sh ", j)]
    assert len(port) == 1, "one job runs the nightly cases"
    job = port[0]
    parts = re.findall(r"(?m)^        part: \[([0-9, ]+)\]$", job)
    assert len(parts) == 1, "the job has one matrix of parts"
    numbers = [int(x) for x in parts[0].split(",")]
    runs = re.findall(r"(?m)^        run: scripts/port_nightly\.sh (.*)$", job)
    assert runs == [f"--shard ${{{{ matrix.part }}}}/{len(numbers)}"], runs
    assert numbers == list(range(1, len(numbers) + 1)), numbers
    n = len(numbers)
    assert re.search(r"(?m)^    timeout-minutes: \d+$", job), "the job has a time limit"
    assert "if: failure()" in job and "upload-artifact" in job, "a failing part keeps its differences"
    # The script runs the whole set, narrowed only by its arguments.
    sh = (ROOT / "scripts" / "port_nightly.sh").read_text().replace("\\\n", " ")
    runs = re.findall(r"(?m)^python3 scripts/difftest\.py run (\S+) .*$", sh)
    assert runs == ["tests/port/nightly/cases.json"], runs
    assert '"$@"' in re.search(r"(?m)^python3 scripts/difftest\.py run .*$", sh).group(0)
    assert "difftest.py build tests/port/nightly/cases.json target/nightly-build" in sh
    path = ROOT / "tests" / "port" / "nightly" / "cases.json"
    cases = json.loads(path.read_text())["cases"]
    names = [c["name"] for c in cases]
    assert len(set(names)) == len(names), "case names are unique"
    parts, _ = difftest.shards(cases, difftest.load_costs(str(path)), n)
    held = sorted(cases[i]["name"] for p in parts for i in p)
    assert held == sorted(names), "every case is in exactly one part"
    assert all(parts), "every part has a case"
    stale = sorted(set(difftest.load_costs(str(path))) - set(names))
    assert stale == [], f"costs.json names cases that are gone: {stale}"
    # The per-push gate does not run the nightly set.
    assert "nightly" not in (ROOT / "scripts" / "check.sh").read_text()


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_"):
            fn()
    print("ok")
