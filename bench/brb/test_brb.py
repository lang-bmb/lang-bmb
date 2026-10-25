"""Tests of the BRB harness pieces that the oracle and weak runs do not cover:
task-set invariants, the G5 verdict and summary table, crash and hang
recovery of the case runner, the Rust tool sandbox, the checker verdicts, and
(when proptest's crates are available offline) a proptest session."""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import cases as casegen  # noqa: E402
import harness  # noqa: E402
import judge  # noqa: E402
from rust_tools import TOOLS, RustTools  # noqa: E402

sys.path.insert(0, os.path.join(harness.ROOT, "bench", "m1"))
from anthropic_agent import to_api_tools  # noqa: E402

TASKS = harness.tasks()


class TaskSet(unittest.TestCase):
    def test_count_and_ids(self):
        self.assertTrue(20 <= len(TASKS) <= 30, len(TASKS))
        ids = [t.ID for t in TASKS]
        self.assertEqual(len(set(ids)), len(ids))
        self.assertEqual(len({t.NAME for t in TASKS}), len(TASKS))

    def test_cases_deterministic_in_domain_and_sized(self):
        for t in TASKS:
            cs = judge.hidden_cases(t)
            self.assertEqual(cs, judge.hidden_cases(t), t.ID)
            self.assertTrue(150 <= len(cs) <= 1000, (t.ID, len(cs)))
            for args in cs:
                self.assertTrue(t.domain(*args), (t.ID, args))
                for (_, ty), v in zip(t.PARAMS, args):
                    vals = v if ty in judge.SLICES else [v]
                    lo, hi = (0, casegen.U64_MAX) if ty in ("u64", "usize") else (casegen.I64_MIN, casegen.I64_MAX)
                    self.assertTrue(all(lo <= x <= hi for x in vals), (t.ID, args))
                judge.expected(t, args)

    def test_every_task_has_judge_material(self):
        for t in TASKS:
            self.assertGreaterEqual(len(t.RUST_MUTANTS), 3, t.ID)
            for name in ("BMB_REF", "BMB_WEAK", "RUST_REF", "RUST_WEAK", "STATEMENT", "DOMAIN"):
                self.assertTrue(getattr(t, name), (t.ID, name))
            self.assertIn("#[test]", t.RUST_WEAK, t.ID)
            self.assertIn(f"fn {t.NAME}(", t.BMB_REF, t.ID)
            self.assertIn(judge.sig_rust(t), t.RUST_REF, t.ID)

    def test_both_arms_get_the_same_statement(self):
        for t in TASKS:
            b, r = judge.task_prompt(t, "bmb"), judge.task_prompt(t, "rust")
            head = b.split("Required signature")[0]
            self.assertEqual(head, r.split("Required signature")[0])
            self.assertIn(t.STATEMENT, head)
            self.assertIn(judge.sig_bmb(t), b)
            self.assertIn(judge.sig_rust(t), r)
            # Nothing hidden reaches the agent.
            for leak in ("mutant", "hidden case", "model("):
                self.assertNotIn(leak, b + r)

    def test_manifest_matches(self):
        self.assertEqual(harness.read_bytes(os.path.join(HERE, "MANIFEST")).decode(), harness.manifest_text())


class Verdict(unittest.TestCase):
    def test_oracle_line(self):
        self.assertIn("→ 지지", harness.oracle_verdict((1, 20), (5, 20)))
        self.assertIn("→ 반증", harness.oracle_verdict((5, 20), (5, 20)))  # equal shares falsify
        self.assertIn("→ 반증", harness.oracle_verdict((3, 10), (5, 20)))
        self.assertIn("판정 불가(Rust 통과 & 숨은 실패 0)", harness.oracle_verdict((0, 20), (0, 20)))
        self.assertIn("판정 불가(자기 검사를 통과한 과제가 없는 비교군)", harness.oracle_verdict((0, 0), (3, 20)))

    def test_g5_line(self):
        self.assertIn("→ 통과", harness.g5_verdict((2, 24, 150), (6, 24, 100)))
        self.assertIn("결함률 비 0.333, 총 토큰 비 1.500", harness.g5_verdict((2, 24, 150), (6, 24, 100)))
        self.assertIn("→ 미달", harness.g5_verdict((4, 24, 150), (6, 24, 100)))  # 0.667 > 1/2
        self.assertIn("→ 미달", harness.g5_verdict((1, 24, 250), (6, 24, 100)))  # tokens 2.5 > 2
        self.assertIn("판정 불가(Rust 결함 0)", harness.g5_verdict((0, 24, 10), (0, 24, 10)))
        self.assertIn("판정 불가(Rust 토큰 0)", harness.g5_verdict((1, 24, 0), (3, 24, 0)))

    def _results(self, d, name, arm, rows, manifest="m"):
        path = os.path.join(d, name)
        with open(path, "w") as f:
            for i, row in enumerate(rows):
                defect, tokens, final_ok = (*row, True)[:3]
                f.write(json.dumps({"id": f"t{i}", "arm": arm, "success": not defect, "defect": defect,
                                    "final_check_ok": final_ok,
                                    "delivered": True, "cases": 10, "failed": int(defect), "fail_kinds": {},
                                    "total_tokens": tokens, "model_usage": {}, "tool_calls": 3, "checks": 1,
                                    "limit_hit": None, "session_seconds": 1.0, "tool_seconds": 0.5}) + "\n")
            f.write(json.dumps({"summary": True, "arm": arm, "agent": "x", "manifest": manifest}) + "\n")
        return path

    def test_summarize(self):
        d = tempfile.mkdtemp()
        try:
            b = self._results(d, "b.jsonl", "bmb", [(False, 30), (False, 30), (True, 30), (False, 30)])
            r = self._results(d, "r.jsonl", "rust", [(True, 20), (True, 20), (False, 20), (False, 20)])
            out = harness.summarize([b, r])
            self.assertIn("| 성공(숨은 테스트 전부 통과) | 3/4 | 2/4 |", out)
            self.assertIn("| 결함률(결함 과제 / 과제) | 0.250 | 0.500 |", out)
            self.assertIn("결함률 비 0.500, 총 토큰 비 1.500 → 통과", out)
            self.assertIn("| 자기 검사 통과 & 숨은 실패 / 자기 검사 통과 | 1/4 | 2/4 |", out)
            self.assertIn("BMB 1/4 = 0.250, Rust 2/4 = 0.500 → 지지", out)
            # A task whose checker fails at the end is not a "passed" task.
            b2 = self._results(d, "b2.jsonl", "bmb", [(True, 30, False), (False, 30), (True, 30), (False, 30)])
            self.assertIn("BMB 1/3 = 0.333, Rust 2/4 = 0.500 → 지지", harness.summarize([b2, r]))
            self.assertIn("G5: 판정 불가(BMB 결과 하나와 Rust 결과 하나를 함께 주어야 함)", harness.summarize([b]))
            other = self._results(d, "o.jsonl", "rust", [(True, 20)], manifest="other")
            with self.assertRaises(SystemExit):
                harness.summarize([b, other])
        finally:
            shutil.rmtree(d)


class CaseRunner(unittest.TestCase):
    def test_crash_and_hang_are_recovered(self):
        d = tempfile.mkdtemp()
        try:
            drv = os.path.join(d, "drv.py")
            with open(drv, "w") as f:
                f.write("import os, sys, time\n"
                        "start = int(sys.argv[1])\n"
                        "for k in range(start, 8):\n"
                        "    if k == 2: os._exit(3)\n"
                        "    if k == 4: time.sleep(30)\n"
                        "    if k == 5: print(f'{k} panic', flush=True); continue\n"
                        "    print(f'{k} ok {k * 10}', flush=True)\n")
            got = judge.run_cases(lambda s: [sys.executable, drv, str(s)], 8, timeout=2)
            self.assertEqual(got[0], ("ok", ["0"]))
            self.assertEqual(got[2], ("crash",))
            self.assertEqual(got[3], ("ok", ["30"]))
            self.assertEqual(got[4], ("timeout",))
            self.assertEqual(got[5], ("panic",))
            self.assertEqual(got[7], ("ok", ["70"]))
        finally:
            shutil.rmtree(d)


class Sandbox(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp()
        self.crate = os.path.join(self.d, "solution")
        harness.make_crate(self.crate, proptest=False)
        self.tools = RustTools(self.crate, os.path.join(self.d, "target"))

    def tearDown(self):
        shutil.rmtree(self.d)

    def test_writes_are_confined(self):
        for path in ("Cargo.toml", "../x.rs", "/tmp/x.rs", "src/../../x.rs", "build.rs", "src/notes.txt", "target/x.rs"):
            _, err = self.tools.call("write_file", {"path": path, "content": "x"})
            self.assertTrue(err, path)
        _, err = self.tools.call("write_file", {"path": "tests/t.rs", "content": "#[test]\nfn t() {}\n"})
        self.assertFalse(err)
        text, err = self.tools.call("list_files", {})
        self.assertEqual(text.split("\n"), ["Cargo.toml", "src/lib.rs", "tests/t.rs"])
        self.assertFalse(self.tools.call("read_file", {"path": "Cargo.toml"})[1])
        self.assertTrue(self.tools.call("read_file", {"path": "../solution/../x"})[1])
        self.assertNotIn("proptest", self.tools.call("read_file", {"path": "Cargo.toml"})[0])

    def test_cargo_reports_errors(self):
        self.tools.call("write_file", {"path": "src/lib.rs", "content": "pub fn f() -> i64 { \"x\" }\n"})
        text, err = self.tools.call("cargo_build", {})
        self.assertTrue(err)
        self.assertIn("mismatched types", text)
        self.assertFalse(harness.check_ok("rust", text, err))

    def test_tools_fit_the_agent_adapter(self):
        api = to_api_tools(TOOLS)
        self.assertEqual([t["name"] for t in api],
                         ["list_files", "read_file", "write_file", "cargo_build", "cargo_test"])
        self.assertTrue(all(t["input_schema"]["type"] == "object" for t in api))


class StubClient:
    """Stands in for the Anthropic client: replays fixed responses."""

    def __init__(self, responses):
        self.responses = list(responses)
        self.messages = self

    def create(self, **kw):
        return self.responses.pop(0)


def _resp(stop, out_tokens, *uses):
    from types import SimpleNamespace as NS
    content = [NS(type="tool_use", id=f"u{i}", name=n, input=a) for i, (n, a) in enumerate(uses)]
    return NS(stop_reason=stop, content=content, usage=NS(input_tokens=100, output_tokens=out_tokens))


class ModelAdapter(unittest.TestCase):
    """The real-model adapter drives the Rust arm through the same session
    loop; the session output-token cap stops it (no network)."""

    def _session(self, responses):
        from anthropic_agent import AnthropicAgent
        t = next(t for t in TASKS if t.ID == "t01-clamp")
        real = harness.make_agent
        harness.make_agent = lambda kind, task, arm: AnthropicAgent("stub", "sys", judge.task_prompt(task, arm),
                                                                   client=StubClient(responses))
        d = tempfile.mkdtemp()
        try:
            return harness.run_session(t, "rust", "anthropic:stub", os.path.join(d, "target"), False)
        finally:
            harness.make_agent = real
            shutil.rmtree(d)

    def test_rust_session(self):
        t = next(t for t in TASKS if t.ID == "t01-clamp")
        r = self._session([
            _resp("tool_use", 50, ("write_file", {"path": "src/lib.rs", "content": t.RUST_REF}), ("cargo_test", {})),
            _resp("end_turn", 10),
        ])
        self.assertEqual((r["success"], r["tool_calls"], r["last_check_ok"]), (True, 2, True))
        self.assertEqual((r["total_tokens"], r["limit_hit"]), (260, None))

    def test_output_token_cap(self):
        r = self._session([
            _resp("tool_use", harness.SESSION_OUTPUT_TOKENS, ("list_files", {})),
            _resp("tool_use", 10, ("list_files", {})),
        ])
        self.assertEqual((r["limit_hit"], r["tool_calls"], r["delivered"]), ("output_tokens", 1, False))


class Checker(unittest.TestCase):
    def test_bmb_verify_verdict(self):
        ok = '{"def":"tasks::f","status":"verified","obligations":1,"failed":0}\n'
        bad = '{"def":"tasks::f","status":"failed","obligations":1,"failed":1}\n'
        diag = '{"code":"E0303","severity":"error","message":"m"}\n'
        self.assertTrue(harness.check_ok("bmb", ok, False))
        self.assertFalse(harness.check_ok("bmb", bad, False))
        self.assertFalse(harness.check_ok("bmb", diag + ok, False))
        self.assertFalse(harness.check_ok("bmb", "", False))
        self.assertTrue(harness.check_ok("rust", "exit status: 0\nok", False))
        self.assertFalse(harness.check_ok("rust", "exit status: 101\nfailed", True))


def proptest_offline():
    d = tempfile.mkdtemp()
    try:
        crate = os.path.join(d, "c")
        harness.make_crate(crate, proptest=True)
        return subprocess.run(["cargo", "fetch", "--locked", "--offline"], cwd=crate,
                              capture_output=True).returncode == 0
    finally:
        shutil.rmtree(d)


@unittest.skipUnless(os.environ.get("BRB_PROPTEST") == "1" or proptest_offline(),
                     "proptest's crates are not available offline (run once with network, or set BRB_PROPTEST=1)")
class Proptest(unittest.TestCase):
    def test_proptest_session(self):
        d = tempfile.mkdtemp()
        try:
            target = os.path.join(d, "target")
            harness.warm_rust(target, proptest=True)
            crate = os.path.join(d, "solution")
            harness.make_crate(crate, proptest=True)
            tools = RustTools(crate, target)
            t = next(t for t in TASKS if t.ID == "t02-midpoint")
            prop = (t.RUST_REF + "\n#[cfg(test)]\nmod tests {\n    use super::*;\n    use proptest::prelude::*;\n"
                    "    proptest! {\n        #[test]\n        fn floor_mean(a: i64, b: i64) {\n"
                    "            prop_assert_eq!(midpoint(a, b) as i128, (a as i128 + b as i128).div_euclid(2));\n"
                    "        }\n    }\n}\n")
            tools.call("write_file", {"path": "src/lib.rs", "content": prop})
            text, err = tools.call("cargo_test", {})
            self.assertFalse(err, text)
            self.assertIn("floor_mean", text)
            # The judge does not need the dev-dependencies.
            v = judge.judge_rust(t, crate, os.path.join(d, "j"))
            self.assertEqual((v["delivered"], v["failed"]), (True, 0))
        finally:
            shutil.rmtree(d)


if __name__ == "__main__":
    unittest.main()
