#!/usr/bin/env python3
"""Tests of scripts/solver_stability.py (backlog 1B-40, T2-07)."""
import os
import random
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import solver_stability as st  # noqa: E402

SCRIPT = """; obligation 0: E0803 `+` may overflow `i64`
(reset)
(set-option :smt.random_seed 0)
(set-option :sat.random_seed 0)
(set-option :rlimit 5000000)
(define-fun bmb_abs ((x Int)) Int (ite (>= x 0) x (- x)))
(declare-const |x@0| Int)
(assert (and (<= 0 |x@0|) (<= |x@0| 10)))
(declare-const |unit::f.result@0| Int)
(assert (= |unit::f.result@0| (+ (bmb_abs |x@0|) 1)))
(assert (not (<= |unit::f.result@0| 11)))
(check-sat)
; obligation 1: E0805 index may be out of bounds
(reset)
(set-option :smt.random_seed 0)
(set-option :sat.random_seed 0)
(declare-const |i@0| Int)
(assert (< |i@0| 3))
(assert (not (< |i@0| 2)))
(check-sat)
"""


PROBE = "; probe of obligation {k}: E0805 index may be out of bounds\n(reset)\n(declare-const |p@0| Int)\n(check-sat)\n"
RETRY = "; retry of obligation {k} with ground instances\n(reset)\n(declare-const |r@0| Int)\n(check-sat)\n"


class Variants(unittest.TestCase):
    def setUp(self):
        self.obs = [st.commands(bodies[0]) for _, bodies in st.blocks(SCRIPT)]

    def test_blocks_and_commands(self):
        self.assertEqual([h for h, _ in st.blocks(SCRIPT)],
                         ["0: E0803 `+` may overflow `i64`", "1: E0805 index may be out of bounds"])
        self.assertEqual([len(b) for _, b in st.blocks(SCRIPT)], [1, 1])
        self.assertEqual(self.obs[0][0], "(reset)")
        self.assertEqual(self.obs[0][-1], "(check-sat)")

    def test_variants_change_the_text_and_keep_the_answer(self):
        base = st.solve(["\n".join(c) for c in self.obs])
        self.assertEqual(base, ["unsat", "sat"])
        for kind in st.KINDS:
            for k in range(3):
                rng = random.Random(f"{kind}{k}")
                vs = [st.variant(c, kind, rng) for c in self.obs]
                if kind != "shuffle":
                    self.assertNotEqual(vs[0], self.obs[0], kind)
                self.assertEqual(st.solve(["\n".join(v) for v in vs]), base, kind)

    def test_every_attempt_is_a_stage(self):
        # ADR-0048 §4: the exact query of a bit obligation follows its
        # abstracted attempt; ADR-0067: the probe comes just before the full
        # query (after the abstracted attempt); ADR-0023: a retry follows.
        # They are stages of one obligation, in that order (backlog T2-07).
        probe = PROBE.format(k=1)
        script = SCRIPT.replace("; obligation 1: ", probe + "; obligation 1: ")
        script += RETRY.format(k=1)
        blocks = st.blocks(script)
        self.assertEqual([h for h, _ in blocks], [h for h, _ in st.blocks(SCRIPT)])
        stages = [[st.commands(b) for b in bodies] for _, bodies in blocks]
        self.assertEqual(stages[0], [self.obs[0]])
        self.assertEqual(len(stages[1]), 3)
        self.assertEqual(stages[1][0][1], "(declare-const |p@0| Int)")
        self.assertEqual(stages[1][1], self.obs[1])
        self.assertEqual(stages[1][2][1], "(declare-const |r@0| Int)")
        bits = SCRIPT.replace("; obligation 1: E0805 index may be out of bounds\n",
                              "; obligation 1: E0805 index may be out of bounds (bit operations abstracted)\n")
        bits += probe + "; exact query of obligation 1\n(reset)\n(declare-const |e@0| Int)\n(check-sat)\n"
        stages = [[st.commands(b) for b in bodies] for _, bodies in st.blocks(bits)]
        self.assertEqual(stages[1][0], self.obs[1])
        self.assertEqual([c[1] for c in stages[1][1:]], ["(declare-const |p@0| Int)", "(declare-const |e@0| Int)"])

    def test_the_verdict_is_any_stage_unsat(self):
        # An obligation whose first stage does not decide and whose second
        # proves it is proved, unchanged and under every variant: stable.
        undecided = "(reset)\n(declare-const |y@0| Int)\n(assert (> |y@0| 0))\n(check-sat)"
        obs = [[st.commands(undecided), self.obs[0]], [self.obs[0]], [self.obs[1]]]
        base, changed = st.unstable_obligations("p", "f.smt2", obs, 2)
        self.assertEqual(base, [["sat", "unsat"], ["unsat"], ["sat"]])
        self.assertTrue(st.proved(base[0]))
        self.assertFalse(st.proved(base[2]))
        self.assertEqual(changed, {})

    def test_shuffle_declares_before_use(self):
        v = st.variant(self.obs[0], "shuffle", random.Random(1))
        first_assert = next(i for i, c in enumerate(v) if c.startswith("(assert"))
        self.assertTrue(all(not c.startswith(("(declare", "(define")) for c in v[first_assert:]))
        self.assertEqual(sorted(v), sorted(self.obs[0]))

    def test_rename_is_a_bijection(self):
        v = st.variant(self.obs[0], "rename", random.Random(2))
        text = "\n".join(v)
        self.assertNotIn("|x@0|", text)
        self.assertNotIn("bmb_abs", text)
        self.assertEqual(len(set(st.SYM.findall(text))), len(set(st.SYM.findall("\n".join(self.obs[0])))))


if __name__ == "__main__":
    unittest.main()
