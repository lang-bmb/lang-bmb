#!/usr/bin/env python3
"""Session records (backlog INF-17): what a record holds, and that the result
rows are the same with or without it."""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import sessionlog  # noqa: E402

TASK = "w01-clamp"


def run(out, sessions=None):
    argv = [sys.executable, os.path.join(HERE, "harness.py"), "run", "--agent", "oracle", "--tasks", TASK,
            "--out", out] + (["--sessions", sessions] if sessions else [])
    subprocess.run(argv, cwd=ROOT, check=True, capture_output=True)
    with open(out) as f:
        return [json.loads(line) for line in f if line.strip()]


class Records(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="test-sessionlog-")

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_a_record_has_every_call_and_the_changed_workspace(self):
        sessions = os.path.join(self.tmp, "s")
        with_rec = run(os.path.join(self.tmp, "a.jsonl"), sessions)
        without = run(os.path.join(self.tmp, "b.jsonl"))
        strip = [{k: v for k, v in r.items() if k != "commit"} for r in with_rec]
        self.assertEqual(strip, [{k: v for k, v in r.items() if k != "commit"} for r in without])
        rec = sessionlog.read(os.path.join(sessions, TASK + ".json.gz"))
        row = next(r for r in with_rec if r.get("id") == TASK)
        self.assertEqual(len(rec["calls"]), row["tool_calls"])
        self.assertEqual([c["name"] for c in rec["calls"]][-2:], ["tx_commit", "verify"])
        self.assertTrue(all("text" in c for c in rec["calls"]))
        self.assertTrue(rec["workspace"]["changed"])
        self.assertTrue(all(p.startswith("src/") for p in rec["workspace"]["changed"]))
        self.assertEqual(rec["requests"], [])  # a scripted agent makes no model requests

    def test_records_are_byte_stable(self):
        a, b = os.path.join(self.tmp, "x"), os.path.join(self.tmp, "y")
        sessionlog.write(a, {"id": "t", "calls": []})
        sessionlog.write(b, {"id": "t", "calls": []})
        with open(os.path.join(a, "t.json.gz"), "rb") as f, open(os.path.join(b, "t.json.gz"), "rb") as g:
            self.assertEqual(f.read(), g.read())

    def test_workspace_changes(self):
        seed, ws = os.path.join(self.tmp, "seed"), os.path.join(self.tmp, "ws")
        for d, files in ((seed, {"a": "1", "b": "2"}), (ws, {"a": "1", "c": "3", ".bmb-cache/x": "y"})):
            for name, text in files.items():
                os.makedirs(os.path.dirname(os.path.join(d, name)), exist_ok=True)
                with open(os.path.join(d, name), "w") as f:
                    f.write(text)
        self.assertEqual(sessionlog.workspace_changes(seed, ws), {"changed": {"c": "3"}, "removed": ["b"]})


if __name__ == "__main__":
    unittest.main()
