#!/usr/bin/env python3
"""H-REPAIR's ablation (backlog T1-45): the generated set differs from
m1b-write only in its tool filter, the filter takes out exactly the repair
information, the harness counts failed proofs, the scripted agents still pass
and fail, and the verdict follows the registered rule. No network, no model.
Needs `bmb` ($BMB or target/debug/bmb)."""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.dont_write_bytecode = True
sys.path.insert(0, HERE)
import compare  # noqa: E402
import gen  # noqa: E402

harness = gen.harness
HARNESS = os.path.join(BENCH, "m1", "harness.py")
SET = os.path.join(BENCH, gen.NAME)
SOURCE = os.path.join(BENCH, gen.SOURCE)
TASKS = "w01-all-in-range,w14-transfer-post,n04-impact-contract"

DIAG = ('{"code":"E0803","severity":"error","message":"`+` may overflow `i64` in `u::f`; counterexample: x = 9",'
        '"file":"src/u.bmb","line":5,"col":9,"end_line":5,"end_col":14}')
UNDECIDED = ('{"code":"E0811","severity":"error","message":"postcondition in `u::g`: solver could not decide '
             '(resource limit reached, rlimit 1); unconfirmed counterexample: a = [1, 2] (len 2)",'
             '"file":"src/u.bmb","line":1,"col":1,"end_line":1,"end_col":2,'
             '"fix":{"message":"the proof sees `p` only by name; add to #[uses(p)]"}}')
SYNTAX = ('{"code":"E0601","severity":"error","message":"`g` has no `#[effects(..)]` attribute","file":"tx/op0",'
          '"line":1,"col":24,"end_line":1,"end_col":26,"fix":{"message":"add `#[effects(pure)]` above the function",'
          '"replacement":"#[effects(pure)] fn"}}')
ROW = '{"def":"u::f","status":"failed","obligations":1,"failed":1,"rlimit":558,"rlimit_max":558}'


class Generated(unittest.TestCase):
    def test_committed_set_is_the_generated_one(self):
        tmp = tempfile.mkdtemp(prefix="bmb-repair-")
        try:
            self.assertEqual(gen.tree(gen.build(tmp)), gen.tree(SET))
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_only_the_tool_filter_differs(self):
        a, b = gen.tree(SOURCE), gen.tree(SET)
        self.assertEqual(sorted(a), sorted(b))
        for rel in a:
            if rel not in ("set.json", "MANIFEST", "README.md"):
                self.assertEqual(a[rel], b[rel], rel)
        old, new = (json.loads(t["set.json"]) for t in (a, b))
        self.assertEqual(new, dict(old, tool_filter="no-repair"))


class Filter(unittest.TestCase):
    def test_counterexamples_and_fixes_go(self):
        out = harness.strip_repair("\n".join(['{"committed":true,"changed":["u::f"],"removed":[]}', DIAG, UNDECIDED,
                                              SYNTAX, ROW, ""])).split("\n")
        self.assertEqual(out[0], '{"committed":true,"changed":["u::f"],"removed":[]}')
        self.assertEqual(json.loads(out[1])["message"], "`+` may overflow `i64` in `u::f`")
        self.assertEqual(json.loads(out[2])["message"],
                         "postcondition in `u::g`: solver could not decide (resource limit reached, rlimit 1)")
        self.assertNotIn("fix", json.loads(out[2]))
        self.assertEqual(json.loads(out[3]), {k: v for k, v in json.loads(SYNTAX).items() if k != "fix"})
        self.assertEqual(out[4], ROW)
        self.assertEqual(out[5], "")
        for line in out:
            self.assertNotIn("counterexample", line)
            self.assertNotIn('"fix"', line)

    def test_nested_and_plain_text(self):
        nested = json.dumps({"diagnostics": [json.loads(DIAG)], "weaken": {"counterexample": {"x": 1}, "v": 2}},
                            separators=(",", ":"))
        self.assertEqual(json.loads(harness.strip_repair(nested)),
                         {"diagnostics": [{**json.loads(DIAG), "message": "`+` may overflow `i64` in `u::f`"}],
                          "weaken": {"v": 2}})
        self.assertEqual(harness.strip_repair("not json; counterexample: x = 1"), "not json; counterexample: x = 1")
        # A counterexample in the middle of a message is not the tail form and stays.
        mid = '{"message":"see the counterexample: none here"}'
        self.assertEqual(harness.strip_repair(mid), mid)

    def test_the_served_results_are_filtered_only_in_the_ablation_set(self):
        ws = tempfile.mkdtemp(prefix="bmb-repair-ws-")
        saved = harness.SET
        try:
            os.makedirs(os.path.join(ws, "src"))
            texts = {}
            for name, set_dir in (("full", SOURCE), ("none", SET)):
                harness.SET = set_dir
                mcp = harness.Mcp(ws)
                try:
                    over, _ = mcp.call("write", {"unit": f"u_{name}", "summary": "x",
                                                 "code": "#[effects(pure)]\nfn f(x: i64) -> i64 { x + 1 }"})
                    bad, _ = mcp.call("write", {"unit": f"u_{name}", "code": "fn g(x: i64) -> i64 { x }"})
                finally:
                    mcp.close()
                texts[name] = (over, bad)
            self.assertIn("; counterexample: x = ", texts["full"][0])
            self.assertIn('"fix":', texts["full"][1])
            for t in texts["none"]:
                self.assertNotIn("counterexample", t)
                self.assertNotIn('"fix"', t)
            self.assertIn('"code":"E0803"', texts["none"][0])
            self.assertTrue(harness.proof_failed("write", texts["none"][0]))
            self.assertFalse(harness.proof_failed("write", texts["none"][1]))  # refused, counted as rejected
            self.assertIs(harness.commit_outcome("write", texts["none"][1]), False)
        finally:
            harness.SET = saved
            shutil.rmtree(ws, ignore_errors=True)


class Counting(unittest.TestCase):
    def test_failed_proofs(self):
        ok = '{"committed":true,"changed":["u::f"],"removed":[]}\n{"def":"u::f","status":"verified"}'
        bad = '{"committed":true,"changed":["u::f"],"removed":[]}\n' + DIAG + "\n" + ROW
        self.assertFalse(harness.proof_failed("write", ok))
        self.assertTrue(harness.proof_failed("write", bad))
        self.assertTrue(harness.proof_failed("verify", DIAG + "\n" + ROW))
        self.assertFalse(harness.proof_failed("verify", ROW.replace("failed", "verified")))
        self.assertFalse(harness.proof_failed("query", DIAG))

    def run_agent(self, agent, expect):
        out = tempfile.mktemp(suffix=".jsonl")
        try:
            r = subprocess.run([sys.executable, HARNESS, "--set", SET, "run", "--agent", agent, "--expect", expect,
                                "--tasks", TASKS, "--jobs", "2", "--out", out], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr[-2000:])
            return [json.loads(line) for line in open(out) if line.strip()]
        finally:
            if os.path.exists(out):
                os.unlink(out)

    def test_reference_passes_and_weak_fails(self):
        rows = self.run_agent("oracle", "all-pass")
        self.assertEqual(rows[-1]["task_set"], "m1b-write-norepair")
        writes = [r for r in rows if r.get("kind") == "write"]
        self.assertEqual(len(writes), 2)
        for r in writes:
            self.assertEqual((r["proofs_failed"], r["commits_rejected"]), (0, 0), r["id"])
        self.run_agent("weak", "weak-fail")


class Verdict(unittest.TestCase):
    def test_rule(self):
        v = compare.verdict
        self.assertIn("감소 40.0% → 확정", v((10, 3, 3), (10, 5, 5)))
        self.assertIn("→ 반증", v((10, 4, 3), (10, 5, 5)))
        self.assertIn("→ 반증", v((10, 6, 6), (10, 5, 5)))
        self.assertIn("판정 불가(성공한 작성 세션이 없는 쪽)", v((0, 0, 0), (10, 5, 5)))
        self.assertIn("판정 불가(절제 쪽 비용 0)", v((10, 0, 0), (10, 0, 0)))

    def write_run(self, path, task_set, rows, **head):
        with open(path, "w") as f:
            for r in rows:
                f.write(json.dumps(r) + "\n")
            f.write(json.dumps({"summary": True, "task_set": task_set, "agent": "a", "commit": "c", **head}) + "\n")

    def test_compare_counts_successful_write_sessions_only(self):
        tmp = tempfile.mkdtemp()
        try:
            a, b = os.path.join(tmp, "a.jsonl"), os.path.join(tmp, "b.jsonl")
            row = {"kind": "write", "success": True, "commits_rejected": 1, "proofs_failed": 1}
            self.write_run(a, "m1b-write", [row, dict(row, success=False, proofs_failed=9),
                                            {"kind": "navigate", "success": True}])
            self.write_run(b, "m1b-write-norepair", [dict(row, proofs_failed=3), dict(row, proofs_failed=5)])
            text = compare.compare(a, b)
            self.assertIn("| 작성 과제 성공 | 1/2 | 2/2 |", text)
            self.assertIn("전체 (1+1)/1 = 2.00, 절제 (2+8)/2 = 5.00, 감소 60.0% → 확정", text)
            self.write_run(b, "m1b-write-norepair", [row], commit="other")
            with self.assertRaises(SystemExit):
                compare.compare(a, b)
            with self.assertRaises(SystemExit):
                compare.compare(b, a)
        finally:
            shutil.rmtree(tmp)


if __name__ == "__main__":
    unittest.main()
