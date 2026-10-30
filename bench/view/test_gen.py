#!/usr/bin/env python3
"""The compact view sets (`gen.py`) are the write sets plus `--compact`, and the
scripted reference and weak agents still pass and fail through the compact
view (backlog 2D-23, ADR-0068)."""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.path.insert(0, HERE)
import gen  # noqa: E402

HARNESS = os.path.join(BENCH, "m1", "harness.py")
# A few tasks per set: the full sets run on the frozen write sets in the gate.
TASKS = {"m1b-write-compact": "w01-all-in-range,w14-transfer-post,n04-impact-contract",
         "bvb-write-compact": "vb-clover-binary-search,vc-va0093-max-balance"}


class Generated(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="bmb-view-")
        cls.sets = gen.build(cls.tmp)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def test_only_the_server_flag_differs(self):
        for out in self.sets:
            src = os.path.join(BENCH, os.path.basename(out)[:-len("-compact")])
            for rel in ("prompt.md",) + tuple(os.path.join("tasks", f) for f in os.listdir(os.path.join(src, "tasks"))):
                with open(os.path.join(src, rel), "rb") as a, open(os.path.join(out, rel), "rb") as b:
                    self.assertEqual(a.read(), b.read(), rel)
            old, new = (json.load(open(os.path.join(d, "set.json"))) for d in (src, out))
            self.assertEqual(new["mcp_flags"], old["mcp_flags"] + ["--compact"])
            self.assertEqual(new["hide_tools"], old["hide_tools"])

    def run_agent(self, out, agent, expect):
        r = subprocess.run([sys.executable, HARNESS, "--set", out, "run", "--agent", agent, "--expect", expect,
                            "--tasks", TASKS[os.path.basename(out)], "--jobs", "2"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr[-2000:])
        return [json.loads(l) for l in r.stdout.splitlines() if l.startswith("{")]

    def test_reference_passes_and_weak_fails(self):
        for out in self.sets:
            rows = self.run_agent(out, "oracle", "all-pass")
            self.assertTrue(rows[-1]["summary"])
            self.assertEqual(sorted(r["id"] for r in rows[:-1]), sorted(TASKS[os.path.basename(out)].split(",")))
            self.run_agent(out, "weak", "weak-fail")


if __name__ == "__main__":
    unittest.main()
