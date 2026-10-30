"""Tests of bench/run_models.py beyond the dry run the gate does (backlog INF-14)."""
import json
import os
import sys
import tempfile
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


class Resume(unittest.TestCase):
    """A rerun skips sets finished on this commit and continues partial ones
    (backlog INF-16)."""

    def setUp(self):
        self.root, self.saved = tempfile.mkdtemp(), run_models.ROOT
        run_models.ROOT = self.root
        os.makedirs(os.path.join(self.root, "bench/m1/results"))

    def tearDown(self):
        run_models.ROOT = self.saved

    def put(self, name, *rows):
        with open(os.path.join(self.root, "bench/m1/results", name), "w") as f:
            f.write("".join(json.dumps(r) + "\n" for r in rows))

    def test_earlier_run(self):
        a = "openai:qwen"
        self.put("2026-10-29-qwen.jsonl", {"id": "w01", "model_usage": {"input_tokens": 5}},
                 {"summary": True, "agent": a, "commit": "old"})
        self.put("2026-10-30-qwen.jsonl", {"partial": True, "agent": a, "commit": "c"}, {"id": "w01"})
        self.put("2026-10-30-qwen-bmb.jsonl", {"summary": True, "agent": a, "commit": "c"})
        self.assertEqual(run_models.earlier_run("bench/m1/results", "qwen", "", a, "c"),
                         ("partial", os.path.join(self.root, "bench/m1/results/2026-10-30-qwen.jsonl")))
        self.assertEqual(run_models.earlier_run("bench/m1/results", "qwen", "-bmb", a, "c")[0], "finished")
        self.assertEqual(run_models.earlier_run("bench/m1/results", "qwen", "", a, "old")[0], "finished")
        self.assertEqual(run_models.earlier_run("bench/m1/results", "qwen", "", "anthropic:qwen", "c"),
                         (None, None))
        self.assertEqual(run_models.earlier_run("bench/nosuch/results", "qwen", "", a, "c"), (None, None))

    def test_usage_counts_sessions_only(self):
        self.put("p.jsonl", {"partial": True}, {"id": "a", "model_usage": {"input_tokens": 3, "output_tokens": 1}},
                 {"resumed": True}, {"id": "b"})
        self.assertEqual(run_models.usage_of(os.path.join(self.root, "bench/m1/results/p.jsonl")), (2, 3, 1))


if __name__ == "__main__":
    unittest.main()
