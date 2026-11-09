#!/usr/bin/env python3
"""BRB-HARD task set and harness (backlog T0-08). No network, no model.

The reference and weak runs and the mutant adequacy are the gate's
`bench brb-hard` jobs; here are the parts they do not show: the frozen set,
the statements both arms get, the cases, and the source of every task.
"""
import os
import sys
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.dont_write_bytecode = True
sys.path.insert(0, HERE)
import harness  # noqa: E402

judge = harness.judge
TASKS = {t.ID: t for t in harness.tasks()}


class TaskSet(unittest.TestCase):
    def test_twelve_tasks_frozen(self):
        self.assertEqual(len(TASKS), 12)
        self.assertEqual(sorted(TASKS)[0], "h01-first-in-sorted")
        harness.check_manifest()

    def test_every_task_names_its_quixbugs_source(self):
        for t in TASKS.values():
            path = os.path.join(HERE, "tasks", os.path.basename(t.__file__))
            with open(path) as f:
                self.assertTrue(f.readline().startswith("# QuixBugs `"), t.ID)
            self.assertEqual(len(t.RUST_MUTANTS), 4, t.ID)

    def test_both_arms_get_the_same_statement(self):
        for t in TASKS.values():
            b, r = judge.task_prompt(t, "bmb"), judge.task_prompt(t, "rust")
            self.assertEqual(b.split("Required signature")[0], r.split("Required signature")[0], t.ID)

    def test_hidden_cases_are_deterministic_and_in_domain(self):
        total = 0
        for t in TASKS.values():
            cs = judge.hidden_cases(t)
            self.assertGreater(len(cs), 300, t.ID)
            self.assertEqual(cs, judge.hidden_cases(t), t.ID)
            for args in cs:
                self.assertTrue(t.domain(*[list(a) if isinstance(a, list) else a for a in args]), t.ID)
            total += len(cs)
        self.assertEqual(total, 6845)

    def test_the_bmb_prompt_differs_from_the_write_set_by_one_rule(self):
        with open(os.path.join(HERE, "prompt-bmb.md")) as f:
            mine = f.read().splitlines()
        with open(os.path.join(harness.WRITE, "prompt-bmb.md")) as f:
            theirs = f.read().splitlines()
        self.assertEqual(sorted(set(mine) - set(theirs)), sorted([
            "- No `return`, `for`, `loop`, `match`, `break`, `continue`, `?`, `.unwrap()`.",
            "  Use `if`/`else` and `while`; a block's last expression is its value.",
            "- `x as T` converts between integer types; the value must provably fit in `T`",
            "  (for example `i as usize` needs `i >= 0`).",
        ]))


if __name__ == "__main__":
    unittest.main()
