#!/usr/bin/env python3
"""Tests for condition.py (ADR-0070): the probe sees what the model would see,
and a harness change the model would see is found."""
import json
import os
import shutil
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import condition  # noqa: E402

BMB = os.environ.get("BMB", os.path.join(condition.ROOT, "target", "debug", "bmb"))
TASK = "w01-clamp"


def copy_tree(dst):
    """The parts of the repository the m1 harness reads."""
    shutil.copytree(os.path.join(condition.ROOT, "bench", "m1"), os.path.join(dst, "bench", "m1"),
                    ignore=shutil.ignore_patterns("results", "__pycache__"))
    return dst


class Probe(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.mkdtemp(prefix="test-condition-")

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def run_probe(self, tree, provider, script, label):
        out = os.path.join(self.tmp, f"{label}-{provider}-{script}.jsonl")
        condition.run_probe(tree, "m1", provider, script, out, BMB, TASK)
        return out

    def test_the_probe_records_every_request_and_the_stress_script_hits_the_limit(self):
        out = self.run_probe(condition.ROOT, "openai", "stress", "rec")
        with open(out + ".bodies.json") as f:
            bodies = json.load(f)[TASK]
        row = condition.result_rows(out)[TASK]
        self.assertTrue(row["limit_hit"])
        self.assertEqual(len(bodies), row["model_usage"]["requests"])
        # The unknown tool's error reaches the model through the adapter.
        tool_msgs = [m for m in bodies[-1]["messages"] if m["role"] == "tool"]
        self.assertTrue(any(m["content"].startswith("error: ") for m in tool_msgs))

    def test_the_same_harness_is_the_same_condition(self):
        for provider in condition.PROVIDERS:
            a = self.run_probe(condition.ROOT, provider, "oracle", "a")
            b = self.run_probe(condition.ROOT, provider, "oracle", "b")
            self.assertEqual(condition.compare(a, condition.ROOT, b, condition.ROOT), [])

    def test_a_change_the_model_sees_is_found(self):
        tree = copy_tree(os.path.join(self.tmp, "changed"))
        path = os.path.join(tree, "bench", "m1", "model_api.py")
        with open(path) as f:
            src = f.read()
        self.assertIn('("error: " if err else "")', src)
        with open(path, "w") as f:
            f.write(src.replace('("error: " if err else "")', '("Error: " if err else "")'))
        a = self.run_probe(condition.ROOT, "openai", "stress", "base")
        b = self.run_probe(tree, "openai", "stress", "changed")
        diffs = condition.compare(a, condition.ROOT, b, tree)
        self.assertTrue(any("differs in ['messages']" in d for d in diffs), diffs)

    def rewrite(self, src, label, change):
        rows = condition.result_rows(src)
        change(rows[TASK])
        out = os.path.join(self.tmp, label + ".jsonl")
        with open(out, "w") as f:
            f.write("\n".join(json.dumps(r) for r in rows.values()) + "\n")
        shutil.copy(src + ".bodies.json", out + ".bodies.json")
        return out

    def test_a_changed_or_missing_verdict_field_is_found(self):
        a = self.run_probe(condition.ROOT, "openai", "oracle", "v0")
        b = self.rewrite(a, "v1", lambda r: r.update(success=not r["success"]))
        c = self.rewrite(a, "v2", lambda r: r.pop("spec_accepted"))
        self.assertTrue(any("field success" in d for d in condition.compare(a, condition.ROOT, b, condition.ROOT)))
        self.assertTrue(any("spec_accepted missing" in d for d in condition.compare(a, condition.ROOT, c, condition.ROOT)))

    def test_new_fields_and_timings_are_allowed(self):
        a = self.run_probe(condition.ROOT, "openai", "oracle", "n0")
        b = self.rewrite(a, "n1", lambda r: r.update(session_record="x", judge_seconds=1.0))
        self.assertEqual(condition.compare(a, condition.ROOT, b, condition.ROOT), [])

    def test_cargo_times_are_not_part_of_the_condition(self):
        a = "    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s\n; finished in 0.00s"
        b = "    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.31s\n; finished in 12ms"
        self.assertEqual(condition.normalize(a, "/t"), condition.normalize(b, "/t"))
        self.assertNotEqual(condition.normalize("in 3 tests", "/t"), condition.normalize("in 4 tests", "/t"))


if __name__ == "__main__":
    unittest.main()
