"""Judging logic of the M1 harness (backlog 1D-21). No network, no paid API.

Needs the `bmb` binary (target/debug/bmb, or $BMB) like the harness itself.
Frozen inputs are only read: workspaces are temporary copies of `seed/`.
"""
import copy
import json
import os
import shutil
import subprocess
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
            # The empty list is the right answer when nothing is to be found.
            right = harness.judge_navigate(self.task, self.ws, [])
            self.assertEqual((right["precision"], right["recall"]), (1.0, 1.0))
            self.assertTrue(right["success"])


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
        self.assertNotIn("calls", r)  # the call log is opt-in

    def test_context_log_lists_every_call(self):
        # BMB_CONTEXT_LOG=1: every call in order with its result size (backlog 2D-25).
        with mock.patch.dict(os.environ, {"BMB_CONTEXT_LOG": "1"}):
            r = run_with_ops(self.task, self.task["reference"])
        names = [c[0] for c in harness.ops_to_calls(self.task["reference"])]
        self.assertEqual([c["name"] for c in r["calls"]], names)
        self.assertTrue(all(c["bytes"] > 0 for c in r["calls"]))

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
        # An agent unit named `bmb_accept` must not collide with the hidden
        # acceptance unit: the correct solution is still accepted.
        extra = [{"op": "unit", "path": "bmb_accept", "summary": "도움"},
                 {"op": "put", "unit": "bmb_accept",
                  "code": "spec fn helper(x: i64) -> bool { x > 0 }"}]
        r = run_with_ops(self.task, self.task["reference"] + extra)
        self.assertTrue(r["success"], self.verdict(r))
        r = run_with_ops(self.task, self.task["weak_reference"] + extra)
        self.assertFalse(r["success"], self.verdict(r))

    def test_unit_return_task(self):
        # `-> ()` tasks build an acceptance check without a result name.
        t = task("w05-fill-from")
        r = run_with_ops(t, t["reference"])
        self.assertTrue(r["success"], self.verdict(r))


M1B = os.path.join(os.path.dirname(HERE), "m1b")


class OtherSet(unittest.TestCase):
    """Runs a test with `harness.SET` pointing at the `m1b` task set (2A-04)."""

    def setUp(self):
        self.saved = harness.SET
        harness.SET = M1B

    def tearDown(self):
        harness.SET = self.saved


class FrozenInputs(OtherSet):
    def test_manifest_still_verifies(self):
        harness.SET = HERE
        with open(os.path.join(HERE, "MANIFEST")) as f:
            self.assertEqual(f.read(), harness.manifest_text())

    def test_m1b_manifest_still_verifies(self):
        with open(os.path.join(M1B, "MANIFEST")) as f:
            self.assertEqual(f.read(), harness.manifest_text())
        self.assertEqual(harness.set_name(), "m1b")

    def test_m1b_prompt_is_m1_prompt(self):
        # ADR-0025 §1: the two sets differ only in seed and tasks.
        with open(os.path.join(HERE, "prompt.md"), "rb") as a, open(os.path.join(M1B, "prompt.md"), "rb") as b:
            self.assertEqual(a.read(), b.read())


class TruthKinds(OtherSet):
    """Navigation truths added for m1b: `impact` rows, transitive closure, several queries."""

    def setUp(self):
        super().setUp()
        self.tmp = tempfile.mkdtemp(prefix="bmb-test-harness-")
        self.ws = os.path.join(self.tmp, "ws")
        shutil.copytree(os.path.join(M1B, "seed"), self.ws)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)
        super().tearDown()

    def truth(self, t):
        return sorted(harness.truth({"truth": t}, self.ws))

    def test_impact_stops_after_one_step(self):
        t = {"query": ["impact", "num::max2", "iface"], "reasons": ["changed", "calls", "uses", "names"], "field": "path"}
        self.assertEqual(self.truth(t), ["num::max2", "stats::max_of"])

    def test_impact_body_is_only_itself(self):
        t = {"query": ["impact", "ledger::withdraw", "body"], "reasons": ["changed", "calls", "uses", "names"], "field": "path"}
        self.assertEqual(self.truth(t), ["ledger::withdraw"])

    def test_closure_follows_calls(self):
        one = {"query": ["edges", "ledger::deposit", "in", "1"], "kinds": ["calls"], "field": "from"}
        self.assertEqual(self.truth(one), ["ledger::transfer"])
        self.assertEqual(self.truth(dict(one, closure=True)), ["app::main", "ledger::pay_all", "ledger::transfer"])

    def test_also_takes_one_query_or_a_list(self):
        base = {"query": ["contracts", "num::small"], "list": "uses_by"}
        one = self.truth(dict(base, also=["contracts", "num::ordered"]))
        many = self.truth(dict(base, also=[["contracts", "num::ordered"]]))
        self.assertEqual(one, many)
        self.assertLess(len(self.truth(base)), len(one))


class AlsoCheck(unittest.TestCase):
    """Multi-unit write tasks: every target must pass signature and spec strength."""

    def judge(self, outcomes):
        task = {"def": "tasks::f", "also_check": [{"def": "num::g"}]}
        with mock.patch.object(harness, "bmb", lambda *a: (0, "")), \
                mock.patch.object(harness, "judge_target", lambda t, ws, v: outcomes[t["def"]]):
            return harness.judge_write(task, "ws")

    def test_all_targets_pass(self):
        v = self.judge({"tasks::f": (True, True, None), "num::g": (True, True, None)})
        self.assertTrue(v["success"])
        self.assertNotIn("spec_rejections", v)

    def test_one_weak_target_fails(self):
        v = self.judge({"tasks::f": (True, True, None), "num::g": (True, False, "undecided")})
        self.assertEqual((v["spec_accepted"], v["success"]), (False, False))
        self.assertEqual(v["spec_rejections"], [{"def": "num::g", "reason": "undecided"}])

    def test_one_wrong_signature_fails(self):
        v = self.judge({"tasks::f": (False, False, None), "num::g": (True, True, None)})
        self.assertEqual((v["signature_ok"], v["success"]), (False, False))


class SummarizeRejections(unittest.TestCase):
    def test_refuted_and_undecided_are_counted_apart(self):
        rows = [{"summary": True, "agent": "a", "task_set": "m1"}]
        base = {"kind": "write", "read_calls": 0, "read_bytes": 0, "commits": 1, "e02": 0,
                "unproved_obligations": 0, "io_defs": 0, "tool_calls": 1, "limit_hit": False,
                "verified": True, "signature_ok": True}
        rows.append(dict(base, id="w1", success=False, spec_accepted=False,
                         spec_rejections=[{"def": "t::f", "reason": "undecided"}]))
        rows.append(dict(base, id="w2", success=False, spec_accepted=False,
                         spec_rejections=[{"def": "t::g", "reason": "refuted"}]))
        rows.append(dict(base, id="w3", success=True, spec_accepted=True))
        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as f:
            f.write("".join(json.dumps(r) + "\n" for r in rows))
        try:
            text = harness.summarize([f.name])
        finally:
            os.unlink(f.name)
        self.assertIn("| 명세 강도 거부: 반박(반례) | 1 |", text)
        self.assertIn("| 명세 강도 거부: 미결정(해법기) | 1 |", text)
        self.assertIn("| w1 | 실패: spec_accepted (undecided) |", text)


class SpecRejection(unittest.TestCase):
    def test_reason_comes_from_the_solver_verdict(self):
        self.assertEqual(harness.spec_rejection(
            '{"code":"E0811","message":"postcondition `ref_post` may not hold in `bmb_accept::check`: '
            'solver could not decide (a model exists for the instantiated definitions only)"}'), "undecided")
        self.assertEqual(harness.spec_rejection(
            '{"code":"E0811","message":"postcondition `ref_post` may not hold in `bmb_accept::check`: '
            'counterexample: x = 3"}'), "refuted")


def m1b_task(name):
    with open(os.path.join(M1B, "tasks", name + ".json")) as f:
        return json.load(f)


class SpecKill(OtherSet):
    """BSB (backlog 2A-10): hidden mutants of m1b write tasks and the kill rate."""

    def test_mutant_files_match_their_manifest(self):
        self.assertTrue(harness.bsb.check_manifest(M1B))
        self.assertFalse(harness.bsb.check_manifest(HERE))  # m1 has no mutant sets

    def test_every_write_task_has_a_mutant_set(self):
        tdir = os.path.join(M1B, "tasks")
        for f in sorted(os.listdir(tdir)):
            t = json.load(open(os.path.join(tdir, f)))
            if t["kind"] != "write":
                continue
            data = harness.bsb.load(M1B, t["id"])
            self.assertEqual([x["def"] for x in data["targets"]],
                             [t["def"]] + [x["def"] for x in t.get("also_check", [])])
            for target in data["targets"]:
                c = target["counts"]
                self.assertEqual(c["generated"], len(target["mutants"]) + len(target["discarded"]))
                self.assertEqual(c["kept"], len(target["mutants"]))
                self.assertEqual(c["generated"], c["kept"] + sum(c[k] for k in (
                    "equivalent", "trap_only", "allowed", "no_check")))
                for m in target["mutants"]:
                    self.assertTrue(1 <= len(m["witnesses"]) <= data["max_witnesses"])
                    for w in m["witnesses"]:
                        # A witness distinguishes the mutant from the reference.
                        self.assertNotEqual(w["expected"], w["output"])
                        self.assertIn(w["input"], target["inputs"])
            self.assertGreater(sum(len(x["mutants"]) for x in data["targets"]), 0, t["id"])

    def test_generator_reproduces_the_files(self):
        r = subprocess.run([sys.executable, os.path.join(HERE, "bsb_gen.py"), "--check",
                            "--tasks", "w03-max3,w23-div-floor"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_reference_contract_kills_every_mutant_and_weak_fewer(self):
        t = m1b_task("w04-argmin")
        strong = run_with_ops(t, t["reference"])
        self.assertTrue(strong["success"])
        self.assertEqual(strong["bsb_killed"], strong["bsb_mutants"])
        self.assertGreater(strong["bsb_mutants"], 0)
        self.assertEqual(strong["bsb_survivors"], [])
        weak = run_with_ops(t, t["weak_reference"])
        self.assertTrue(weak["verified"] and weak["signature_ok"])
        self.assertLess(weak["bsb_killed"], strong["bsb_killed"])
        self.assertEqual(len(weak["bsb_survivors"]), weak["bsb_mutants"] - weak["bsb_killed"])

    def test_no_postcondition_kills_nothing(self):
        # The kill check needs the contract: a verified body alone kills nothing.
        t = m1b_task("w03-max3")
        ops = [{"op": "unit", "path": "tasks", "summary": "과제 구현"},
               {"op": "put", "unit": "tasks", "code":
                "#[effects(pure)]\nfn max3(a: i64, b: i64, c: i64) -> (r: i64) {\n"
                "    num::max2(num::max2(a, b), c)\n}"}]
        r = run_with_ops(t, ops)
        self.assertEqual((r["verified"], r["signature_ok"], r["success"]), (True, True, False))
        self.assertEqual((r["bsb_killed"], r["bsb_mutants"]), (0, 1))

    def test_unverified_workspace_kills_nothing(self):
        t = m1b_task("w04-argmin")
        v = harness.spec_kill(t, "no-such-workspace", {"verified": False, "signature_ok": True})
        self.assertEqual(v["bsb_killed"], 0)
        self.assertEqual(len(v["bsb_survivors"]), v["bsb_mutants"])

    def test_kill_function_text(self):
        w = {"input": {"a": [3, -1], "x": 5, "f": True}, "output": {"r": False, "a": [-1, 3]}}
        sig = "fn f(a: &mut [i64], x: i64, f: bool) -> (r: bool)"
        text = harness.bsb.kill_fn("k0_1", ["p", "u::q"], "u::f", sig, [w])
        self.assertEqual(text, "#[effects(pure)]\n#[uses(p, u::q)]\nfn k0_1() -> () {\n"
                               "    let mut a: [i64; 2] = [3, -1];\n"
                               "    let bsb_r: bool = u::f(&mut a, 5, true);\n"
                               "    assert(!(!bsb_r && a[0] == -1 && a[1] == 3));\n}\n")
        # Two witnesses: independent calls; the outputs cannot all occur.
        w2 = {"input": {"a": [], "x": 0, "f": False}, "output": {"r": True, "a": []}}
        text = harness.bsb.kill_fn("k0_1", [], "u::f", sig, [w, w2])
        self.assertEqual(text, "#[effects(pure)]\nfn k0_1() -> () {\n"
                               "    let mut a_0: [i64; 2] = [3, -1];\n"
                               "    let bsb_r0: bool = u::f(&mut a_0, 5, true);\n"
                               "    let mut a_1: [i64; 0] = [];\n"
                               "    let bsb_r1: bool = u::f(&mut a_1, 0, false);\n"
                               "    assert(!((!bsb_r0 && a_0[0] == -1 && a_0[1] == 3) && (bsb_r1)));\n}\n")


class Summarize(unittest.TestCase):
    def write(self, d, name, task_set):
        head = {"summary": True, "agent": "oracle"}
        if task_set:
            head["task_set"] = task_set
        rec = {"id": "w01", "kind": "write", "success": True, "verified": True, "signature_ok": True,
               "spec_accepted": True, "read_calls": 0, "read_bytes": 0, "commits": 1, "e02": 0,
               "unproved_obligations": 0, "io_defs": 0, "tool_calls": 3, "limit_hit": False}
        p = os.path.join(d, name)
        with open(p, "w") as f:
            f.write(json.dumps(rec) + "\n" + json.dumps(head) + "\n")
        return p

    def test_mixed_task_sets_are_refused(self):
        with tempfile.TemporaryDirectory() as d:
            a = self.write(d, "a.jsonl", None)  # before 2A-04: m1
            b = self.write(d, "b.jsonl", "m1b")
            with self.assertRaises(SystemExit):
                harness.summarize([a, b])

    def test_same_task_set_is_one_table(self):
        with tempfile.TemporaryDirectory() as d:
            a = self.write(d, "a.jsonl", "m1b")
            b = self.write(d, "b.jsonl", "m1b")
            out = harness.summarize([a, b])
            self.assertIn("| 과제 집합 | m1b | m1b |", out)
            self.assertIn("| 작성 1회 성공률 | 1/1 | 1/1 |", out)
            # No BSB fields (m1, or results from before 2A-10): no rate.
            self.assertIn("| 명세 변이체 판별률 | — | — |", out)

    def test_kill_rate_row(self):
        with tempfile.TemporaryDirectory() as d:
            p = self.write(d, "a.jsonl", "m1b")
            recs = [json.loads(l) for l in open(p)]
            extra = dict(recs[0], id="w02", bsb_mutants=3, bsb_killed=1)
            recs[0].update(bsb_mutants=5, bsb_killed=5)
            with open(p, "w") as f:
                f.write("".join(json.dumps(r) + "\n" for r in [recs[0], extra, recs[1]]))
            self.assertIn("| 명세 변이체 판별률 | 6/8 (75.0%) |", harness.summarize([p]))


class Resume(unittest.TestCase):
    """An interrupted run continues to the file one uninterrupted run writes
    (backlog INF-16)."""

    def run_harness(self, out):
        subprocess.run([sys.executable, os.path.join(HERE, "harness.py"), "run", "--agent", "oracle", "--tasks",
                        "n01-callers,w02-abs-diff", "--out", out], check=True, capture_output=True)
        with open(out) as f:
            return [json.loads(line) for line in f]

    def test_resumed_run_has_the_rows_of_one_run(self):
        d = tempfile.mkdtemp()
        whole = self.run_harness(os.path.join(d, "whole.jsonl"))
        s = whole[-1]
        cut = os.path.join(d, "cut.jsonl")
        with open(cut, "w") as f:  # stopped after the first session
            head = {"partial": True, "agent": "oracle", "task_set": "m1", "manifest": s["manifest"],
                    "commit": s["commit"], "tasks": ["n01-callers", "w02-abs-diff"]}
            f.write(json.dumps(head) + "\n" + json.dumps(whole[0]) + "\n")
        resumed = self.run_harness(cut)
        self.assertEqual(resumed[:-1], whole[:-1])
        self.assertEqual(resumed[-1], {**s, "resumes": 1})


if __name__ == "__main__":
    # The harness leaves subprocess pipes to the garbage collector.
    unittest.main(warnings="ignore")
