#!/usr/bin/env python3
"""The lean sets (`gen.py`, backlog 2D-28) differ from the write sets only in
what every request carries, the tools the prompt stops describing are
described by the server, and the scripted reference and weak agents still
pass and fail on the committed sets."""
import json
import os
import re
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

HARNESS = os.path.join(BENCH, "m1", "harness.py")
# A few tasks per set: seed, tasks and mutants are the write sets' (checked
# below), whose full runs are in the gate; the scripted agents read neither
# the prompt nor the tool descriptions.
TASKS = {"m1b-write-lean": "w01-all-in-range,w14-transfer-post,n04-impact-contract",
         "bvb-write-lean": "vb-clover-binary-search,vc-va0093-max-balance"}


def bmb():
    return os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


def served_tools(set_dir):
    cfg = json.load(open(os.path.join(set_dir, "set.json")))
    msg = '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n'
    r = subprocess.run([bmb(), "mcp", *cfg["mcp_flags"], os.path.join(set_dir, "seed")], input=msg,
                       capture_output=True, text=True, check=True)
    return {t["name"]: t for t in json.loads(r.stdout)["result"]["tools"] if t["name"] not in cfg["hide_tools"]}


class Generated(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="bmb-lean-")
        cls.sets = gen.build(cls.tmp)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def test_committed_sets_are_the_generated_ones(self):
        for out in self.sets:
            name = os.path.basename(out)
            self.assertEqual(gen.edit.tree(out), gen.edit.tree(os.path.join(BENCH, name)), name)

    def test_only_the_fixed_part_differs(self):
        for out in self.sets:
            src = os.path.join(BENCH, os.path.basename(out)[:-len("-lean")])
            a, b = gen.edit.tree(src), gen.edit.tree(out)
            for rel in a:
                if rel.startswith(("seed", "tasks", "mutants")):
                    self.assertEqual(a[rel], b[rel], rel)
            self.assertEqual(sorted(r for r in a if r.startswith(("seed", "tasks", "mutants"))),
                             sorted(r for r in b if r.startswith(("seed", "tasks", "mutants"))))
            old, new = (json.load(open(os.path.join(d, "set.json"))) for d in (src, out))
            self.assertEqual(new["mcp_flags"], old["mcp_flags"] + ["--tool-docs"])
            self.assertEqual(new["hide_tools"], old["hide_tools"] + ["build"])
            old_p, new_p = (open(os.path.join(d, "prompt.md")).read() for d in (src, out))
            self.assertLess(len(new_p), len(old_p))
            self.assertNotIn("## Tools", new_p)
            # The example and the way of working are the write set's.
            for head in ("## Example", "## How to work"):
                self.assertIn(old_p[old_p.index(head):].split("\n## ")[0], new_p)

    def test_the_prompt_names_only_tools_the_model_has(self):
        for out in self.sets:
            tools = served_tools(out)
            self.assertEqual(sorted(tools), ["map", "query", "verify", "write"])
            prompt = open(os.path.join(out, "prompt.md")).read()
            named = set(re.findall(r"`([a-z_]+)`", prompt))
            # `answer` is the harness's own tool for questions.
            self.assertLessEqual(named & {"query", "map", "verify", "write", "answer", "build", "tx_begin"},
                                 set(tools) | {"answer"})

    def test_the_server_says_what_the_prompt_section_said(self):
        for out in self.sets:
            tools = served_tools(out)
            for part in ("Start with `units`", "`surface DEF` (signature, contract, proof status)",
                         "`slice DEF` (the definition and the contracts it depends on)"):
                self.assertIn(part, tools["query"]["description"])
            self.assertTrue(tools["write"]["description"].startswith("The only way to write"))
            for t in tools.values():
                self.assertNotRegex(t["description"], r"ADR-|backlog")

    def run_agent(self, name, agent, expect):
        r = subprocess.run([sys.executable, HARNESS, "--set", os.path.join(BENCH, name), "run", "--agent", agent,
                            "--expect", expect, "--tasks", TASKS[name], "--jobs", "2"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr[-2000:])
        return [json.loads(l) for l in r.stdout.splitlines() if l.startswith("{")]

    def test_reference_passes_and_weak_fails(self):
        for name in TASKS:
            rows = self.run_agent(name, "oracle", "all-pass")
            self.assertTrue(rows[-1]["summary"])
            self.assertEqual(sorted(r["id"] for r in rows[:-1]), sorted(TASKS[name].split(",")))
            self.run_agent(name, "weak", "weak-fail")


if __name__ == "__main__":
    unittest.main()
