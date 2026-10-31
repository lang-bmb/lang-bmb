#!/usr/bin/env python3
"""BRB-ENCAP task set, judge and harness logic (backlog T0-09). No network, no
model. Needs `bmb` ($BMB or target/debug/bmb), clang and cargo (offline).

The frozen tasks and seeds are only read; sessions run on temporary
workspaces. The full reference, weak and gaming runs are the gate's
`bench brb-encap` jobs; here are the parts they do not show.
"""
import json
import os
import shutil
import subprocess
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
SSET = TASKS["e01-sorted-set"]
MIN, MAX = -(1 << 63), (1 << 63) - 1


def session(task, arm, calls):
    """One session of `task` with a scripted agent playing `calls`."""
    agent = harness.m1.ScriptAgent(calls)
    target = tempfile.mkdtemp(prefix="brb-encap-test-target-")
    try:
        with mock.patch.object(harness, "make_agent", lambda kind, task, arm: agent):
            return harness.run_session(task, arm, "scripted", target, False)
    finally:
        shutil.rmtree(target, ignore_errors=True)


def put(task, code):
    return harness.m1.ops_to_calls([{"op": "put", "unit": task.UNIT, "code": code}])


def sset_records(script, corrupt=None):
    """Driver tokens for `script` on a correct sorted set, raw fields from the
    model; `corrupt(t, rec)` may change record `t`."""
    st, toks = SSET.init(), []
    for t, (op, a, b) in enumerate(script):
        rec = list(SSET.step(st, op, a, b)) + [len(st)] + st + [0] * (8 - len(st))
        if corrupt:
            rec = corrupt(t, rec)
        toks += [str(x) for x in rec]
    return toks


class TaskSet(unittest.TestCase):
    def test_four_structures_frozen(self):
        self.assertEqual(sorted(TASKS), ["e01-sorted-set", "e02-ring", "e03-intervals", "e04-min-heap"])
        harness.check_manifest()
        files = harness.frozen_files()
        for t in TASKS.values():
            for f in judge.seed_files(t, "bmb"):
                self.assertIn(f"seed/{t.SEED}/{os.path.basename(f)}", files)
            self.assertIn(f"seed/{t.SEED}/lib.rs", files)
            self.assertIn(f"seed/{t.SEED}/seed.rs", files)
        self.assertIn("../brb/MANIFEST", files)

    def test_both_arms_get_the_same_statement(self):
        for t in TASKS.values():
            b, r = judge.task_prompt(t, "bmb"), judge.task_prompt(t, "rust")
            self.assertTrue(b.startswith(f"Change the {t.KIND}. {t.STATEMENT}\n\n"))
            self.assertTrue(r.startswith(f"Change the {t.KIND}. {t.STATEMENT}\n\n"))
            self.assertIn(t.SIG_BMB, b)
            self.assertIn(t.SIG_RUST, r)
            self.assertNotIn(t.SIG_RUST, b)

    def test_hidden_scripts_are_deterministic_and_in_range(self):
        for t in TASKS.values():
            cs = judge.hidden_cases(t)
            self.assertEqual(cs, judge.hidden_cases(t))
            self.assertGreaterEqual(len(cs), 300)
            ops = {op[0]: op for op in t.OPS}
            target = 0
            for script in cs:
                self.assertTrue(1 <= len(script) <= 50)
                for op, a, b in script:
                    self.assertIn(op, ops)
                    for v, ty in zip((a, b), ops[op][1] + ["i64", "i64"]):
                        self.assertTrue(MIN <= v <= MAX)
                        if ty == "usize":
                            self.assertGreaterEqual(v, 0)
                    target += op == t.NAME
            # The operation under test is a large share of the calls.
            self.assertGreater(target, 1000, t.ID)

    def test_scripts_reach_the_hard_states(self):
        """Full structures and the operation's edge cases occur."""
        full, edges = {}, {}
        for t in TASKS.values():
            for script in judge.hidden_cases(t):
                st = t.init()
                for op, a, b in script:
                    before = len(st)
                    t.step(st, op, a, b)
                    full[t.ID] = full.get(t.ID, 0) + (op == t.NAME and before == 8)
                    edges[t.ID] = edges.get(t.ID, 0) + (op == t.NAME and a in (MIN, MAX))
        for tid in TASKS:
            self.assertGreater(full[tid], 50, tid)
        for tid in ("e01-sorted-set", "e03-intervals"):
            self.assertGreater(edges[tid], 5, tid)

    def test_weak_and_reference_differ_and_mutants_include_the_weak(self):
        for t in TASKS.values():
            self.assertNotEqual(t.BMB_REF, t.BMB_WEAK)
            self.assertNotEqual(t.RUST_REF, t.RUST_WEAK)
            self.assertIn(t.RUST_WEAK, t.RUST_MUTANTS)
            self.assertGreaterEqual(len(t.RUST_MUTANTS), 4)
            if t.BMB_GAME:
                # The gaming control redefines the type's invariant predicate.
                src = judge.seed_files(t, "bmb")[f"src/{t.UNIT}.bmb"]
                pred = src.split("#[invariant(", 1)[1].split(")", 1)[0]
                self.assertTrue(t.BMB_GAME.startswith(f"spec fn {pred}("), t.ID)


class Scoring(unittest.TestCase):
    SCRIPT = [("insert", 5, 0), ("insert", 3, 0), ("replace", 3, 9), ("at", 1, 0), ("size", 0, 0)]

    def kind(self, toks):
        kind, viol, _ = judge.score_case(SSET, self.SCRIPT, ("ok", toks))
        return kind, viol

    def test_a_correct_run_passes(self):
        self.assertEqual(self.kind(sset_records(self.SCRIPT)), ("pass", None))

    def test_a_broken_invariant_is_a_violation_even_with_right_results(self):
        def swap(t, rec):
            return rec[:3] + [rec[4], rec[3]] + rec[5:] if t == 2 else rec
        self.assertEqual(self.kind(sset_records(self.SCRIPT, swap)), ("violation", 2))

    def test_a_violation_after_a_wrong_result_is_still_counted(self):
        def bad(t, rec):
            if t == 1:
                return [0] + rec[1:]
            if t == 3:
                return rec[:3] + [9, 5] + rec[5:]
            return rec
        self.assertEqual(self.kind(sset_records(self.SCRIPT, bad)), ("wrong", 3))

    def test_a_wrong_value_with_the_invariant_kept_is_wrong(self):
        def other(t, rec):
            return rec[:4] + [10] + rec[5:] if t == 2 else rec
        self.assertEqual(self.kind(sset_records(self.SCRIPT, other)), ("wrong", None))

    def test_a_panic_or_a_cut_run_fails(self):
        toks = sset_records(self.SCRIPT)[:2 * judge.width(SSET)]
        self.assertEqual(self.kind(toks + ["panic"]), ("panic", None))
        self.assertEqual(self.kind(toks), ("crash", None))
        self.assertEqual(judge.score_case(SSET, self.SCRIPT, ("timeout",))[0], "timeout")

    def test_score_counts_violations_and_kinds(self):
        cs = [self.SCRIPT, self.SCRIPT]
        got = {0: ("ok", sset_records(self.SCRIPT)), 1: ("crash",)}
        s = judge.score(SSET, cs, got)
        self.assertEqual((s["passed"], s["failed"], s["fail_kinds"], s["violation_cases"]),
                         (1, 1, {"crash": 1}, 0))


class Verdict(unittest.TestCase):
    def test_h_encap_line(self):
        v = harness.encap_verdict
        self.assertIn("판정 불가(자기 검사를 통과한 과제가 없는 비교군)", v((0, 0, 0), (2, 4, 0)))
        self.assertIn("→ 반증(BMB 위반 1건 중 불변식 변경 0건)", v((1, 4, 0), (3, 4, 0)))
        self.assertIn("→ 반증(BMB 위반 2건 중 불변식 변경 2건)", v((2, 3, 2), (0, 4, 0)))
        self.assertIn("→ 판정 불가(Rust 통과 & 위반 0)", v((0, 4, 0), (0, 4, 0)))
        self.assertIn("BMB 0/4 = 0.000, Rust 3/4 = 0.750 → 지지", v((0, 4, 0), (3, 4, 0)))

    def row(self, tid, arm, **kw):
        r = {"id": tid, "arm": arm, "success": False, "delivered": True, "failed": 0, "cases": 10,
             "fail_kinds": {}, "final_check_ok": True, "violation_cases": 0, "total_tokens": 0,
             "tool_calls": 4, "checks": 1, "limit_hit": None, "session_seconds": 1.0, "test_code": None}
        r.update(kw)
        return r

    def test_summarize_table_and_line(self):
        tmp = tempfile.mkdtemp()
        try:
            b, r = os.path.join(tmp, "b.jsonl"), os.path.join(tmp, "r.jsonl")
            with open(b, "w") as f:
                f.write(json.dumps({"summary": True, "arm": "bmb", "agent": "x", "manifest": "m"}) + "\n")
                f.write(json.dumps(self.row("e01", "bmb", success=True, invariant_kept=True)) + "\n")
                f.write(json.dumps(self.row("e02", "bmb", final_check_ok=False, delivered=False, why="not verified",
                                            failed=10, fail_kinds={"not delivered": 10}, violation_cases=4,
                                            invariant_rejected=True, invariant_kept=True)) + "\n")
            with open(r, "w") as f:
                f.write(json.dumps({"summary": True, "arm": "rust", "agent": "x", "manifest": "m"}) + "\n")
                f.write(json.dumps(self.row("e01", "rust", success=True)) + "\n")
                f.write(json.dumps(self.row("e02", "rust", failed=3, fail_kinds={"violation": 3},
                                            violation_cases=3)) + "\n")
            table = harness.summarize([b, r])
            self.assertIn("| 자기 검사 통과 & 숨은 불변식 위반 / 자기 검사 통과 | 0/1 | 1/2 |", table)
            self.assertIn("| 게이트 거부: 불변식 경계(BMB, E0823·E0811) | 1 | — |", table)
            self.assertIn("BMB 0/1 = 0.000, Rust 1/2 = 0.500 → 지지", table)
            self.assertIn("| e02 | 미제출: not verified (검사 실패, 위반 4) | 결함: 3/10 (violation 3) (검사 통과) |", table)
            with open(r) as f:
                text = f.read().replace('"manifest": "m"', '"manifest": "other"')
            with open(r, "w") as f:
                f.write(text)
            with self.assertRaises(SystemExit):
                harness.summarize([b, r])
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_expect_conditions(self):
        weak_rust = self.row("e01", "rust", last_check_ok=True, violation_cases=5)
        self.assertEqual(harness.expect_failures("weak-caught", "rust", [weak_rust]), [])
        self.assertTrue(harness.expect_failures("weak-caught", "rust", [{**weak_rust, "violation_cases": 0}]))
        weak_bmb = self.row("e01", "bmb", last_check_ok=False, final_check_ok=False, invariant_rejected=True,
                            violation_cases=5, invariant_kept=True)
        self.assertEqual(harness.expect_failures("weak-caught", "bmb", [weak_bmb]), [])
        self.assertTrue(harness.expect_failures("weak-caught", "bmb", [{**weak_bmb, "invariant_rejected": False}]))
        game = self.row("e01", "bmb", invariant_kept=False, violation_cases=5)
        self.assertEqual(harness.expect_failures("gamed", "bmb", [game]), [])
        self.assertTrue(harness.expect_failures("gamed", "bmb", [{**game, "invariant_kept": True}]))
        self.assertEqual(harness.expect_failures("all-pass", "bmb", []), ["no results"])


class BmbJudge(unittest.TestCase):
    def test_seeds_verify_and_need_their_invariant(self):
        """Every seed is proved as given; its unit never repeats the
        invariant as a contract (the type carries it)."""
        bmb = harness.brb.bmb_bin()
        for t in TASKS.values():
            tmp = tempfile.mkdtemp()
            try:
                judge.write_files(tmp, judge.seed_files(t, "bmb"))
                r = subprocess.run([bmb, "verify", tmp], capture_output=True, text=True)
                self.assertEqual(r.returncode, 0, t.ID + r.stdout[-500:])
                src = judge.seed_files(t, "bmb")[f"src/{t.UNIT}.bmb"]
                self.assertIn(f"#[invariant(", src)
                self.assertNotIn("requires", src)
            finally:
                shutil.rmtree(tmp, ignore_errors=True)

    def test_a_stronger_requires_is_a_domain_violation(self):
        code = """spec fn differ(from: i64, to: i64) -> bool { from != to }

#[effects(pure)]
#[uses(differ)]
fn replace(s: &mut Set, from: i64, to: i64) -> bool
    requires differ(from, to),
{
    if !Set::contains(s, from) || Set::contains(s, to) {
        return false;
    }
    let taken: bool = Set::remove(s, from);
    let added: bool = Set::insert(s, to);
    true
}"""
        r = session(SSET, "bmb", put(SSET, code))
        self.assertTrue(r["final_check_ok"])
        self.assertTrue(r["delivered"])
        self.assertIs(r["domain_ok"], False)
        self.assertFalse(r["success"])

    def test_another_signature_is_not_delivered(self):
        code = """#[effects(pure)]
fn replace(s: &mut Set, from: i64) -> bool {
    Set::remove(s, from)
}"""
        r = session(SSET, "bmb", put(SSET, code))
        self.assertTrue(r["final_check_ok"])
        self.assertFalse(r["delivered"])
        self.assertEqual(r["why"], "target missing or wrong signature")

    def test_renaming_the_predicate_keeps_the_invariant(self):
        """The type's identity follows the predicate's body, not its name
        (ADR-0059): only a changed body counts as a changed invariant."""
        bmb = harness.brb.bmb_bin()
        tmp = tempfile.mkdtemp()
        try:
            ws = os.path.join(tmp, "ws")
            judge.write_files(ws, judge.seed_files(SSET, "bmb"))
            ops = os.path.join(tmp, "ops.json")
            with open(ops, "w") as f:
                json.dump([{"op": "rename", "def": "sset::sorted", "to": "ascending"}], f)
            subprocess.run([bmb, "apply", ws, ops], check=True, capture_output=True)

            def run(*a):
                p = subprocess.run([bmb, *a], capture_output=True, text=True)
                return p.returncode, p.stdout

            self.assertEqual(judge.type_id(run, [], ws, SSET), judge.seed_type_id(run, SSET, tmp))
            with open(ops, "w") as f:
                json.dump([{"op": "put", "unit": "sset", "code": "spec fn ascending(s: &Set) -> bool { s.len <= 8 }"}], f)
            subprocess.run([bmb, "apply", ws, ops], check=True, capture_output=True)
            self.assertNotEqual(judge.type_id(run, [], ws, SSET), judge.seed_type_id(run, SSET, tmp))
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


class RustJudge(unittest.TestCase):
    def test_changed_fields_are_not_delivered(self):
        lib = judge.seed_files(SSET, "rust")["src/lib.rs"].replace("[i64; 8]", "[i64; 9]").replace("[0; 8]", "[0; 9]")
        r = session(SSET, "rust", [("write_file", {"path": "src/lib.rs", "content": lib}), ("cargo_test", {})])
        self.assertTrue(r["final_check_ok"])
        self.assertFalse(r["delivered"])
        self.assertTrue(r["why"].startswith("judge build"))

    def test_the_probe_reads_private_fields_of_the_crate_root(self):
        probe = judge.probe_module(SSET)
        self.assertIn("pub fn raw(s: &super::SortedSet) -> Vec<i64>", probe)
        self.assertIn("let items: &[i64; 8] = &s.items;", probe)
        self.assertIn("let len: usize = s.len;", probe)


if __name__ == "__main__":
    unittest.main()
