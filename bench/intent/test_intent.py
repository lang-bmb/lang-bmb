#!/usr/bin/env python3
"""H-INTENT's item set (ADR-0078, backlog T1-44e) as a second set of the
review harness (`bench/review`): the set is frozen and is what the generator
makes, the committed examples are what the compiler prints, every program is
proved, the hidden labels are what the compiler says of the contracts, the
views are deterministic and do not depend on the label, the scripted
reviewers detect everything and nothing in both arms, and the table computes
the registered rule and reads a verdict only from one model run. The numbers
the README states before any model run are pinned here. No network, no
model. Needs `bmb` ($BMB or target/debug/bmb)."""
import concurrent.futures
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
REVIEW = os.path.join(BENCH, "review")
sys.dont_write_bytecode = True
sys.path.insert(0, REVIEW)
sys.path.insert(0, HERE)  # `gen` is this set's; `harness`, `judge`, `views` are bench/review's
sys.path.append(os.path.join(BENCH, "m1"))  # last: m1 has its own `harness`
import gen  # noqa: E402
import harness  # noqa: E402
import judge  # noqa: E402
import views  # noqa: E402
from model_api import OpenAIAgent  # noqa: E402

harness.SET = HERE
CFG = views.load_set(HERE)
ITEMS = views.load_changes(HERE)
BMB = harness.bmb_bin()
JOBS = 2
N = len(ITEMS)
A, B = "code+spec", "spec+examples"
_views = {}


def m1_harness():
    """The m1 harness as a library, judging against BVB (its hidden contracts)."""
    spec = importlib.util.spec_from_file_location("intent_m1_harness", os.path.join(BENCH, "m1", "harness.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    mod.SET = os.path.join(BENCH, "bvb")
    return mod


def all_views(arm):
    if arm not in _views:
        _views[arm] = {i["id"]: views.view(HERE, CFG, i, arm, BMB) for i in ITEMS}
    return _views[arm]


def mean_tokens(arm, label=None):
    ids = [i["id"] for i in ITEMS if label in (None, i["label"])]
    return sum(views.size(all_views(arm)[x])[1] for x in ids) / len(ids)


class Frozen(unittest.TestCase):
    def test_manifest_matches(self):
        with open(os.path.join(HERE, "MANIFEST")) as f:
            self.assertEqual(f.read(), harness.manifest_text())
        listed = [line.split("  ", 1)[1] for line in harness.manifest_text().splitlines()]
        for name in ("prompt.md", "set.json", "gen.py", "../review/gen.py", "../review/views.py",
                     "../review/judge.py", "../review/harness.py", "../bvb/MANIFEST"):
            self.assertIn(name, listed)
        self.assertEqual(sum(1 for n in listed if n.startswith("changes/")), N)

    def test_the_review_set_is_still_frozen(self):
        # This set shares the harness, the views and the judge with
        # bench/review: its MANIFEST has to hold with the same bytes.
        r = subprocess.run([sys.executable, os.path.join(REVIEW, "harness.py"), "manifest"], capture_output=True,
                           text=True)
        self.assertEqual((r.returncode, r.stdout), (0, "MANIFEST ok\n"))

    def test_structure_without_a_compiler(self):
        self.assertEqual(gen.structure_problems(os.path.join(HERE, "changes")), [])
        self.assertRegex(CFG["generated_with"]["compiler_commit"], r"^[0-9a-f]{8}$")

    def test_structure_check_sees_edits(self):
        tmp = tempfile.mkdtemp(prefix="bmb-intent-")
        self.addCleanup(shutil.rmtree, tmp, True)
        d = os.path.join(tmp, "changes")
        shutil.copytree(os.path.join(HERE, "changes"), d)
        name = "vb-mbpp-62-smallest.weak.json"

        def edited(f):
            path = os.path.join(d, name)
            with open(path) as fh:
                saved = fh.read()
            item = json.loads(saved)
            f(item)
            with open(path, "w") as fh:
                fh.write(gen.dump(item))
            bad = gen.structure_problems(d)
            with open(path, "w") as fh:
                fh.write(saved)
            return bad

        def rows(item, f):
            lines = item["examples"].split("\n")
            item["examples"] = "\n".join(f(lines))
        self.assertEqual(gen.structure_problems(d), [])
        self.assertTrue(edited(lambda i: i.update(label="clean")))
        self.assertTrue(edited(lambda i: i.update(after=i["after"].replace("lower_bound(nums, r)", "true"))))
        self.assertTrue(edited(lambda i: i.update(intent=i["intent"] + " ")))
        self.assertTrue(edited(lambda i: i.pop("examples")))
        self.assertTrue(edited(lambda i: i.update(examples="")))
        self.assertTrue(edited(lambda i: i.update(examples="f([1]) = 1 allowed\n")))
        self.assertTrue(edited(lambda i: rows(i, lambda ls: ls[1:])))    # a row dropped: the summary counts it
        self.assertTrue(edited(lambda i: rows(i, lambda ls: ls[:-2] + [""])))  # no summary
        self.assertTrue(edited(lambda i: i.update(examples=i["examples"].replace("smallest_num", "other"))))
        self.assertTrue(edited(lambda i: i.update(examples=i["examples"].replace('"allowed","input"', '"fine","input"'))))
        self.assertEqual(gen.structure_problems(d), [])
        os.remove(os.path.join(d, name))
        self.assertTrue(gen.structure_problems(d))

    def test_composition(self):
        tasks = sorted(f[:-5] for f in os.listdir(os.path.join(BENCH, "bvb", "tasks")))
        self.assertEqual(len(tasks), 45)
        self.assertEqual([i["id"] for i in ITEMS], sorted(f"{t}.{k}" for t in tasks for k in ("reference", "weak")))
        self.assertEqual({(i["label"], i["kind"]) for i in ITEMS},
                         {("clean", "reference-contract"), ("defect", "weak-contract")})
        for t in tasks:
            ref, weak = (next(i for i in ITEMS if i["id"] == f"{t}.{k}") for k in ("reference", "weak"))
            self.assertEqual((ref["label"], ref["source"]["program"]), ("clean", "reference"))
            self.assertEqual((weak["label"], weak["source"]["program"]), ("defect", "weak_reference"))
            # One intent, one target, two programs with different contracts.
            self.assertEqual((ref["intent"], ref["def"], ref["file"]), (weak["intent"], weak["def"], weak["file"]))
            self.assertNotEqual(views.contract_text(ref["after"]), views.contract_text(weak["after"]))
            self.assertNotIn("before", ref)
            with open(os.path.join(BENCH, "bvb", "tasks", t + ".json")) as f:
                task = json.load(f)
            self.assertEqual(ref["def"], task["def"])
            self.assertIn(ref["intent"], task["prompt"])
            self.assertNotIn("Required signature", ref["intent"])

    def test_the_programs_are_those_of_the_review_set(self):
        # The same derivation as bench/review: a task's two items are the two
        # ends of its `weaken` change.
        review = {c["id"]: c for c in views.load_changes(REVIEW)}
        for i in ITEMS:
            c = review[i["source"]["task"] + ".weaken"]
            self.assertEqual(i["after"], c["before" if i["label"] == "clean" else "after"])
            self.assertEqual(i["intent"], c["intent"])


class Programs(unittest.TestCase):
    """Proved programs, and labels as constructed: with the m1 judge (ADR-0021
    §2: the program is proved, and its contract implies the task's hidden
    reference contract) a program is accepted exactly when its item is
    `clean`."""

    def test_every_program_is_proved_and_labels_are_as_constructed(self):
        m1 = m1_harness()
        self.assertEqual(len({(i["source"]["task"], i["after"]) for i in ITEMS}), 90)

        def check(item):
            with open(os.path.join(BENCH, "bvb", "tasks", item["source"]["task"] + ".json")) as f:
                task = json.load(f)
            tmp = tempfile.mkdtemp(prefix="bmb-intent-")
            try:
                ws = views.workspace(HERE, item, "after", os.path.join(tmp, "ws"))
                r = subprocess.run([BMB, "verify", ws], capture_output=True, text=True)
                sig_ok, got, _ = m1.judge_target(task, ws, r.returncode == 0)
                return item, r.returncode == 0, sig_ok, got
            finally:
                shutil.rmtree(tmp, ignore_errors=True)

        with concurrent.futures.ThreadPoolExecutor(JOBS) as pool:
            results = list(pool.map(check, ITEMS))
        for item, proved, sig_ok, got in results:
            self.assertTrue(proved, f"{item['id']}: not proved")
            self.assertTrue(sig_ok, f"{item['id']}: signature")
            self.assertEqual(got, item["label"] == "clean", f"{item['id']}: hidden contract")


class Examples(unittest.TestCase):
    def test_the_committed_examples_are_what_this_compiler_prints(self):
        # The reviewer is shown the committed text. This compares it with the
        # tool's output now (about a minute). The output is the solver's
        # answer under a fixed seed and resource limit: the same bytes run
        # after run on one compiler. A compiler change that alters it fails
        # here; the README says what then.
        with concurrent.futures.ThreadPoolExecutor(JOBS) as pool:
            texts = list(pool.map(lambda i: gen.examples(BMB, i), ITEMS))
        differing = [i["id"] for i, t in zip(ITEMS, texts) if t != i["examples"]]
        self.assertEqual(differing, [], "bmb examples no longer prints the committed text for these items")

    def test_shape(self):
        for i in ITEMS:
            self.assertIsNone(gen.examples_problem(i, i["examples"]), i["id"])
            rows, summary = gen.parse_examples(i["examples"])
            self.assertEqual(summary["max_len"], 4)
            self.assertLessEqual(max((summary[k] for k in gen.ROW_KINDS)), 6)
            self.assertNotIn("tmp", i["examples"])

    def test_what_the_readme_states_about_the_examples(self):
        s = gen.stats(ITEMS)
        want = {"행이 하나도 없음": (0, 0), "`allowed` 행 없음": (0, 0), "`rejected` 행 없음": (0, 2),
                "`excluded` 행 없음": (16, 16), "`undecided` > 0": (4, 3), "요약에 `reason`(질의 없음)": (0, 0),
                "한 입력에 `allowed` 결과가 둘": (5, 44), "`allowed` 행이 있고 입력마다 결과 하나": (40, 1)}
        self.assertEqual({k: (v["clean"], v["defect"]) for k, v in s["counts"].items()}, want)
        self.assertEqual(s["same_examples_tasks"], [])
        self.assertEqual({k: round(v, 2) for k, v in s["mean_rows"].items()}, {"clean": 7.93, "defect": 9.93})
        # The rule "defect exactly when some input has two allowed results".
        right = sum(1 for i in ITEMS if (gen.features(i)["two_allowed_inputs"] > 0) == (i["label"] == "defect"))
        self.assertEqual((right, N), (84, 90))
        text = gen.stats_text(ITEMS)
        self.assertIn("| 한 입력에 `allowed` 결과가 둘 | 5 | 44 |", text)
        self.assertIn("두 항목의 예시 텍스트가 같은 과제: 0 / 45\n", text)

    def test_what_the_readme_states_about_sizes(self):
        got = {(arm, label): round(mean_tokens(arm, label), 1) for arm in (A, B) for label in (None, "clean", "defect")}
        self.assertEqual(got, {(A, None): 216.1, (A, "clean"): 221.4, (A, "defect"): 210.7,
                               (B, None): 359.0, (B, "clean"): 341.9, (B, "defect"): 376.1})
        self.assertEqual([round(mean_tokens(B, lab) / mean_tokens(A, lab), 3) for lab in (None, "clean", "defect")],
                         [1.662, 1.544, 1.785])
        # No item is within the criterion (1/2).
        within = [i["id"] for i in ITEMS
                  if 2 * views.size(all_views(B)[i["id"]])[1] <= views.size(all_views(A)[i["id"]])[1]]
        self.assertEqual(within, [])
        sizes = harness.sizes()
        self.assertIn("| (전체) | 90 | 1.662 | 1.486 |", sizes)
        self.assertIn("| reference-contract | 45 | 1.544 | 1.403 |", sizes)
        self.assertIn("| weak-contract | 45 | 1.785 | 1.572 |", sizes)
        self.assertIn("토큰 비는 spec+examples/code+spec, 기준 ≤ 1/2", sizes)


class Views(unittest.TestCase):
    def test_arms(self):
        self.assertEqual(CFG["arms"], {A: ["code"], B: ["contract", "field:examples"]})
        self.assertEqual(CFG["compare"], {"candidate": B, "baseline": A, "token_ratio": [1, 2]})
        self.assertEqual(CFG["hypothesis"], "H-INTENT")
        i = next(x for x in ITEMS if x["id"] == "vb-mbpp-62-smallest.weak")
        a, b = all_views(A)[i["id"]], all_views(B)[i["id"]]
        self.assertEqual(a, "## Code\n" + i["after"])
        self.assertIn("let mut", a)
        self.assertEqual(b, "## " + views.TITLES["contract"] + "\n" + views.contract_text(i["after"])
                         + "\n## Output of `bmb examples` for the function under review\n" + i["examples"])
        self.assertIn("    ensures lower_bound(nums, r),\n", b)
        self.assertIn('{"def":"tasks::smallest_num","kind":"allowed","input":{"nums":"[7] (len 1)"},"result":"0"}\n'
                      '{"def":"tasks::smallest_num","kind":"allowed","input":{"nums":"[7] (len 1)"},"result":"-1"}\n', b)
        self.assertEqual(views.message(i, b), "## Intent\n" + i["intent"] + "\n\n" + b)

    def test_the_examples_arm_shows_no_body_and_the_code_arm_no_examples(self):
        for i in ITEMS:
            a, b = all_views(A)[i["id"]], all_views(B)[i["id"]]
            contract = b.split("\n## Output of `bmb examples`", 1)[0]
            for body in ("invariant", "let ", "#[effects", "#[uses", "while "):
                self.assertNotIn(body, contract, i["id"])
            self.assertIn("ensures", contract, i["id"])
            self.assertNotIn('"summary":true', a)
            self.assertNotIn("bmb examples", a)
            self.assertTrue(b.endswith(i["examples"]))

    def test_views_are_deterministic_and_sized(self):
        for arm in CFG["arms"]:
            first = all_views(arm)
            rows, s = harness.run(arm, "oracle", jobs=JOBS)
            self.assertEqual(len(rows), N)
            for r in rows:
                text = first[r["id"]]
                self.assertEqual(r["view_sha256"], harness.hashlib.sha256(text.encode()).hexdigest(), r["id"])
                n = len(text.encode())
                self.assertEqual((r["view_bytes"], r["view_tokens"]), (n, -(-n // 4)))
                self.assertGreater(r["message_tokens"], r["view_tokens"])

    def test_views_need_no_compiler(self):
        for arm in CFG["arms"]:
            for i in ITEMS[:6]:
                self.assertEqual(views.view(HERE, CFG, i, arm, "/nonexistent/bmb"), all_views(arm)[i["id"]])

    def test_views_do_not_read_the_label(self):
        for i in ITEMS:
            other = dict(i, label="clean" if i["label"] == "defect" else "defect", kind="x", source={}, id="x")
            for arm in CFG["arms"]:
                self.assertEqual(views.view(HERE, CFG, other, arm, BMB), all_views(arm)[i["id"]])
                message = views.message(i, all_views(arm)[i["id"]])
                for hidden in ("weak_reference", "reference-contract", "weak-contract", i["id"], '"label"'):
                    self.assertNotIn(hidden, message)


class Scripted(unittest.TestCase):
    def test_oracle_detects_all_with_no_false_alarm_in_both_arms(self):
        for arm in (A, B):
            rows, s = harness.run(arm, "oracle")
            self.assertEqual((s["task_set"], s["hypothesis"], s["arm"]), ("intent", "H-INTENT", arm))
            self.assertEqual((s["defects"], s["detected"], s["cleans"], s["false_alarms"], s["malformed"]),
                             (45, 45, 45, 0, 0))
            self.assertEqual({r["outcome"] for r in rows}, {"detected", "passed"})

    def test_blind_detects_nothing_in_both_arms(self):
        for arm in (A, B):
            rows, s = harness.run(arm, "blind")
            self.assertEqual((s["defects"], s["detected"], s["cleans"], s["false_alarms"], s["malformed"]),
                             (45, 0, 45, 0, 0))
            self.assertEqual({r["outcome"] for r in rows}, {"missed", "passed"})
            self.assertEqual((s["detection_rate"], s["false_alarm_rate"]), (0.0, 0.0))

    def test_cli(self):
        def cli(*args):
            return subprocess.run([sys.executable, os.path.join(REVIEW, "harness.py"), "--set", HERE, *args],
                                  capture_output=True, text=True)
        self.assertEqual(cli("manifest").stdout, "MANIFEST ok\n")
        self.assertEqual(cli("run", "--arm", B, "--agent", "oracle", "--expect", "all-right").returncode, 0)
        self.assertEqual(cli("run", "--arm", A, "--agent", "blind", "--expect", "none-detected").returncode, 0)
        self.assertNotEqual(cli("run", "--arm", B, "--agent", "blind", "--expect", "all-right").returncode, 0)
        self.assertNotEqual(cli("run", "--arm", "spec", "--agent", "oracle").returncode, 0)  # bench/review's arm
        item = ITEMS[0]
        self.assertEqual(cli("view", "--arm", B, "--change", item["id"]).stdout,
                         views.message(item, all_views(B)[item["id"]]))
        r = subprocess.run([sys.executable, os.path.join(HERE, "gen.py"), "--check"], capture_output=True, text=True)
        self.assertEqual((r.returncode, r.stdout), (0, "bench/intent/changes ok (structure)\n"))


class StubPost:
    def __init__(self, responses):
        self.responses, self.requests = list(responses), []

    def __call__(self, url, body, headers, stats=None, fold=None):
        self.requests.append(json.loads(json.dumps(body)))
        return {"choices": [{"finish_reason": "stop", "message": {"role": "assistant",
                                                                  "content": self.responses.pop(0)}}],
                "usage": {"prompt_tokens": 100, "completion_tokens": 7}}


class Model(unittest.TestCase):
    """The real-model path with a stub transport: this set's frozen prompt,
    one request per item, the same judgement as bench/review."""

    def test_one_request_per_item_with_this_sets_prompt(self):
        ids = ["vb-mbpp-62-smallest.reference", "vb-mbpp-62-smallest.weak", "vb-misc-sum.reference"]
        post = StubPost(['{"verdict": "ok", "reason": "states the minimum"}',
                         '```json\n{"verdict": "defect", "reason": "only a lower bound"}\n```',
                         "The contract looks fine."])
        make = lambda kind, system, message: OpenAIAgent(kind.partition(":")[2], system, message, post=post,  # noqa: E731
                                                         base="http://x")
        rows, s = harness.run(B, "openai:qwen-test", only=set(ids), make=make)
        self.assertEqual([r["outcome"] for r in rows], ["passed", "detected", "malformed"])
        self.assertEqual((s["detected"], s["defects"], s["false_alarms"], s["cleans"], s["malformed"]), (1, 1, 0, 2, 1))
        with open(os.path.join(HERE, "prompt.md")) as f:
            prompt = f.read()
        self.assertTrue(prompt.startswith("You are reviewing the contract of one function"))
        for word in ("`allowed`", "`rejected`", "`excluded`", "`undecided`", "does not mean that no\n    such example exists",
                     '{"verdict": "defect" | "ok", "reason": "<one line saying why>"}'):
            self.assertIn(word, prompt)
        for req, iid in zip(post.requests, ids):
            item = next(x for x in ITEMS if x["id"] == iid)
            self.assertEqual(req["messages"], [{"role": "system", "content": prompt},
                                               {"role": "user", "content": views.message(item, all_views(B)[iid])}])
            self.assertNotIn("tools", req)


class Results(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="bmb-intent-")
        self.addCleanup(shutil.rmtree, self.tmp, True)

    def write(self, name, arm, agent, rate, tokens, commit="abc12345", n=None):
        ids = [i["id"] for i in ITEMS][:n]
        head = {"header": True, "task_set": "intent", "hypothesis": "H-INTENT", "arm": arm, "agent": agent,
                "commit": commit, "manifest": harness.manifest_digest(), "changes": ids}
        rows, seen = [], 0
        for i in ITEMS[:n]:
            verdict = "ok"
            if i["label"] == "defect":
                seen += 1
                verdict = "defect" if seen <= round(rate * 45) else "ok"
            rows.append({"id": i["id"], "arm": arm, "label": i["label"], "kind": i["kind"], "verdict": verdict,
                         "outcome": judge.outcome(i["label"], verdict), "view_tokens": tokens,
                         "message_tokens": tokens + 80})
        path = os.path.join(self.tmp, name)
        with open(path, "w") as f:
            for o in [head] + rows + [{"summary": True, **judge.summary(rows)}]:
                f.write(json.dumps(o) + "\n")
        return path

    def test_the_rule_and_when_the_verdict_is_read(self):
        # ADR-0078 H-INTENT: confirmed when (b)'s detection is not lower by
        # more than D and (b) read at most 1/2 of (a)'s tokens; refuted when
        # (b)'s detection is lower by more than D.
        b = self.write("b.jsonl", B, "openai:m", 0.8, 100)
        a = self.write("a.jsonl", A, "openai:m", 1.0, 200)
        t = harness.table([b, a])
        self.assertIn("| spec+examples | openai:m | abc12345 | 36/45 (80.0%) | 0/45 (0.0%) | 0/90 | 100.0 | 180.0 |", t)
        self.assertIn("| code+spec | openai:m | abc12345 | 45/45 (100.0%) | 0/45 (0.0%) | 0/90 | 200.0 | 280.0 |", t)
        self.assertIn("spec+examples 대 code+spec: 탐지율 차이 -20.0%p (D = 30%p), 토큰 비 0.500 "
                      "(기준 ≤ 1/2; 의도 포함 0.643), 규칙의 값: confirmed", t)
        self.assertIn("| weak-contract | defect (`defect` 답) | 36/45 (80.0%) | 45/45 (100.0%) |", t)
        self.assertIn("| reference-contract | clean (`defect` 답) | 0/45 (0.0%) | 0/45 (0.0%) |", t)
        self.assertTrue(t.endswith("H-INTENT 판정(openai:m): 확정 조건 충족\n"))
        self.assertTrue(harness.table([b, a], d=10).endswith("H-INTENT 판정(openai:m): 반증\n"))
        low = self.write("l.jsonl", B, "openai:m", 0.6, 100)
        self.assertTrue(harness.table([low, a]).endswith("H-INTENT 판정(openai:m): 반증\n"))  # -40%p
        wide = self.write("w.jsonl", B, "openai:m", 1.0, 101)
        self.assertTrue(harness.table([wide, a]).endswith(
            "H-INTENT 판정(openai:m): 확정 아님(탐지율은 D 안, 토큰 비가 기준 초과)\n"))

        def why(*paths):
            last = harness.table(list(paths)).strip().split("\n")[-1]
            self.assertTrue(last.startswith("H-INTENT 판정: not judged ("), last)
            return last
        self.assertIn("no run of arm `code+spec`", why(b))
        self.assertIn("different agents", why(b, self.write("a2.jsonl", A, "anthropic:m", 1.0, 200)))
        self.assertIn("different compiler commits", why(b, self.write("a3.jsonl", A, "openai:m", 1, 200, "ffff0000")))
        self.assertIn("uncommitted sources", why(self.write("b4.jsonl", B, "openai:m", 1, 100, "abc12345+dirty"),
                                                 self.write("a4.jsonl", A, "openai:m", 1, 200, "abc12345+dirty")))
        self.assertIn("not the whole set", why(self.write("b5.jsonl", B, "openai:m", 1, 100, n=10),
                                               self.write("a5.jsonl", A, "openai:m", 1, 200, n=10)))
        self.assertIn("more than one run of arm `spec+examples`", why(b, b, a))

    def test_scripted_runs_print_the_rule_and_are_not_judged(self):
        paths = []
        for arm in (B, A):
            paths.append(os.path.join(self.tmp, arm + ".jsonl"))
            harness.run(arm, "oracle", out=paths[-1])
        head = harness.read(paths[0])[0]
        self.assertEqual((head["task_set"], head["hypothesis"], head["manifest"], len(head["changes"])),
                         ("intent", "H-INTENT", harness.manifest_digest(), N))
        t = harness.table(paths)
        self.assertIn("spec+examples 대 code+spec: 탐지율 차이 +0.0%p (D = 30%p), 토큰 비 1.662 "
                      "(기준 ≤ 1/2; 의도 포함 1.486), 규칙의 값: neither", t)
        self.assertTrue(t.endswith("H-INTENT 판정: not judged (scripted agent `oracle`, not a model)\n"))
        sys.stdout.write(t)


if __name__ == "__main__":
    unittest.main()
