#!/usr/bin/env python3
"""The utility report's tables (backlog T0-26): what they count, and that the
gate finds a stale block."""
import json
import os
import shutil
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import utility  # noqa: E402


def write(root, rel, rows):
    path = os.path.join(root, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")


class Tables(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="test-utility-")
        self.saved = utility.ROOT
        utility.ROOT = self.root
        brb = lambda arm, ok, lines, hidden: {  # noqa: E731
            "id": f"t-{ok}-{lines}", "arm": arm, "success": ok, "delivered": True, "final_check_ok": True,
            "failed": 3 if hidden else 0, "test_code": {"test_lines": lines}, "checks": 2, "total_tokens": 100,
            "tool_calls": 4, "tool_counts": {"read_file": 1, "write_file": 2, "cargo_test": 1}}
        write(self.root, "bench/brb/results/2026-11-01-m-rust.jsonl",
              [brb("rust", True, 10, False), brb("rust", False, 5, True),
               {"summary": True, "agent": "openai:m", "arm": "rust"}])
        write(self.root, "bench/m1b/results/2026-11-01-m.jsonl",
              [{"id": "w", "kind": "write", "success": True, "model_usage": {"input_tokens": 7, "output_tokens": 3},
                "tool_calls": 2},
               {"id": "n", "kind": "navigate", "success": True, "read_calls": 1, "read_bytes": 900,
                "model_usage": {"input_tokens": 50, "output_tokens": 5}},
               {"summary": True, "agent": "anthropic:m"}])
        write(self.root, "bench/m1/results/2026-11-01-m.jsonl", [{"id": "w", "success": True}])  # no summary

    def tearDown(self):
        utility.ROOT = self.saved
        shutil.rmtree(self.root, ignore_errors=True)

    def test_counts_come_from_the_rows(self):
        t = utility.tables()
        self.assertIn("| m1b | bmb | m | 1/1 | 10 | 10 | 2 |", t)
        self.assertIn("| m | rust | 1/2 | 10 | 10.0 | 2 | 1/2 |", t)  # test lines of successful sessions only
        self.assertIn("| m1b | m | 1/1 | 1 | 900 | 55 |", t)
        self.assertIn("| brb | rust | m | 2 | 4 | 2 | 25% |", t)
        self.assertIn("결과 파일 2개", t)  # the file without a summary row is not a finished run

    def test_a_stale_block_fails_the_check(self):
        report = os.path.join(self.root, "r.md")
        with open(report, "w") as f:
            f.write(f"# r\n\n{utility.BEGIN}\nold\n{utility.END}\n")
        self.assertEqual(utility.main(["--check", report]), 1)
        self.assertEqual(utility.main(["--write", report]), 0)
        self.assertEqual(utility.main(["--check", report]), 0)
        with open(report) as f:
            self.assertTrue(f.read().startswith("# r\n\n" + utility.BEGIN))



class Distribution(unittest.TestCase):
    """Table F (backlog T0-35): median, top-3 share, the sessions with an
    undecided response from the session records, and the cache-read share."""

    def test_shares_from_rows_and_session_records(self):
        import gzip
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "2026-11-04-m-bmb.jsonl")
            sessions = path[:-len(".jsonl")] + ".sessions"
            os.makedirs(sessions)
            for sid, text in (("a", "solver could not decide (rlimit)"), ("b", "ok"), ("c", "ok"), ("d", "ok")):
                with gzip.open(os.path.join(sessions, f"{sid}.json.gz"), "wt") as f:
                    json.dump({"id": sid, "calls": [{"text": text}]}, f)
            rows = [{"id": "a", "total_tokens": 70, "model_usage": {"input_tokens": 60, "cache_read_input_tokens": 30}},
                    {"id": "b", "total_tokens": 10, "model_usage": {"input_tokens": 10, "cache_read_input_tokens": 0}},
                    {"id": "c", "total_tokens": 10, "model_usage": {"input_tokens": 10, "cache_read_input_tokens": 0}},
                    {"id": "d", "total_tokens": 10, "model_usage": {"input_tokens": 10, "cache_read_input_tokens": 0}}]
            lines = utility.distribution_table([("brb-write", "m", "", {}, rows)], [path])
            self.assertEqual(lines[2], "| brb-write | bmb | m | 4 | 10 | 90.0% | 1/4 | 70.0% | 33.3% |")


if __name__ == "__main__":
    unittest.main()
