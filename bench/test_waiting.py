#!/usr/bin/env python3
"""The measurement queue's table (backlog INF-39): estimates come only from
committed result files, and the committed queue names files that exist."""
import json
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import waiting as bench_queue  # noqa: E402


class Queue(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = self.tmp.name
        for name in ("a1", "a2", "b1"):
            open(os.path.join(self.root, name + ".jsonl"), "w").close()
        self.usage = lambda p: {"a1": (10, 100, 20), "a2": (10, 300, 40), "b1": (5, 50, 6)}[
            os.path.basename(p)[:-6]
        ]

    def tearDown(self):
        self.tmp.cleanup()

    def est(self, run):
        return bench_queue.estimate(run, self.root, self.usage)

    def test_mean_of_the_files_a_glob_matches(self):
        self.assertEqual(self.est({"what": "x", "like": ["a*.jsonl"]}), (10, 200, 30, 2))

    def test_sum_over_globs_when_a_run_covers_several_sets(self):
        run = {"what": "x", "like": ["a*.jsonl", "b*.jsonl"], "sum": True}
        self.assertEqual(self.est(run), (15, 250, 36, 3))
        with self.assertRaises(ValueError):
            self.est({"what": "x", "like": ["a*.jsonl", "b*.jsonl"]})

    def test_no_file_no_estimate(self):
        self.assertIsNone(self.est({"what": "x", "like": ["c*.jsonl"]}))
        self.assertIsNone(self.est({"what": "x"}))

    def test_table_totals_and_days(self):
        spec = {"entries": [
            {"task": "T-1", "tier": "first", "runs": [
                {"what": "set", "model": "m", "times": 3, "like": ["a*.jsonl"]},
                {"what": "cpu", "model": "-", "times": 1, "cpu": True},
                {"what": "new", "model": "m", "times": 2},
            ]},
        ]}
        rows = bench_queue.rows_of(spec, self.root, self.usage)
        text = bench_queue.table(rows, {"m": (1000000.0, 0.0)}, 250.0)
        self.assertIn("| 1 | `T-1` | set | m | 3 | 10 | 200 | 30 | 600 | 90 | 결과 파일 2개 |", text)
        self.assertIn("모델 없음", text)
        self.assertIn("지난 실행 없음", text)
        self.assertIn("| first | m | 5 | 600 | 90 | 2 | 600.00 |", text)
        self.assertIn("- first: 3일(누적 3일)", text)
        plain = bench_queue.table(rows, {}, None)
        self.assertNotIn("USD", plain)

    def test_the_committed_queue_reads(self):
        with open(os.path.join(HERE, "waiting.json"), encoding="utf-8") as f:
            spec = json.load(f)
        rows = bench_queue.rows_of(spec)
        self.assertTrue(rows)
        for r in rows:
            run = next(x for e in spec["entries"] for x in e["runs"] if x["what"] == r["what"] and x["model"] == r["model"])
            if run.get("like"):
                self.assertIsNotNone(r["est"], f"{r['task']} {r['what']}: no file matches")


if __name__ == "__main__":
    unittest.main()
