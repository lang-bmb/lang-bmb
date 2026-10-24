"""Judging logic of the M1 harness (backlog 1D-21). No network, no paid API.

Needs the `bmb` binary (target/debug/bmb, or $BMB) like the harness itself.
Frozen inputs are only read: workspaces are temporary copies of `seed/`.
"""
import copy
import json
import os
import shutil
import sys
import tempfile
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.dont_write_bytecode = True
import harness  # noqa: E402


def task(name):
    with open(os.path.join(HERE, "tasks", name + ".json")) as f:
        return json.load(f)


def run_with_ops(t, ops):
    """Runs a write task with a scripted agent that applies `ops` through MCP."""
    agent = harness.ScriptAgent(harness.ops_to_calls(ops))
    with mock.patch.object(harness, "make_agent", lambda kind, task, ws: agent):
        return harness.run_task(t, "scripted")


class Workspace(unittest.TestCase):
    """A temporary copy of the seed workspace."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="bmb-test-harness-")
        self.ws = os.path.join(self.tmp, "ws")
        shutil.copytree(os.path.join(HERE, "seed"), self.ws)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)


class JudgeNavigate(Workspace):
    def setUp(self):
        super().setUp()
        self.task = task("n01-callers")
        self.want = sorted(harness.truth(self.task, self.ws))
        self.assertTrue(self.want, "n01 truth should be non-empty")

    def judge(self, answer):
        return harness.judge_navigate(self.task, self.ws, answer)

    def test_exact_answer_succeeds(self):
        v = self.judge(list(self.want))
        self.assertEqual((v["precision"], v["recall"], v["success"]), (1.0, 1.0, True))

    def test_answer_never_given(self):
        v = self.judge(None)
        self.assertEqual((v["precision"], v["recall"], v["success"]), (0.0, 0.0, False))
        self.assertEqual(v["answer"], [])

    def test_empty_answer(self):
        v = self.judge([])
        self.assertEqual((v["precision"], v["recall"], v["success"]), (0.0, 0.0, False))

    def test_duplicate_entries_count_once(self):
        v = self.judge(self.want + self.want)
        self.assertEqual((v["precision"], v["recall"], v["success"]), (1.0, 1.0, True))
        self.assertEqual(v["answer"], self.want)

    def test_superset_answer_fails_on_precision(self):
        extra = self.want + ["arrays::not_a_caller"]
        v = self.judge(extra)
        self.assertEqual(v["recall"], 1.0)
        self.assertLess(v["precision"], 1.0)
        self.assertFalse(v["success"])

    def test_subset_answer_fails_on_recall(self):
        if len(self.want) < 2:
            self.skipTest("needs at least two true definitions")
        v = self.judge(self.want[:1])
        self.assertEqual(v["precision"], 1.0)
        self.assertLess(v["recall"], 1.0)
        self.assertFalse(v["success"])

    def test_empty_truth(self):
        # No frozen task has an empty truth; the rule is checked directly.
        with mock.patch.object(harness, "truth", lambda t, ws: set()):
            wrong = harness.judge_navigate(self.task, self.ws, ["arrays::swap"])
            self.assertEqual((wrong["precision"], wrong["recall"]), (0.0, 1.0))
            self.assertFalse(wrong["success"])
            # Finding (1D-21): the correct answer to a question with no
            # definitions -- the empty list -- is judged a failure, because
            # precision of an empty answer is 0.0 regardless of the truth.
            right = harness.judge_navigate(self.task, self.ws, [])
            self.assertEqual((right["precision"], right["recall"]), (0.0, 1.0))
            self.assertFalse(right["success"])


class Signatures(unittest.TestCase):
    def test_strip_result_name_array(self):
        self.assertEqual(harness.strip_result_name("fn f(a: &[i64]) -> (r: [i64; 3])"),
                         "fn f(a: &[i64]) -> [i64; 3]")

    def test_strip_result_name_unit(self):
        self.assertEqual(harness.strip_result_name("fn f(a: &mut [i64]) -> ()"),
                         "fn f(a: &mut [i64]) -> ()")

    def test_strip_result_name_unnamed_matches_named(self):
        self.assertEqual(harness.strip_result_name("fn f(x: i64) -> i64"),
                         harness.strip_result_name("fn f(x: i64) -> (r: i64)"))

    def test_parse_signature_array(self):
        self.assertEqual(harness.parse_signature("fn f(a: &[i64], n: usize) -> (r: [i64; 3])"),
                         ("f", "a: &[i64], n: usize", "[i64; 3]"))

    def test_parse_signature_unit(self):
        self.assertEqual(harness.parse_signature("fn fill_from(a: &mut [i64], start: usize, v: i64) -> ()"),
                         ("fill_from", "a: &mut [i64], start: usize, v: i64", "()"))

    def test_parse_signature_no_params(self):
        self.assertEqual(harness.parse_signature("fn f() -> (r: i64)"), ("f", "", "i64"))


class JudgeWrite(unittest.TestCase):
    """Names the agent may choose that also appear in the hidden acceptance module."""

    def setUp(self):
        self.task = task("w01-clamp")

    def verdict(self, r):
        return {k: r[k] for k in ("verified", "signature_ok", "spec_accepted", "success")}

    def test_reference_is_accepted(self):
        r = run_with_ops(self.task, self.task["reference"])
        self.assertTrue(r["success"], self.verdict(r))

    def test_weak_reference_is_rejected(self):
        r = run_with_ops(self.task, self.task["weak_reference"])
        self.assertEqual(self.verdict(r),
                         {"verified": True, "signature_ok": True, "spec_accepted": False, "success": False})

    def test_agent_spec_fns_named_ref_pre_ref_post(self):
        # The agent's own `tasks::ref_pre` / `tasks::ref_post` do not clash with
        # `bmb_accept::ref_pre` / `bmb_accept::ref_post`: names resolve per unit.
        def renamed(ops, post):
            out = copy.deepcopy(ops)
            for op in out:
                if "code" in op:
                    op["code"] = op["code"].replace("ordered", "ref_pre").replace(post, "ref_post")
            return out

        strong = renamed(self.task["reference"], "clamped")
        self.assertIn("spec fn ref_post", strong[1]["code"])
        r = run_with_ops(self.task, strong)
        self.assertTrue(r["success"], self.verdict(r))

        weak = renamed(self.task["weak_reference"], "above")
        self.assertIn("spec fn ref_post", weak[1]["code"])
        r = run_with_ops(self.task, weak)
        self.assertEqual(self.verdict(r),
                         {"verified": True, "signature_ok": True, "spec_accepted": False, "success": False})

    def test_agent_unit_named_bmb_accept(self):
        # Finding (1D-21): an extra agent unit named `bmb_accept` duplicates the
        # hidden acceptance unit, so a correct solution is judged not accepted.
        # A wrong solution cannot pass this way (the duplicate is an error).
        extra = [{"op": "unit", "path": "bmb_accept", "summary": "도움"},
                 {"op": "put", "unit": "bmb_accept",
                  "code": "spec fn helper(x: i64) -> bool { x > 0 }"}]
        r = run_with_ops(self.task, self.task["reference"] + extra)
        self.assertEqual(self.verdict(r),
                         {"verified": True, "signature_ok": True, "spec_accepted": False, "success": False})
        r = run_with_ops(self.task, self.task["weak_reference"] + extra)
        self.assertFalse(r["success"], self.verdict(r))

    def test_unit_return_task(self):
        # `-> ()` tasks build an acceptance check without a result name.
        t = task("w05-fill-from")
        r = run_with_ops(t, t["reference"])
        self.assertTrue(r["success"], self.verdict(r))


class FrozenInputs(unittest.TestCase):
    def test_manifest_still_verifies(self):
        with open(os.path.join(HERE, "MANIFEST")) as f:
            self.assertEqual(f.read(), harness.manifest_text())


if __name__ == "__main__":
    # The harness leaves subprocess pipes to the garbage collector.
    unittest.main(warnings="ignore")
