#!/usr/bin/env python3
"""Rules of bench/brb/oracle_strength.py (ADR-0119): the mutant swap, the
paired statistics and the registered verdict."""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import oracle_strength as o  # noqa: E402


def row(tid, killed, classes=None):
    return {"id": tid, "arm": "x", "killed": killed, "classes": classes or ["output"] * len(killed)}


class Swap(unittest.TestCase):
    def test_the_mutant_takes_the_name_and_the_tests_stay(self):
        lib = ("pub fn gcd(a: i64, b: i64) -> i64 { if b == 0 { a } else { gcd(b, a % b) } }\n"
               "fn helper() {}\n#[cfg(test)]\nmod tests { use super::*; #[test] fn t() { assert_eq!(gcd(4, 6), 2); } }\n")
        out = o.with_mutant(lib, "gcd", "pub fn gcd(a: i64, b: i64) -> i64 { a }\nfn helper() {}")
        self.assertIn("pub fn gcd__agent(a: i64, b: i64)", out)
        self.assertIn("gcd(b, a % b)", out)  # calls now reach the mutant
        self.assertIn("assert_eq!(gcd(4, 6), 2)", out)
        self.assertIn("mod brb_mutant {", out)  # its helper clashes with nothing
        self.assertIn("pub use brb_mutant::gcd;", out)
        self.assertNotIn("fn gcd_all", o.with_mutant("fn gcd_all() {}", "gcd", "fn gcd() {}").split("mod brb_mutant")[1])
        self.assertIn("fn gcd_all()", o.with_mutant("fn gcd_all() {}", "gcd", "fn gcd() {}"))


class Stats(unittest.TestCase):
    def test_rates_skip_unbuilt_mutants_and_split_by_class(self):
        r = row("t", [True, None, False, True], ["output", "output", "trap", "trap"])
        self.assertEqual(o.rate(r), (2, 3))
        self.assertEqual(o.rate(r, "output"), (1, 1))
        self.assertEqual(o.rate(r, "trap"), (1, 2))

    def test_pairs_are_tasks_both_arms_have(self):
        bmb = [row("a", [True, True]), row("b", [False]), {"id": "c", "arm": "bmb", "excluded": "x"}]
        rust = [row("a", [True, False]), row("c", [True])]
        self.assertEqual(o.paired(bmb, rust), [("a", 1.0, 0.5)])

    def test_sign_flip_p_is_exact(self):
        self.assertEqual(o.sign_flip_p([1.0, 1.0, 1.0]), 2 / 8)
        self.assertEqual(o.sign_flip_p([0.0, 0.0]), 1.0)
        self.assertAlmostEqual(o.sign_flip_p([0.5] * 10), 2 / 1024)

    def test_interval(self):
        m, lo, hi = o.interval([0.5, 0.5, 0.5, 0.5])
        self.assertEqual((m, lo, hi), (0.5, 0.5, 0.5))
        m, lo, hi = o.interval([0.0, 1.0])
        self.assertAlmostEqual(m, 0.5)
        self.assertLess(lo, 0)


class Verdict(unittest.TestCase):
    def pairs(self, b, u, n=12):
        return [(f"t{i}", b, u) for i in range(n)]

    def test_registered_rule(self):
        up = [(f"t{i}", 0.75 + 0.01 * (i % 3), 0.25) for i in range(12)]
        self.assertEqual(o.verdict(up, self.pairs(0.6, 0.5))[0], "성립")
        self.assertEqual(o.verdict(up, self.pairs(0.4, 0.5))[0], "판정 불가")  # the other model disagrees
        self.assertEqual(o.verdict(up)[0], "판정 불가")  # no other model
        down = [(t, u, b) for t, b, u in up]
        self.assertEqual(o.verdict(down, self.pairs(0.5, 0.5))[0], "반증")
        # the comparison arm kills nearly all: nothing to tell apart
        self.assertEqual(o.verdict(self.pairs(1.0, 0.95), self.pairs(1.0, 0.9))[0], "변별 불가")
        mixed = [("a", 1.0, 0.0), ("b", 0.0, 1.0), ("c", 0.5, 0.5)]
        self.assertEqual(o.verdict(mixed, mixed)[0], "판정 불가")
        self.assertEqual(o.verdict([], [])[0], "판정 불가")


if __name__ == "__main__":
    unittest.main()
