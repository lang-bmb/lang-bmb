"""BVB task set (backlog 2A-08): frozen inputs, selection record, BMB judging extensions.

Needs the `bmb` binary like bench/m1/test_harness.py. No network, no Verus.
"""
import copy
import json
import os
import re
import sys
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "bench", "m1"))
sys.dont_write_bytecode = True
import harness  # noqa: E402  (bench/m1)
import port  # noqa: E402
import sources  # noqa: E402

RULES = {"T1", "T2", "T3", "T4", "T5", "T6", "T7", "G1"}


def tasks():
    out = {}
    for f in sorted(os.listdir(os.path.join(HERE, "tasks"))):
        with open(os.path.join(HERE, "tasks", f)) as fh:
            out[f[:-5]] = json.load(fh)
    return out


class BvbSet(unittest.TestCase):
    def setUp(self):
        self.saved = harness.SET
        harness.SET = HERE

    def tearDown(self):
        harness.SET = self.saved


class Frozen(BvbSet):
    def test_manifest(self):
        harness.check_manifest()
        self.assertEqual(harness.set_name(), "bvb")

    def test_tasks_are_built_from_port_sources(self):
        self.assertEqual(set(port.rendered()), {t + ".json" for t in tasks()})
        for name, text in port.rendered().items():
            with open(os.path.join(HERE, "tasks", name)) as f:
                self.assertEqual(f.read(), text, name)

    def test_prompt_is_m1_prompt(self):
        # Same agent instructions as m1/m1b (ADR-0025 §1): only tasks and seed differ.
        with open(os.path.join(HERE, "prompt.md"), "rb") as a, \
                open(os.path.join(ROOT, "bench", "m1", "prompt.md"), "rb") as b:
            self.assertEqual(a.read(), b.read())

    def test_task_shape(self):
        for tid, t in tasks().items():
            self.assertEqual(t["kind"], "write", tid)
            self.assertTrue(set(t["source"]["rules"]) <= RULES, tid)
            self.assertIn(t["source"]["benchmark"], ("Verus-Bench", "Vericoding"), tid)
            # Hidden names cannot clash with the agent's: they live in the hidden unit.
            for name in re.findall(r"spec fn (\w+)", t["accept"]["specs"]):
                self.assertTrue(name.startswith("ref_"), (tid, name))
            self.assertIn(t["verus"]["reference_origin"],
                          ("upstream", "upstream-triggers", "upstream-migrated", "bvb"), tid)
            self.assertEqual("frozen" in t, "given" in t["verus"], tid)


class Selection(unittest.TestCase):
    def test_every_verus_bench_task_has_one_decision(self):
        self.assertEqual(len(sources.VERUS_BENCH), 150)
        for k, d in sources.VERUS_BENCH.items():
            self.assertTrue(d in sources.REASONS or d in tasks(), (k, d))

    def test_vericoding_sample_stops_at_the_target(self):
        chosen = [d for _, d in sources.VERICODING if d not in sources.REASONS]
        self.assertEqual(len(chosen), sources.VERICODING_TARGET)
        self.assertNotIn(sources.VERICODING[-1][1], sources.REASONS)
        keys = [sources.sample_key(i) for i, _ in sources.VERICODING]
        self.assertEqual(keys, sorted(keys))

    def test_selected_tasks_are_the_task_set(self):
        chosen = [d for d in sources.VERUS_BENCH.values() if d not in sources.REASONS]
        chosen += [d for _, d in sources.VERICODING if d not in sources.REASONS]
        self.assertEqual(sorted(chosen), sorted(tasks()))

    def test_exclusions_are_explained(self):
        for k, d in list(sources.VERUS_BENCH.items()) + sources.VERICODING:
            if d == "dup":
                self.assertIn(k, sources.DUP_OF)
            if d == "unsat":
                self.assertIn(k, sources.UNSAT_WITNESS)

    def test_sources_md_shows_the_record(self):
        with open(os.path.join(HERE, "SOURCES.md")) as f:
            self.assertIn(sources.table(), f.read())

    def test_seed_holds_only_given_definitions_in_use(self):
        with open(os.path.join(HERE, "seed", "src", "given.bmb")) as f:
            seed = {"given::" + n for n in re.findall(r"spec fn (\w+)", f.read())}
        used = {p for t in tasks().values() for p in t.get("frozen", [])}
        self.assertEqual(seed, used)

    def test_task_sources_match_the_record(self):
        for tid, t in tasks().items():
            src = t["source"]
            if src["benchmark"] == "Verus-Bench":
                key = [k for k, d in sources.VERUS_BENCH.items() if d == tid]
                self.assertEqual(len(key), 1, tid)
                name = key[0].replace("mbpp_task_id_", "task_id_").split("_", 1)[1] \
                    if not key[0].startswith("mbpp") else key[0].replace("mbpp_", "")
                self.assertTrue(src["path"].endswith(name + ".rs"), (tid, src["path"]))
            else:
                vid = src["path"].split("#")[1]
                self.assertEqual(dict(sources.VERICODING)[vid], tid)


class GivenDefinitions(BvbSet):
    """The hidden contract of a G1 task names a given definition; changing it fails the task."""

    def run_ops(self, t, ops):
        agent = harness.ScriptAgent(harness.ops_to_calls(ops))
        with mock.patch.object(harness, "make_agent", lambda kind, task, ws: agent):
            return harness.run_task(t, "scripted")

    def test_reference_keeps_given_intact(self):
        t = tasks()["vb-misc-sum"]
        r = self.run_ops(t, t["reference"])
        self.assertTrue(r["given_intact"])
        self.assertTrue(r["success"])

    def test_changing_a_given_definition_fails(self):
        t = tasks()["vb-misc-sum"]
        ops = copy.deepcopy(t["reference"]) + [{
            "op": "put", "unit": "given",
            "code": "spec fn arith_sum(i: u64) -> u64\n    decreases i,\n{\n    if i == 0 { 0 } else { i + arith_sum(i - 1) + 0 * i }\n}"}]
        r = self.run_ops(t, ops)
        self.assertFalse(r["given_intact"])
        self.assertFalse(r["success"])

    def test_tasks_without_given_have_no_flag(self):
        t = tasks()["vb-mbpp-284-all-equal"]
        r = self.run_ops(t, t["reference"])
        self.assertNotIn("given_intact", r)
        self.assertTrue(r["success"])


class HiddenHelpers(BvbSet):
    def test_hidden_helper_predicates_are_unfolded(self):
        # vc-va0590's hidden contract calls ref_total etc.; the check must unfold them.
        t = tasks()["vc-va0590-boring-apartment"]
        self.assertGreater(len(re.findall(r"spec fn ref_", t["accept"]["specs"])), 2)
        r = GivenDefinitions.run_ops(self, t, t["reference"])
        self.assertTrue(r["success"])


if __name__ == "__main__":
    unittest.main()
