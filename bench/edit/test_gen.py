#!/usr/bin/env python3
"""The write-surface sets (`gen.py`, backlog T1-06, ADR-0073) differ from their
source sets only in the write surface, the model is shown exactly the tools
the prompt names, and the scripted agents' surface translation
(`bench/m1/surface.py`) writes what the transaction would. The full scripted
runs of the committed sets are gate jobs (`scripts/check.sh`, `bench edit`)."""
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
sys.path.insert(0, os.path.join(BENCH, "m1"))
import gen  # noqa: E402
import surface  # noqa: E402

# What each surface shows the model besides `query`, `map` and `verify`.
SHOWN = {"write": ["build", "write"], "diff": ["build", "edit_file", "read_file"],
         "ast": ["build", "edit_ast", "read_ast"]}


def bmb():
    return os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


def served_tools(set_dir, ws):
    cfg = json.load(open(os.path.join(set_dir, "set.json")))
    msg = '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n'
    r = subprocess.run([bmb(), "mcp", *cfg["mcp_flags"], ws], input=msg, capture_output=True, text=True, check=True)
    return {t["name"] for t in json.loads(r.stdout)["result"]["tools"] if t["name"] not in cfg["hide_tools"]}


def mcp_session(flags, ws, calls):
    """Runs tool calls on a fresh `bmb mcp`; returns (text, is_error) per call."""
    lines = [json.dumps({"jsonrpc": "2.0", "id": i + 1, "method": "tools/call",
                         "params": {"name": n, "arguments": a}}) for i, (n, a) in enumerate(calls)]
    r = subprocess.run([bmb(), "mcp", *flags, ws], input="\n".join(lines) + "\n", capture_output=True, text=True,
                       check=True)
    out = []
    for line in r.stdout.splitlines():
        res = json.loads(line)["result"]
        out.append((res["content"][0]["text"], res["isError"]))
    return out


def surface_of(name):
    return name.rsplit("-", 1)[1]


class Generated(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="bmb-edit-")
        cls.sets = gen.build(cls.tmp)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def test_committed_sets_are_the_generated_ones(self):
        self.assertEqual(len(self.sets), 9)
        for out in self.sets:
            name = os.path.basename(out)
            self.assertEqual(gen.tree(out), gen.tree(os.path.join(BENCH, name)), name)

    def test_only_the_write_surface_differs(self):
        for out in self.sets:
            name = os.path.basename(out)
            if name.startswith("brb-"):
                self.assertEqual(sorted(gen.tree(out)), ["MANIFEST", "README.md", "prompt-bmb.md", "set.json"])
                continue
            a, b = gen.tree(os.path.join(BENCH, name.rsplit("-", 1)[0])), gen.tree(out)
            frozen = sorted(r for r in a if r.startswith(("seed", "tasks", "mutants")))
            self.assertEqual(frozen, sorted(r for r in b if r.startswith(("seed", "tasks", "mutants"))))
            for rel in frozen:
                self.assertEqual(a[rel], b[rel], rel)
            # The language rules are the source set's.
            old, new = a["prompt.md"].decode(), b["prompt.md"].decode()
            rules = old[old.index("## Language rules"):old.index("## Example")]
            self.assertIn(rules, new)

    def test_the_prompt_names_only_tools_the_model_has(self):
        ws = os.path.join(self.tmp, "empty-ws")
        os.makedirs(ws, exist_ok=True)
        for out in self.sets:
            name = os.path.basename(out)
            tools = served_tools(out, ws)
            self.assertEqual(sorted(tools), sorted(["map", "query", "verify"] + SHOWN[surface_of(name)]), name)
            prompt = open(os.path.join(out, "prompt-bmb.md" if name.startswith("brb-") else "prompt.md")).read()
            section = prompt[prompt.index("## Tools"):prompt.index("## Language rules")]
            named = set(re.findall(r"^- `([a-z_]+)`", section, re.M))
            self.assertLessEqual(named - {"answer"}, tools, name)
            # Every tool that writes is named; the transaction tools are not.
            self.assertIn({"write": "write", "diff": "edit_file", "ast": "edit_ast"}[surface_of(name)], named)
            self.assertNotIn("tx_begin", prompt)

    def test_the_ast_example_is_what_read_ast_shows(self):
        for out in self.sets:
            name = os.path.basename(out)
            if surface_of(name) != "ast":
                continue
            prompt = open(os.path.join(out, "prompt-bmb.md" if name.startswith("brb-") else "prompt.md")).read()
            code = re.search(r"## Example\n```\n(.*?)```\n", prompt, re.S).group(1)
            shown = re.search(r"shows it at `/demo`:\n```\n(.*?)\n```\n", prompt, re.S).group(1)
            ws = os.path.join(self.tmp, "example-ws")
            shutil.rmtree(ws, ignore_errors=True)
            os.makedirs(os.path.join(ws, "src"))
            with open(os.path.join(ws, "src", "demo.bmb"), "w") as f:
                f.write(code)
            (text, err), = mcp_session(["--ast-tools"], ws, [("read_ast", {"path": "/demo"})])
            self.assertFalse(err, text)
            self.assertEqual(text, shown)


class Scripts(unittest.TestCase):
    """The scripted agents' writes through each surface end in the files the
    transaction would commit."""

    OPS = [{"op": "unit", "path": "calc", "summary": "calc"},
           {"op": "put", "unit": "calc", "code": "#[effects(pure)]\nfn one() -> i64 { 1 }"},
           {"op": "put", "unit": "arith", "code": "#[effects(pure)]\nfn two() -> i64 { 2 }"}]

    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="bmb-surface-test-")
        self.seed = os.path.join(BENCH, "m1b", "seed")

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def ws(self, name):
        d = os.path.join(self.tmp, name)
        shutil.copytree(os.path.join(self.seed, "src"), os.path.join(d, "src"))
        return d

    def test_each_surface_writes_what_the_transaction_commits(self):
        want = self.ws("tx")
        with open(os.path.join(self.tmp, "ops.json"), "w") as f:
            json.dump(self.OPS, f)
        subprocess.run([bmb(), "apply", want, os.path.join(self.tmp, "ops.json")], check=True, capture_output=True)
        for name, flags in (("diff", ["--file-tools"]), ("ast", ["--ast-tools"])):
            ws = self.ws(name)
            calls = surface.calls(name, self.OPS, ws, bmb())
            self.assertEqual(len(calls), 1)
            (text, err), = mcp_session(flags, ws, calls)
            self.assertIn('"committed":true', text, name)
            self.assertEqual(surface._files(ws), surface._files(want), name)

    def test_ops_that_do_not_commit_only_verify(self):
        bad = [{"op": "put", "unit": "nope", "code": "#[effects(pure)]\nfn f() -> i64 { 1 }"}]
        for name in ("diff", "ast"):
            self.assertEqual(surface.calls(name, bad, self.ws(name), bmb()), [("verify", {})])
            self.assertEqual(surface.calls(name, [], self.ws(name + "2"), bmb()), [("verify", {})])

    def test_json_patch_turns_one_document_into_the_other(self):
        a = {"u": {"fns": {"f": {"x": [1, 2]}, "g": 1}}, "v/w": {}}
        b = {"u": {"fns": {"f": {"x": [1, 3]}, "h": 2}}, "z": [1]}
        ops = surface.json_patch(a, b)
        self.assertEqual(ops, [{"op": "remove", "path": "/v~1w"},
                               {"op": "remove", "path": "/u/fns/g"},
                               {"op": "replace", "path": "/u/fns/f/x/1", "value": 3},
                               {"op": "add", "path": "/u/fns/h", "value": 2},
                               {"op": "add", "path": "/z", "value": [1]}])


if __name__ == "__main__":
    unittest.main()
