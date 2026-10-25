"""Tests of bench/run_models.py beyond the dry run the gate does (backlog INF-14)."""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run_models  # noqa: E402


class Guard(unittest.TestCase):
    def test_cost(self):
        self.assertAlmostEqual(run_models.cost(2_000_000, 500_000, 3.0, 15.0), 13.5)

    def test_projection_stops_before_a_set_over_budget(self):
        self.assertIsNone(run_models.over_budget(0.0, 0, 45, 1.0))  # no projection yet
        self.assertIsNone(run_models.over_budget(15.0, 15, 30, 50.0))  # 15 + 30 = 45 <= 50
        why = run_models.over_budget(15.0, 15, 40, 50.0)  # 15 + 40 = 55 > 50
        self.assertIn("예상 $40.00", why)

    def test_real_runs_need_prices_budget_and_whole_sets(self):
        for argv in (["--agent", "anthropic:m"],
                     ["--agent", "anthropic:m", "--price-in", "1", "--price-out", "1"],
                     ["--agent", "anthropic:m", "--price-in", "1", "--price-out", "1", "--budget", "5", "--limit", "2"],
                     ["--agent", "oracle", "--price-in", "1", "--price-out", "1", "--budget", "5"],
                     ["--dry-run", "--sets", "nosuch"]):
            with self.assertRaises(SystemExit, msg=argv) as cm:
                run_models.main(argv)
            self.assertNotEqual(cm.exception.code, 0, argv)

    def test_every_set_has_tasks(self):
        for name, ids, _, _ in run_models.SETS:
            self.assertTrue(ids(), name)
        self.assertEqual(len(run_models.brb_ids()), 24)


if __name__ == "__main__":
    unittest.main()
