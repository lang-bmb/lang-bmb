#!/usr/bin/env python3
"""Tests for failures.py (backlog T0-16)."""
import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import failures  # noqa: E402

W = {"kind": "write", "success": False, "commits": 1, "verified": True, "signature_ok": True}


class Classify(unittest.TestCase):
    def test_ending(self):
        self.assertEqual(failures.ending({"stop": "api_error"}), "api_error")
        self.assertEqual(failures.ending({"stop": None, "limit_hit": True}), "limit")
        self.assertEqual(failures.ending({"stop": None, "limit_hit": "output_tokens"}), "limit")
        self.assertEqual(failures.ending({"stop": "length"}), "length")
        self.assertEqual(failures.ending({"stop": "max_tokens"}), "length")
        self.assertEqual(failures.ending({"stop": "stop"}), "stop")

    def test_verdict_is_the_first_check_not_passed(self):
        self.assertEqual(failures.verdict({**W, "commits": 0, "verified": False}), "no-commit")
        self.assertEqual(failures.verdict({**W, "verified": False}), "unverified")
        self.assertEqual(failures.verdict({**W, "signature_ok": False}), "signature")
        self.assertEqual(failures.verdict({**W, "given_intact": False}), "given-changed")
        self.assertEqual(failures.verdict({**W, "spec_rejections": [{"reason": "undecided"}]}), "spec-undecided")
        self.assertEqual(failures.verdict({**W, "spec_rejections": [{"reason": "refuted"}]}), "spec-refuted")
        self.assertEqual(failures.verdict({"kind": "navigate"}), "navigate")
        self.assertEqual(failures.verdict({"delivered": False, "why": "not verified"}), "not-delivered (not verified)")
        self.assertEqual(failures.verdict({"delivered": True, "failed": 2}), "hidden-cases")
        # Verus rows have no `commits`.
        self.assertEqual(failures.verdict({"kind": "write", "verified": False}), "unverified")

    def test_table(self):
        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as f:
            for r in ({"partial": True, "task_set": "bvb"}, {**W, "id": "a", "signature_ok": False, "limit_hit": True},
                      {**W, "id": "b", "success": True}):
                f.write(json.dumps(r) + "\n")
        text = failures.table(f.name)
        os.unlink(f.name)
        self.assertIn("## bvb: 실패 1/2", text)
        self.assertIn("| limit | 1 | 1 |", text)
        self.assertIn("| a | limit | signature |", text)


if __name__ == "__main__":
    unittest.main()
