"""Logic of the original-Verus BVB harness (backlog 2A-09) with a stub verifier.

No Verus, no network, no paid API: the gate runs this. The real verifier path
(oracle all pass, weak all rejected) runs in the `bvb-verus` CI job, which
installs the pinned Verus release (.github/workflows/check.yml).
"""
import json
import os
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.dont_write_bytecode = True
import harness  # noqa: E402

TASK_DIR = os.path.join(os.path.dirname(HERE), "tasks")


def task(tid):
    with open(os.path.join(TASK_DIR, tid + ".json")) as f:
        return json.load(f)


def all_tasks():
    return [task(f[:-5]) for f in sorted(os.listdir(TASK_DIR))]


class StubVerifier:
    """Verifies everything except files containing one of `fail_on`; records calls."""

    version = "stub"

    def __init__(self, fail_on=()):
        self.fail_on = fail_on
        self.calls = []

    def verify(self, code, extra=()):
        self.calls.append(code)
        bad = [m for m in self.fail_on if m in code]
        return (not bad), ("stub: failed on " + ", ".join(bad)) if bad else "stub: verified"


class Signatures(unittest.TestCase):
    CODE = """use vstd::prelude::*;
fn main() {}
verus! {
spec fn helper(x: u64) -> bool { x > 0 }

pub fn find(v: &Vec<u64>,
            k: u64) -> (idx: usize)
    requires
        v.len() > 0,
    ensures
        idx < v.len(),
        forall|i: int| 0 <= i < v.len() ==> (v[i] <= v[i]),
    decreases v.len(),
{
    0
}
} // verus!
"""

    def test_signature_ignores_pub_whitespace_and_result_name(self):
        sig = harness.signature_of(self.CODE, "find")
        self.assertEqual(harness.normalize_signature(sig),
                         harness.normalize_signature("fn find(v: &Vec<u64>, k: u64) -> (r: usize)"))

    def test_signature_type_mismatch(self):
        sig = harness.signature_of(self.CODE, "find")
        self.assertNotEqual(harness.normalize_signature(sig),
                            harness.normalize_signature("fn find(v: &Vec<u32>, k: u64) -> (r: usize)"))

    def test_missing_function(self):
        self.assertIsNone(harness.signature_of(self.CODE, "absent"))

    def test_spec_fn_is_not_the_target(self):
        self.assertIsNone(harness.signature_of(self.CODE, "helper_missing"))
        self.assertIsNone(harness.signature_of("spec fn find(x: u64) -> bool { true }", "find"))

    def test_strip_ensures_keeps_other_clauses(self):
        weak = harness.strip_ensures(self.CODE, "find")
        self.assertNotIn("ensures", weak)
        self.assertNotIn("idx < v.len()", weak)
        self.assertIn("requires\n        v.len() > 0,", weak)
        self.assertIn("decreases v.len(),", weak)
        self.assertEqual(harness.signature_of(weak, "find"), harness.signature_of(self.CODE, "find"))


class CheckBlock(unittest.TestCase):
    def test_block_names_original_spec_and_target(self):
        t = task("vb-mbpp-113-is-integer")
        block = harness.check_block(t, "fn is_integer(text: &[u8]) -> (r: bool) { true }")
        self.assertIn("fn bvb_check(text: &[u8]) -> (result: bool)", block)
        self.assertIn("spec fn bvb_ref_is_digit_sepc(c: u8)", block)
        self.assertIn("ensures\n        result == (forall", block)
        self.assertIn("    is_integer(text)\n", block)
        self.assertNotIn("requires", block)

    def test_requires_present_when_original_has_one(self):
        t = task("vb-mbpp-567-is-sorted")
        block = harness.check_block(t, "")
        self.assertIn("requires\n        arr.len() > 0,", block)

    def test_hidden_names_avoid_agent_names(self):
        t = task("vb-mbpp-113-is-integer")
        block = harness.check_block(t, "spec fn bvb_check() {} spec fn bvb1_x() {}")
        self.assertIn("fn bvb2_check(", block)
        self.assertIn("bvb2_ref_is_digit_sepc", block)
        self.assertNotIn("bvb_ref", block)

    def test_given_functions_are_not_redefined(self):
        t = task("vb-mbpp-461-count-uppercase")
        block = harness.check_block(t, "")
        self.assertIn("count_uppercase_recursively(text@) == count", block)
        self.assertNotIn("spec fn count_uppercase_recursively", block)


class EveryTask(unittest.TestCase):
    """Shape checks on the frozen tasks that need no verifier."""

    def test_reference_has_the_required_signature(self):
        for t in all_tasks():
            v = t["verus"]
            sig = harness.signature_of(v["reference"], v["target"])
            self.assertIsNotNone(sig, t["id"])
            self.assertEqual(harness.normalize_signature(sig), harness.normalize_signature(v["signature"]), t["id"])

    def test_weak_control_drops_the_target_ensures(self):
        for t in all_tasks():
            v = t["verus"]
            weak = harness.strip_ensures(v["reference"], v["target"])
            self.assertNotEqual(weak, v["reference"], t["id"])
            self.assertEqual(harness.signature_of(weak, v["target"]), harness.signature_of(v["reference"], v["target"]))

    def test_given_functions_are_in_the_reference_and_skeleton(self):
        for t in all_tasks():
            given = t["verus"].get("given")
            if given:
                self.assertIn(harness.squash(given), harness.squash(t["verus"]["reference"]), t["id"])
                self.assertIn(harness.squash(given), harness.squash(harness.skeleton(t)), t["id"])
            else:
                self.assertEqual(harness.skeleton(t), harness.SKELETON)

    def test_prompts_share_the_description(self):
        for t in all_tasks():
            desc = t["verus"]["prompt"].split("\n\n")[0]
            self.assertIn(desc, t["prompt"], t["id"])
            self.assertTrue(t["verus"]["prompt"].endswith(t["verus"]["signature"]), t["id"])


class Judge(unittest.TestCase):
    def test_oracle_passes_with_a_verifying_stub(self):
        t = task("vb-mbpp-284-all-equal")
        v = StubVerifier()
        r = harness.judge(t, t["verus"]["reference"], v)
        self.assertEqual((r["verified"], r["signature_ok"], r["spec_accepted"], r["success"]), (True, True, True, True))
        self.assertEqual(len(v.calls), 2)
        self.assertIn("bvb_check", v.calls[1])

    def test_failed_spec_check(self):
        t = task("vb-mbpp-284-all-equal")
        r = harness.judge(t, t["verus"]["reference"], StubVerifier(fail_on=("bvb_check",)))
        self.assertEqual((r["verified"], r["spec_accepted"], r["success"]), (True, False, False))

    def test_unverified_file_skips_the_spec_check(self):
        t = task("vb-mbpp-284-all-equal")
        v = StubVerifier(fail_on=("all_elements_equals",))
        r = harness.judge(t, t["verus"]["reference"], v)
        self.assertEqual((r["verified"], r["spec_accepted"]), (False, False))
        self.assertEqual(len(v.calls), 1)

    def test_wrong_signature(self):
        t = task("vb-mbpp-284-all-equal")
        code = t["verus"]["reference"].replace("element: i32", "element: i64")
        r = harness.judge(t, code, StubVerifier())
        self.assertEqual((r["signature_ok"], r["success"]), (False, False))

    def test_changed_given_function(self):
        t = task("vb-misc-sum")
        code = t["verus"]["reference"].replace("i + arith_sum_int(", "i + 0 * arith_sum_int(")
        r = harness.judge(t, code, StubVerifier())
        self.assertFalse(r["given_intact"])
        self.assertFalse(r["success"])
        r = harness.judge(t, t["verus"]["reference"], StubVerifier())
        self.assertTrue(r["given_intact"])


class Sessions(unittest.TestCase):
    def run_script(self, t, calls, verifier=None):
        agent = harness.bmb_harness.ScriptAgent(calls)
        return harness.run_task(t, "scripted", verifier or StubVerifier(), agent=agent)

    def test_oracle_and_weak_agents(self):
        t = task("vb-mbpp-284-all-equal")
        for kind in ("oracle", "weak"):
            r = harness.run_task(t, kind, StubVerifier())
            self.assertEqual((r["tool_calls"], r["writes"], r["verify_calls"]), (2, 1, 1), kind)
        weak = harness.make_agent("weak", t).calls[0][1]["code"]
        self.assertNotIn("ensures", weak.split("fn all_elements_equals")[1].split("{")[0])

    def test_session_starts_from_the_skeleton(self):
        t = task("vb-misc-sum")
        seen = []

        class Reader:
            usage = {}

            def __init__(self):
                self.done = False

            def next_calls(self, last):
                if last:
                    seen.append(last[0][2])
                if self.done:
                    return []
                self.done = True
                return [("read_file", {})]

        harness.run_task(t, "scripted", StubVerifier(), agent=Reader())
        self.assertEqual(seen, [harness.skeleton(t)])

    def test_tool_call_limit(self):
        t = task("vb-mbpp-284-all-equal")
        r = self.run_script(t, [("verify", {})] * (harness.MAX_TOOL_CALLS + 5))
        self.assertTrue(r["limit_hit"])
        self.assertEqual(r["tool_calls"], harness.MAX_TOOL_CALLS)

    def test_bad_calls_are_errors_not_crashes(self):
        t = task("vb-mbpp-284-all-equal")
        results = []

        class Agent:
            usage = {}
            calls = [("write_file", {}), ("delete_file", {"path": "x"}), ("verify", {})]

            def next_calls(self, last):
                results.extend(last)
                return [self.calls.pop(0)] if self.calls else []

        r = harness.run_task(t, "scripted", StubVerifier(fail_on=("fn main",)), agent=Agent())
        self.assertEqual([err for *_, err in results], [True, True, True])
        self.assertEqual(r["writes"], 0)
        self.assertFalse(r["success"])

    def test_final_file_is_judged(self):
        t = task("vb-mbpp-284-all-equal")
        r = self.run_script(t, [("write_file", {"code": t["verus"]["reference"]}),
                                ("write_file", {"code": "fn main() {}"})])
        self.assertFalse(r["signature_ok"])


class Compare(unittest.TestCase):
    def write(self, recs):
        f = tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False)
        for r in recs:
            f.write(json.dumps(r) + "\n")
        f.close()
        self.addCleanup(os.unlink, f.name)
        return f.name

    def row(self, tid, ok):
        return {"id": tid, "kind": "write", "verified": True, "signature_ok": True, "spec_accepted": ok,
                "success": ok, "tool_calls": 3, "model_usage": {"input_tokens": 10, "output_tokens": 2}}

    def test_paired_table(self):
        b = self.write([self.row("a", True), self.row("b", False),
                        {"summary": True, "task_set": "bvb", "manifest": "m", "agent": "x"}])
        v = self.write([self.row("a", False), self.row("b", False),
                        {"summary": True, "task_set": "bvb-verus", "manifest": "m", "agent": "x"}])
        table = harness.compare(b, v)
        self.assertIn("| 1회 성공률 | 1/2 | 0/2 |", table)
        self.assertIn("| a | 성공 | 실패: spec_accepted |", table)

    def test_refuses_other_sets_and_manifests(self):
        b = self.write([{"summary": True, "task_set": "m1b", "manifest": "m"}])
        v = self.write([{"summary": True, "task_set": "bvb-verus", "manifest": "m"}])
        with self.assertRaises(SystemExit):
            harness.compare(b, v)
        b = self.write([{"summary": True, "task_set": "bvb", "manifest": "m"}])
        v = self.write([{"summary": True, "task_set": "bvb-verus", "manifest": "other"}])
        with self.assertRaises(SystemExit):
            harness.compare(b, v)


class Install(unittest.TestCase):
    def test_pins(self):
        e = harness.env()
        self.assertTrue(e["VERUS_ASSET"].startswith("verus-" + e["VERUS_VERSION"]))
        self.assertRegex(e["VERUS_SHA256"], r"^[0-9a-f]{64}$")
        self.assertRegex(e["VERUS_RUST_TOOLCHAIN"], r"^\d+\.\d+\.\d+-")


if __name__ == "__main__":
    unittest.main()
