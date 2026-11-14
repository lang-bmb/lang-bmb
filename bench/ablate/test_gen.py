#!/usr/bin/env python3
"""The ablation sets (`gen.py`, backlog T0-37) are the committed ones, differ
from their source in one key of `set.json` and nothing else the model or the
judge reads, their servers show the tool list each should, and the scripted
reference and weak agents pass and fail through the harness of each."""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
ROOT = os.path.dirname(BENCH)
sys.path.insert(0, HERE)
import gen  # noqa: E402

# A few tasks per set: seed, tasks and judge are the source sets' (checked
# below), whose full runs are in the gate; the scripted agents read neither
# the prompt nor the tool descriptions.
BVB_TASKS = "vb-clover-binary-search,vc-va0093-max-balance"
BRB_TASKS = "t01-clamp,t03-sat-sum"


def bmb():
    return os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


def served_tools(flags, seed):
    msg = '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n'
    r = subprocess.run([bmb(), "mcp", *flags, seed], input=msg, capture_output=True, text=True, check=True)
    return sorted(t["name"] for t in json.loads(r.stdout)["result"]["tools"])


class Generated(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="bmb-ablate-")
        cls.sets = gen.build(cls.tmp)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def test_six_sets(self):
        self.assertEqual(sorted(os.path.basename(s) for s in self.sets),
                         sorted(f"{s}-{v}" for s in gen.SOURCES for v in gen.VARIANTS))

    def test_committed_sets_are_the_generated_ones(self):
        for out in self.sets:
            name = os.path.basename(out)
            self.assertEqual(gen.edit.tree(out), gen.edit.tree(os.path.join(BENCH, name)), name)

    def test_only_one_key_of_the_configuration_differs(self):
        for source in gen.SOURCES:
            a = gen.edit.tree(os.path.join(BENCH, source))
            old = json.loads(a["set.json"])
            for variant, (change, *_rest) in gen.VARIANTS.items():
                name = f"{source}-{variant}"
                b = gen.edit.tree(os.path.join(BENCH, name))
                # Everything the model and the judge read is the source's.
                same = sorted(r for r in a if r not in ("set.json", "MANIFEST", "README.md"))
                self.assertEqual(same, sorted(r for r in b if r not in ("set.json", "MANIFEST", "README.md")), name)
                for rel in same:
                    self.assertEqual(a[rel], b[rel], f"{name}/{rel}")
                new = json.loads(b["set.json"])
                (key, value), = change.items()
                for k in set(old) | set(new):
                    if k != key:
                        self.assertEqual(old.get(k), new.get(k), f"{name}: {k}")
                if key == "mcp_flags":
                    self.assertEqual(new[key], old[key] + [value], name)
                else:
                    self.assertNotIn(key, old)
                    self.assertEqual(new[key], value, name)

    def test_the_server_of_each_set_shows_its_tools(self):
        seed = os.path.join(BENCH, "bvb-write", "seed")
        with open(os.path.join(BENCH, "bvb-write", "set.json")) as f:
            base = json.load(f)
        plain = served_tools(base["mcp_flags"], seed)
        self.assertNotIn("probe", plain)
        for variant in gen.VARIANTS:
            for source in gen.SOURCES:
                with open(os.path.join(BENCH, f"{source}-{variant}", "set.json")) as f:
                    cfg = json.load(f)
                tools = served_tools(cfg["mcp_flags"], seed)
                # Only the probe set has one tool more; the others' lists are
                # the source's (`--undecided-info` changes responses, the
                # history is the harness's).
                self.assertEqual(tools, sorted(plain + ["probe"]) if variant == "probe" else plain,
                                 f"{source}-{variant}")
                self.assertEqual(cfg.get("history", "full"), "stateless" if variant == "stateless" else "full")

    def run_rows(self, cmd):
        r = subprocess.run([sys.executable, *cmd], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, (cmd, r.stderr[-2000:]))
        return [json.loads(l) for l in r.stdout.splitlines() if l.startswith("{")]

    def test_reference_passes_and_weak_fails(self):
        m1 = os.path.join(BENCH, "m1", "harness.py")
        brb = os.path.join(BENCH, "brb", "harness.py")
        for variant in gen.VARIANTS:
            d = os.path.join(BENCH, f"bvb-write-{variant}")
            rows = self.run_rows([m1, "--set", d, "run", "--agent", "oracle", "--expect", "all-pass",
                                  "--tasks", BVB_TASKS, "--jobs", "2"])
            self.assertEqual(sorted(r["id"] for r in rows if "id" in r), sorted(BVB_TASKS.split(",")))
            self.run_rows([m1, "--set", d, "run", "--agent", "weak", "--expect", "weak-fail",
                           "--tasks", BVB_TASKS, "--jobs", "2"])
            d = os.path.join(BENCH, f"brb-write-{variant}")
            rows = self.run_rows([brb, "run", "--arm", "bmb", "--variant", d, "--agent", "oracle",
                                  "--expect", "all-pass", "--tasks", BRB_TASKS, "--jobs", "2", "--proptest", "off"])
            self.assertEqual({r["task_set"] for r in rows if "task_set" in r}, {f"brb-write-{variant}"})
            self.assertEqual(sorted(r["id"] for r in rows if "id" in r), sorted(BRB_TASKS.split(",")))
            self.run_rows([brb, "run", "--arm", "bmb", "--variant", d, "--agent", "weak",
                           "--expect", "weak-fail", "--tasks", BRB_TASKS, "--jobs", "2", "--proptest", "off"])

    def test_the_rust_arm_takes_the_same_history(self):
        rows = self.run_rows([os.path.join(BENCH, "brb", "harness.py"), "run", "--arm", "rust", "--agent", "oracle",
                              "--expect", "all-pass", "--tasks", "t01-clamp", "--history", "stateless",
                              "--proptest", "off"])
        self.assertEqual([r["history"] for r in rows if r.get("summary")], ["stateless"])
        self.assertEqual([r["id"] for r in rows if "id" in r], ["t01-clamp"])


if __name__ == "__main__":
    unittest.main()
