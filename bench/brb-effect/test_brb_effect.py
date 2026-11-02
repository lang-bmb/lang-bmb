#!/usr/bin/env python3
"""BRB-EFFECT task set, judge and harness logic (backlog T1-43). No network,
no model. Needs `bmb` ($BMB or target/debug/bmb), clang and cargo (offline).

The frozen tasks are only read; sessions run on temporary workspaces. The
full reference and weak runs and the mutant adequacy are the gate's
`bench brb-effect` jobs; here are the parts they do not show.
"""
import json
import os
import shutil
import sys
import tempfile
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.dont_write_bytecode = True
sys.path.insert(0, HERE)
import harness  # noqa: E402

judge = harness.judge
TASKS = {t.ID: t for t in harness.tasks()}
LEDGER = TASKS["f01-ledger"]
REPORT = TASKS["f02-report"]


def session(task, arm, calls):
    """One session of `task` with a scripted agent playing `calls`."""
    agent = harness.m1.ScriptAgent(calls)
    target = tempfile.mkdtemp(prefix="brb-effect-test-target-")
    try:
        with mock.patch.object(harness.brb, "make_agent", lambda kind, task, arm: agent):
            return harness.brb.run_session(task, arm, "scripted", target, False)
    finally:
        shutil.rmtree(target, ignore_errors=True)


def bmb_calls(code):
    return harness.m1.ops_to_calls([{"op": "unit", "path": "tasks", "summary": "BRB task"},
                                    {"op": "put", "unit": "tasks", "code": code}])


class TaskSet(unittest.TestCase):
    def test_four_tasks_frozen(self):
        self.assertEqual(sorted(TASKS), ["f01-ledger", "f02-report", "f03-limiter", "f04-rle"])
        harness.check_manifest()

    def test_both_arms_get_the_same_statement(self):
        for t in TASKS.values():
            b, r = judge.task_prompt(t, "bmb"), judge.task_prompt(t, "rust")
            self.assertEqual(b.split("Required signature")[0], r.split("Required signature")[0], t.ID)
            self.assertIn("print_i64", b)
            self.assertIn("println!", r)

    def test_hidden_cases_are_deterministic_and_in_domain(self):
        for t in TASKS.values():
            cs = judge.hidden_cases(t)
            self.assertGreater(len(cs), 300, t.ID)
            self.assertEqual(cs, judge.hidden_cases(t), t.ID)
            for args in cs:
                self.assertTrue(t.domain(*args), (t.ID, args))
            # every task prints in most cases
            printed = sum(1 for args in cs if t.model(*args)[1])
            self.assertGreater(printed, len(cs) // 2, t.ID)

    def test_weak_differs_from_the_reference(self):
        for t in TASKS.values():
            self.assertNotEqual(t.BMB_REF, t.BMB_WEAK, t.ID)
            self.assertNotEqual(t.RUST_REF, t.RUST_WEAK, t.ID)
            self.assertEqual(len(t.RUST_MUTANTS), 4, t.ID)


class Output(unittest.TestCase):
    def test_lines_before_a_header_belong_to_its_case(self):
        got, last = judge.parse_output("5\n-1\n0 ok 5\n1 ok 0\n7\n2 panic\n", 0, 3)
        self.assertEqual(got, {0: ("ok", ["5"], ["5", "-1"]), 1: ("ok", ["0"], []), 2: ("panic",)})
        self.assertEqual(last, 2)

    def test_a_header_must_name_the_next_case(self):
        # A printed line that looks like a later header is a printed line.
        got, last = judge.parse_output("2 ok\n0 ok 1\n", 0, 3)
        self.assertEqual(got, {0: ("ok", ["1"], ["2 ok"])})
        self.assertEqual(last, 0)

    def test_a_restarted_run_starts_at_its_case(self):
        got, last = judge.parse_output("4\n3 ok 9\n", 3, 5)
        self.assertEqual(got, {3: ("ok", ["9"], ["4"])})
        self.assertEqual(last, 3)

    def test_right_result_with_wrong_lines_is_wrong(self):
        args = ([1], [6], 5)  # a rejected withdrawal: balance 5, prints -1
        self.assertEqual(judge.expected(LEDGER, args), (["5"], ["-1"]))
        s = judge.score(LEDGER, [args], {0: ("ok", ["5"], ["5"])})
        self.assertEqual((s["failed"], s["wrong_output"], s["fail_kinds"]), (1, 1, {"wrong": 1}))
        s = judge.score(LEDGER, [args], {0: ("ok", ["5"], ["-1"])})
        self.assertEqual((s["failed"], s["wrong_output"]), (0, 0))
        s = judge.score(LEDGER, [args], {0: ("crash",)})
        self.assertEqual(s["fail_kinds"], {"crash": 1})

    def test_the_domain_caller_may_print(self):
        u = judge.domain_unit(LEDGER, "brb_domain", [])
        self.assertIn("#[effects(io)]", u)
        self.assertNotIn("#[effects(pure)]", u)


class Verdict(unittest.TestCase):
    @staticmethod
    def rows(arm, n, need=1, defs=5, lines=10, success=True):
        tc = {"exec_defs": defs, "test_needed_defs": need, "test_lines": lines} if arm == "bmb" else {
            "test_lines": lines}
        return [{"id": f"t{i}", "arm": arm, "success": success, "test_code": tc} for i in range(n)]

    def test_h_effect_line(self):
        v = harness.effect_verdict
        self.assertIn("확정 조건 충족", v(self.rows("bmb", 4, 1, 5, 10), self.rows("rust", 4, lines=30)))
        self.assertIn("반증(면적)", v(self.rows("bmb", 4, 2, 5, 10), self.rows("rust", 4, lines=30)))
        self.assertIn("반증(줄)", v(self.rows("bmb", 4, 1, 5, 11), self.rows("rust", 4, lines=30)))
        self.assertIn("반증(면적·줄)", v(self.rows("bmb", 4, 2, 5, 11), self.rows("rust", 4, lines=30)))
        self.assertIn("판정 불가(Rust 테스트 0줄)", v(self.rows("bmb", 4, 1, 5, 1), self.rows("rust", 4, lines=0)))
        self.assertIn("판정 불가(숨은 판정을 통과한 세션이 없는 비교군)",
                      v(self.rows("bmb", 4, success=False), self.rows("rust", 4)))
        # Only sessions that passed the hidden judge count.
        b = self.rows("bmb", 2, 1, 5, 10) + self.rows("bmb", 2, 5, 5, 99, success=False)
        self.assertIn("면적 2/10", v(b, self.rows("rust", 4, lines=30)))

    def test_summarize_table_and_line(self):
        tmp = tempfile.mkdtemp()
        try:
            paths = []
            for arm, rows in (("bmb", self.rows("bmb", 2, 1, 6, 8)), ("rust", self.rows("rust", 2, lines=40))):
                full = [dict(r, delivered=True, defect=False, cases=10, failed=0, fail_kinds={}, wrong_output=0,
                             final_check_ok=True, total_tokens=0, tool_calls=1, checks=1, limit_hit=None,
                             session_seconds=1.0) for r in rows]
                p = os.path.join(tmp, f"{arm}.jsonl")
                with open(p, "w") as f:
                    for r in full:
                        f.write(json.dumps(r) + "\n")
                    f.write(json.dumps({"summary": True, "arm": arm, "manifest": "m"}) + "\n")
                paths.append(p)
            table = harness.summarize(paths)
            self.assertIn("| 남은 테스트 면적(BMB 통과 세션: 테스트 필요 / 실행 정의) | 2/12 | — |", table)
            self.assertIn("면적 2/12 = 0.167", table)
            self.assertIn("줄 BMB 16 / Rust 80 = 0.200", table)
            self.assertIn("→ 이 모델에서 확정 조건 충족", table)
            self.assertIn("보조(판정에 쓰지 않음): 기준 해법 면적 4/19 = 0.211, 대비 0.79배", table)
        finally:
            shutil.rmtree(tmp)


class BmbJudge(unittest.TestCase):
    def test_a_stronger_requires_is_a_domain_violation(self):
        code = LEDGER.BMB_REF.replace("spec fn nonneg(b: i64) -> bool { b >= 0 }",
                                      "spec fn nonneg(b: i64) -> bool { b >= 0 }\n\n"
                                      "spec fn positive(b: i64) -> bool { b >= 1 }")
        code = code.replace("requires same_len(kinds, amounts), all_nonneg(amounts), nonneg(start),",
                            "requires same_len(kinds, amounts), all_nonneg(amounts), nonneg(start), positive(start),")
        code = code.replace("#[uses(nonneg, outcome, same_len, all_nonneg)]",
                            "#[uses(nonneg, outcome, same_len, all_nonneg, positive)]")
        r = session(LEDGER, "bmb", bmb_calls(code))
        self.assertTrue(r["final_check_ok"])
        self.assertIs(r["domain_ok"], False)
        self.assertFalse(r["success"])

    def test_one_function_that_prints_is_all_surface(self):
        # The whole report in the io function: every executable definition
        # needs a test.
        code = """#[effects(io)]
fn report(a: &[i64], t: i64) -> (r: i64) {
    if a.len() == 0 {
        0
    } else {
        let mut lo: i64 = a[0];
        let mut hi: i64 = a[0];
        let mut c: i64 = 0;
        let mut run: i64 = 0;
        let mut best: i64 = 0;
        let mut k: usize = 0;
        while k < a.len()
            invariant k <= a.len(), c <= k, run <= k, best <= k,
            decreases a.len() - k,
        {
            if a[k] < lo { lo = a[k]; }
            if a[k] > hi { hi = a[k]; }
            if a[k] > t { c = c + 1; run = run + 1; if run > best { best = run; } } else { run = 0; }
            k = k + 1;
        }
        print_i64(lo);
        print_i64(hi);
        let d: i64 = if (lo >= 0) == (hi >= 0) { hi - lo } else { 0 };
        if (lo >= 0) == (hi >= 0) {
            if d % 2 != 0 && d < 0 { print_i64(lo + d / 2 - 1); } else { print_i64(lo + d / 2); }
        } else {
            let s: i64 = lo + hi;
            if s % 2 != 0 && s < 0 { print_i64(s / 2 - 1); } else { print_i64(s / 2); }
        }
        print_i64(c);
        best
    }
}"""
        r = session(REPORT, "bmb", bmb_calls(code))
        self.assertTrue(r["final_check_ok"], r.get("why"))
        tc = r["test_code"]
        self.assertEqual((tc["test_needed_defs"], tc["exec_defs"]), (1, 1))

    def test_the_reference_splits_core_and_shell(self):
        r = session(LEDGER, "bmb", bmb_calls(LEDGER.BMB_REF))
        self.assertTrue(r["success"])
        tc = r["test_code"]
        self.assertEqual((tc["test_needed_defs"], tc["io_defs"], tc["exec_defs"]), (1, 1, 6))


if __name__ == "__main__":
    unittest.main()
