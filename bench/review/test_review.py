#!/usr/bin/env python3
"""H-REVIEW's change set and the review harness (ADR-0078, backlog T1-44):
the set is frozen and is what the generator makes, every program of every
change is proved, the hidden labels are what the compiler says of the
contracts, the views are deterministic and do not depend on the label, the
scripted reviewers detect everything and nothing, and answers are judged by
the registered rule. No network, no model. Needs `bmb` ($BMB or
target/debug/bmb)."""
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
sys.dont_write_bytecode = True
sys.path.insert(0, HERE)
sys.path.append(os.path.join(BENCH, "m1"))  # after HERE: m1 has its own `harness`
import gen  # noqa: E402
import harness  # noqa: E402
import judge  # noqa: E402
import mutants  # noqa: E402
import views  # noqa: E402
from anthropic_agent import AnthropicAgent  # noqa: E402
from model_api import OpenAIAgent  # noqa: E402

CFG = views.load_set(HERE)
CHANGES = views.load_changes(HERE)
BMB = harness.bmb_bin()
JOBS = 2
N = len(CHANGES)
DEFECTS = sum(1 for c in CHANGES if c["label"] == "defect")
CLEANS = N - DEFECTS
# The committed set (generated with the compiler commit in set.json).
COMPOSITION = {("defect", "contract-weakened"): 45, ("clean", "contract-restored"): 45,
               ("defect", "contract-weakened+body"): 26, ("clean", "body-only"): 27}
_views = {}


def labels(changes):
    """(defects, cleans) of a list of changes."""
    d = sum(1 for c in changes if c["label"] == "defect")
    return d, len(changes) - d


def m1_harness():
    """The m1 harness as a library, judging against BVB (its hidden contracts)."""
    spec = importlib.util.spec_from_file_location("review_m1_harness", os.path.join(BENCH, "m1", "harness.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    mod.SET = os.path.join(BENCH, "bvb")
    return mod


def all_views(arm):
    """The view of every change in `arm`, built once for the whole file."""
    if arm not in _views:
        with concurrent.futures.ThreadPoolExecutor(JOBS) as pool:
            texts = list(pool.map(lambda c: views.view(HERE, CFG, c, arm, BMB), CHANGES))
        _views[arm] = {c["id"]: t for c, t in zip(CHANGES, texts)}
    return _views[arm]


class Frozen(unittest.TestCase):
    def test_manifest_matches(self):
        with open(os.path.join(HERE, "MANIFEST")) as f:
            self.assertEqual(f.read(), harness.manifest_text())
        listed = [line.split("  ", 1)[1] for line in harness.manifest_text().splitlines()]
        for name in ("prompt.md", "set.json", "gen.py", "mutants.py", "views.py", "judge.py", "harness.py",
                     "../bvb/MANIFEST"):
            self.assertIn(name, listed)
        self.assertEqual(sum(1 for n in listed if n.startswith("changes/")), len(CHANGES))

    def test_structure_without_a_compiler(self):
        # What `gen.py --check` checks. The whole regeneration (`gen.py --regen`,
        # minutes of proofs) is the gate job's own step.
        self.assertEqual(gen.structure_problems(os.path.join(HERE, "changes")), [])
        self.assertEqual(CFG["generated_with"]["mutants_per_task_and_kind"], mutants.K)
        self.assertRegex(CFG["generated_with"]["compiler_commit"], r"^[0-9a-f]{8}$")

    def test_structure_check_sees_edits(self):
        tmp = tempfile.mkdtemp(prefix="bmb-review-")
        self.addCleanup(shutil.rmtree, tmp, True)
        d = os.path.join(tmp, "changes")
        shutil.copytree(os.path.join(HERE, "changes"), d)

        def edited(name, f):
            path = os.path.join(d, name)
            with open(path) as fh:
                saved = fh.read()
            c = json.loads(saved)
            f(c)
            with open(path, "w") as fh:
                fh.write(gen.dump(c))
            bad = gen.structure_problems(d)
            with open(path, "w") as fh:
                fh.write(saved)
            return bad
        body = next(c for c in CHANGES if c["kind"] == "contract-weakened+body")
        clean = next(c for c in CHANGES if c["kind"] == "body-only")
        base = "vb-misc-sum.weaken.json"
        self.assertEqual(gen.structure_problems(d), [])
        self.assertTrue(edited(base, lambda c: c.update(label="clean")))
        self.assertTrue(edited(base, lambda c: c.update(after=c["after"].replace("r < 10000", "r < 10001"))))
        self.assertTrue(edited(body["id"] + ".json", lambda c: c.update(after=c["after"] + "\n")))
        self.assertTrue(edited(body["id"] + ".json", lambda c: c.update(label="clean")))
        self.assertTrue(edited(body["id"] + ".json", lambda c: c.pop("witness")))
        self.assertTrue(edited(body["id"] + ".json", lambda c: c["source"]["mutation"].update(after="0")))
        self.assertTrue(edited(clean["id"] + ".json", lambda c: c.update(before=c["after"])))
        self.assertTrue(edited(clean["id"] + ".json", lambda c: c["source"].update(after="weak_reference")))
        os.rename(os.path.join(d, clean["id"] + ".json"), os.path.join(d, clean["id"][:-1] + "9.json"))
        self.assertTrue(gen.structure_problems(d))

    def test_composition(self):
        tasks = sorted(f[:-5] for f in os.listdir(os.path.join(BENCH, "bvb", "tasks")))
        self.assertEqual(len(tasks), 45)
        self.assertEqual([c["id"] for c in CHANGES], sorted(c["id"] for c in CHANGES))
        by = {}
        for c in CHANGES:
            by.setdefault((c["label"], c["kind"]), []).append(c)
            self.assertEqual(c["source"]["set"], "bvb")
            self.assertNotEqual(c["before"], c["after"])
        self.assertEqual({k: len(v) for k, v in by.items()}, COMPOSITION)
        self.assertEqual({c["id"] for k in (("defect", "contract-weakened"), ("clean", "contract-restored"))
                          for c in by[k]}, {f"{t}.{k}" for t in tasks for k in ("weaken", "restore")})
        # A contract-only pair is one task's two references in the two directions.
        for c in by[("defect", "contract-weakened")]:
            twin = next(x for x in CHANGES if x["id"] == c["source"]["task"] + ".restore")
            self.assertEqual((c["before"], c["after"], c["intent"]), (twin["after"], twin["before"], twin["intent"]))
        # A mutant change starts from the reference program and differs from the
        # program its source names in one line; at most K per task and kind.
        per = {}
        for kind, key in (("contract-weakened+body", "weak_reference"), ("body-only", "reference")):
            for c in by[("defect" if "weak" in kind else "clean", kind)]:
                base = next(x for x in CHANGES if x["id"] == c["source"]["task"] + ".weaken")
                self.assertEqual(c["before"], base["before"])
                self.assertEqual(c["source"]["after"], key)
                named = base["after"] if key == "weak_reference" else base["before"]
                a, b = named.split("\n"), c["after"].split("\n")
                self.assertEqual(len(a), len(b))
                self.assertEqual([i + 1 for i in range(len(a)) if a[i] != b[i]], [c["source"]["mutation"]["line"]])
                self.assertEqual("witness" in c, kind == "contract-weakened+body")
                per[(c["source"]["task"], kind)] = per.get((c["source"]["task"], kind), 0) + 1
        self.assertLessEqual(max(per.values()), mutants.K)

    def test_intent_is_the_task_description(self):
        with open(os.path.join(BENCH, "bvb", "tasks", "vb-mbpp-62-smallest.json")) as f:
            task = json.load(f)
        c = next(x for x in CHANGES if x["id"] == "vb-mbpp-62-smallest.weaken")
        self.assertEqual(c["intent"], "Implement `smallest_num`: `nums` is non-empty. Return its smallest element. "
                         "The contract must state that the result is at most every element and equal to some element.")
        self.assertIn(c["intent"], task["prompt"])
        for c in CHANGES:
            self.assertNotIn("Required signature", c["intent"])
            self.assertNotIn("Create unit", c["intent"])


class Programs(unittest.TestCase):
    """Proved programs, and labels as constructed. With the m1 judge (ADR-0021
    §2: the program is proved, and its contract implies the task's hidden
    reference contract), a program is accepted exactly when it carries the
    reference contract: every `before` that is a reference program, and every
    `after` of a `clean` change. So a `defect` change ends at a proved program
    whose contract does not state the intent, and a `clean` change at one
    whose contract does."""

    def test_every_program_is_proved_and_labels_are_as_constructed(self):
        m1 = m1_harness()
        programs = {}  # (task, text) -> (expected to be accepted, change, which)
        for c in CHANGES:
            for which, accepted in (("before", c["source"]["before"] == "reference"), ("after", c["label"] == "clean")):
                key = (c["source"]["task"], c[which])
                self.assertEqual(programs.setdefault(key, (accepted, c, which))[0], accepted, c["id"])
        self.assertEqual(len(programs), 90 + sum(1 for c in CHANGES if "mutation" in c["source"]))

        def check(item):
            (task_id, _), (accepted, change, which) = item
            with open(os.path.join(BENCH, "bvb", "tasks", task_id + ".json")) as f:
                task = json.load(f)
            tmp = tempfile.mkdtemp(prefix="bmb-review-")
            try:
                ws = views.workspace(HERE, change, which, os.path.join(tmp, "ws"))
                r = subprocess.run([BMB, "verify", ws], capture_output=True, text=True)
                sig_ok, got, _ = m1.judge_target(task, ws, r.returncode == 0)
                return change["id"], which, r.returncode == 0, sig_ok, got, accepted
            finally:
                shutil.rmtree(tmp, ignore_errors=True)

        with concurrent.futures.ThreadPoolExecutor(JOBS) as pool:
            results = list(pool.map(check, sorted(programs.items(), key=lambda kv: kv[0])))
        for cid, which, proved, sig_ok, got, accepted in results:
            self.assertTrue(proved, f"{cid} {which}: not proved")
            self.assertTrue(sig_ok, f"{cid} {which}: signature")
            self.assertEqual(got, accepted, f"{cid} {which}: hidden contract")

    def test_weaken_never_contradicts_the_label(self):
        # `bmb weaken` compares the two contracts with each other (not with the
        # intent) and may not decide. What it can decide agrees with the label:
        # no `defect` change is reported as kept (same, refined, compatible) and
        # no `clean` change as weakened. A body-only change has the same
        # contract text, which the tool answers without a query. The counts
        # are in the README.
        allowed = {"contract-weakened": {"weakened", "undecided"}, "contract-weakened+body": {"weakened", "undecided"},
                   "contract-restored": {"refined", "compatible", "undecided"}, "body-only": {"same"}}
        spec = all_views("spec")
        for c in CHANGES:
            text = spec[c["id"]].split("## " + views.TITLES["weaken"] + "\n", 1)[1]
            self.assertIn(harness.weaken_verdict(text), allowed[c["kind"]], c["id"])
            self.assertNotIn("/tmp", text)

    def test_a_body_only_change_says_its_contract_diff_is_empty(self):
        spec, code = all_views("spec"), all_views("code")
        for c in CHANGES:
            head = "## " + views.TITLES["contract_diff"] + "\n"
            self.assertTrue(spec[c["id"]].startswith(head))
            empty = spec[c["id"]].startswith(head + "(no difference)\n\n## ")
            self.assertEqual(empty, c["kind"] == "body-only", c["id"])
            self.assertNotIn("(no difference)", code[c["id"]])

    def test_witnesses(self):
        # The stored witness of a `contract-weakened+body` change, decided again
        # for the first three (the regeneration step decides all): the input's
        # hidden precondition is proved, the `after` program returns the stored
        # output there and the reference program the expected one, and the
        # hidden contract rejects the output.
        seed = os.path.join(HERE, gen.SEED, "src", "given.bmb")
        for c in [x for x in CHANGES if "witness" in x][:3]:
            with open(os.path.join(BENCH, "bvb", "tasks", c["source"]["task"] + ".json")) as f:
                task = json.load(f)
            w = c["witness"]
            tmp = tempfile.mkdtemp(prefix="bmb-review-")
            try:
                path = os.path.join(tmp, "tasks.bmb")
                for which, want in (("after", w["output"]), ("before", w["expected"])):
                    with open(path, "w") as f:
                        f.write(c[which])
                    got = mutants.run_program(BMB, [seed, path], task["def"], task["signature"], [w["input"]], tmp)
                    self.assertEqual(got, [{"ok": want}], c["id"])
                self.assertNotEqual(w["output"], w["expected"])
                self.assertEqual(mutants.rejected(BMB, task, seed, [w, dict(w, output=w["expected"])], tmp),
                                 [True, False], c["id"])
                self.assertIn(w["input"], mutants.allowed_inputs(BMB, task, seed, tmp))
            finally:
                shutil.rmtree(tmp, ignore_errors=True)

    def test_mutation_and_inputs_without_a_compiler(self):
        text = "a\n    if x < y {\n"
        m = {"line": 2, "col": 8, "before": "x < y", "after": "x <= y"}
        self.assertEqual(mutants.apply_mutation(text, m), "a\n    if x <= y {\n")
        self.assertIsNone(mutants.apply_mutation(text, dict(m, col=7)))
        self.assertIsNone(mutants.apply_mutation(text, dict(m, line=9)))
        with open(os.path.join(BENCH, "bvb", "tasks", "vb-misc-reverse.json")) as f:
            task = json.load(f)
        ins = mutants.inputs(task)
        self.assertEqual(ins, mutants.inputs(task))
        self.assertTrue(40 <= len(ins) <= mutants.N_INPUTS)
        self.assertEqual(mutants.call_lines("stub", task["signature"], {"v": [1, 2]}, "r", "_0"),
                         ["let mut v_0: [u64; 2] = [1, 2];", "stub(&mut v_0);"])


class Views(unittest.TestCase):
    def test_contract_text(self):
        c = next(x for x in CHANGES if x["id"] == "vb-misc-sum.weaken")
        self.assertEqual(views.contract_text(c["before"]), (
            "spec fn small_sum(n: u64) -> bool { given::arith_sum(n) < 10000 }\n\n"
            "spec fn sum_nonneg(i: u64) -> bool { given::arith_sum(i) >= 0 }\n\n"
            "spec fn sum_is(n: u64, r: u64) -> bool { given::arith_sum(n) == r }\n\n"
            "fn nonneg(i: u64) -> ()\n    ensures sum_nonneg(i),\n\n"
            "fn compute_arith_sum(n: u64) -> (r: u64)\n    requires small_sum(n),\n    ensures sum_is(n, r),\n"))

    def test_contract_text_leaves_out_what_only_the_proof_uses(self):
        text = ('#[summary("s")]\nmod tasks {\n'
                "    spec fn inner(x: u64) -> bool { x > 0 }\n\n"
                "    spec fn outer(x: u64) -> bool\n        decreases x,\n    {\n        inner(x) && x < 9\n    }\n\n"
                "    spec fn inv_only(x: u64) -> bool { x < 100 }\n\n"
                "    #[effects(pure)]\n    #[uses(inner, outer, inv_only)]\n"
                "    fn f(x: u64) -> (r: u64)\n        requires outer(x),\n        ensures r == x,\n"
                "        decreases x,\n    {\n        let y: u64 = x;\n        assert(inv_only(y));\n        y\n    }\n}\n")
        self.assertEqual(views.contract_text(text), (
            "spec fn inner(x: u64) -> bool { x > 0 }\n\n"
            "spec fn outer(x: u64) -> bool\n    decreases x,\n{\n    inner(x) && x < 9\n}\n\n"
            "fn f(x: u64) -> (r: u64)\n    requires outer(x),\n    ensures r == x,\n"))

    def test_every_program_has_a_contract_text(self):
        for c in CHANGES:
            for which in ("before", "after"):
                text = views.contract_text(c[which])
                self.assertIn("ensures", text, c["id"])
                self.assertNotIn("invariant", text, c["id"])
                self.assertNotIn("#[", text.replace("#[trigger]", ""), c["id"])

    def test_arms(self):
        self.assertEqual(CFG["arms"], {"spec": ["contract_diff", "weaken"], "code": ["code_diff"]})
        self.assertEqual(CFG["compare"], {"candidate": "spec", "baseline": "code", "token_ratio": [1, 5]})
        c = next(x for x in CHANGES if x["id"] == "vb-mbpp-62-smallest.weaken")
        spec, code = all_views("spec")[c["id"]], all_views("code")[c["id"]]
        self.assertTrue(spec.startswith("## " + views.TITLES["contract_diff"] + "\n--- a/contract\n+++ b/contract\n"))
        self.assertIn("-    ensures is_min(nums, r),\n+    ensures lower_bound(nums, r),\n", spec)
        self.assertIn("\n## Output of `bmb weaken before after`\n{\"def\":\"tasks::smallest_num\"", spec)
        self.assertNotIn("let mut", spec)  # no body in the spec arm
        self.assertTrue(code.startswith("## Code diff\n--- a/src/tasks.bmb\n+++ b/src/tasks.bmb\n"))
        self.assertIn("-        ensures is_min(nums, r),\n+        ensures lower_bound(nums, r),\n", code)
        self.assertNotIn("bmb weaken", code)
        self.assertEqual(views.message(c, code), "## Intent\n" + c["intent"] + "\n\n" + code)

    def test_views_are_deterministic_and_sized(self):
        # A second build of every view (through the harness, as a run makes
        # it) has the same bytes, and a row's sizes are those of the view.
        for arm in CFG["arms"]:
            first = all_views(arm)
            rows, s = harness.run(arm, "oracle", jobs=JOBS)
            self.assertEqual(len(rows), len(CHANGES))
            # The oracle on the whole set, both arms: every defect, no false alarm.
            self.assertEqual((s["defects"], s["detected"], s["cleans"], s["false_alarms"], s["malformed"]),
                             (DEFECTS, DEFECTS, CLEANS, 0, 0))
            for r in rows:
                text = first[r["id"]]
                self.assertEqual(r["view_sha256"], harness.hashlib.sha256(text.encode()).hexdigest(), r["id"])
                n = len(text.encode())
                self.assertEqual((r["view_bytes"], r["view_tokens"]), (n, -(-n // 4)))
                self.assertGreater(r["message_tokens"], r["view_tokens"])

    def test_views_do_not_read_the_label(self):
        for c in CHANGES[:8]:
            other = {k: v for k, v in c.items() if k != "witness"}
            other.update(label="clean" if c["label"] == "defect" else "defect", kind="x", source={})
            for arm in CFG["arms"]:
                self.assertEqual(views.view(HERE, CFG, other, arm, BMB), all_views(arm)[c["id"]])
        for arm in CFG["arms"]:
            for c in CHANGES:
                for hidden in ("weak_reference", "contract-weakened", "contract-restored", "body-only", "mutant",
                               "witness", c["id"]):
                    self.assertNotIn(hidden, views.message(c, all_views(arm)[c["id"]]))

    def test_size(self):
        self.assertEqual([views.size(s) for s in ("", "a", "abcd", "abcde", "한")], [(0, 0), (1, 1), (4, 1), (5, 2), (3, 1)])


class Scripted(unittest.TestCase):
    def test_oracle_detects_all_with_no_false_alarm(self):
        for arm in ("code",):  # the spec arm's oracle run: Views.test_views_are_deterministic_and_sized
            rows, s = harness.run(arm, "oracle")
            self.assertEqual((s["defects"], s["detected"], s["cleans"], s["false_alarms"], s["malformed"]),
                             (DEFECTS, DEFECTS, CLEANS, 0, 0))
            self.assertEqual({r["outcome"] for r in rows}, {"detected", "passed"})

    def test_oracle_on_the_spec_arm(self):
        d, c = labels(CHANGES[:8])
        _, s = harness.run("spec", "oracle", only={x["id"] for x in CHANGES[:8]})
        self.assertEqual((s["defects"], s["detected"], s["cleans"], s["false_alarms"]), (d, d, c, 0))

    def test_blind_detects_nothing(self):
        rows, s = harness.run("code", "blind")
        self.assertEqual((s["defects"], s["detected"], s["cleans"], s["false_alarms"], s["malformed"]),
                         (DEFECTS, 0, CLEANS, 0, 0))
        self.assertEqual({r["outcome"] for r in rows}, {"missed", "passed"})
        self.assertEqual((s["detection_rate"], s["false_alarm_rate"]), (0.0, 0.0))
        _, s = harness.run("spec", "blind", only={c["id"] for c in CHANGES[:6]})
        self.assertEqual((s["detected"], s["false_alarms"]), (0, 0))

    def test_cli_expectations(self):
        def cli(*args):
            return subprocess.run([sys.executable, os.path.join(HERE, "harness.py"), *args], capture_output=True,
                                  text=True)
        self.assertEqual(cli("manifest").stdout, "MANIFEST ok\n")
        self.assertEqual(cli("run", "--arm", "code", "--agent", "oracle", "--expect", "all-right").returncode, 0)
        self.assertEqual(cli("run", "--arm", "code", "--agent", "blind", "--expect", "none-detected").returncode, 0)
        self.assertNotEqual(cli("run", "--arm", "code", "--agent", "blind", "--expect", "all-right").returncode, 0)
        self.assertNotEqual(cli("run", "--arm", "code", "--agent", "nobody").returncode, 0)
        self.assertNotEqual(cli("run", "--arm", "nothing", "--agent", "oracle").returncode, 0)


class Judgement(unittest.TestCase):
    def test_parse(self):
        ok = '{"verdict": "defect", "reason": "the result need not be an element"}'
        self.assertEqual(judge.parse(ok), ("defect", "the result need not be an element", None))
        self.assertEqual(judge.parse('  {"reason": "", "verdict": "ok"}\n'), ("ok", "", None))
        self.assertEqual(judge.parse("```json\n" + ok + "\n```")[0], "defect")
        self.assertEqual(judge.parse("```\n" + ok + "\n```\n")[0], "defect")
        for bad in ("", None, "defect", "The change is fine.\n" + ok, ok + "\nHope this helps.", ok + ok,
                    '["defect"]', '"defect"', '{"verdict": "defect"}', '{"verdict": "defect", "reason": 3}',
                    '{"verdict": "Defect", "reason": ""}', '{"verdict": "weakened", "reason": ""}',
                    '{"verdict": ["defect"], "reason": ""}', '{"verdict": "ok", "reason": "", "confidence": 1}',
                    "Here it is:\n```json\n" + ok + "\n```", '{"verdict": "defect", "reason": "cut'):
            verdict, reason, why = judge.parse(bad)
            self.assertEqual((verdict, reason), (None, None), bad)
            self.assertTrue(why, bad)

    def test_outcome(self):
        self.assertEqual([judge.outcome(lab, v) for lab in ("defect", "clean") for v in ("defect", "ok", None)],
                         ["detected", "missed", "malformed", "false_alarm", "passed", "malformed"])
        with self.assertRaises(ValueError):
            judge.outcome("weak", "ok")

    def test_malformed_is_its_own_class_and_not_a_detection(self):
        def row(label, verdict):
            return {"label": label, "outcome": judge.outcome(label, verdict), "view_tokens": 10, "message_tokens": 30}
        rows = ([row("defect", "defect")] * 2 + [row("defect", "ok")] + [row("defect", None)]
                + [row("clean", "ok")] * 2 + [row("clean", "defect")] + [row("clean", None)] * 2)
        s = judge.summary(rows)
        self.assertEqual((s["reviews"], s["defects"], s["detected"], s["cleans"], s["false_alarms"], s["malformed"]),
                         (9, 4, 2, 5, 1, 3))
        self.assertEqual((s["detection_rate"], s["false_alarm_rate"]), (0.5, 0.2))  # malformed stay in the denominators
        self.assertEqual((s["mean_view_tokens"], s["mean_message_tokens"]), (10, 30))
        self.assertEqual(judge.summary([])["detection_rate"], None)

    def test_compare(self):
        def arm(rate, tokens):
            return {"detection_rate": rate, "mean_view_tokens": tokens, "mean_message_tokens": tokens + 100}
        v = lambda a, b, d=30, ratio=(1, 5): judge.compare(a, b, d, ratio)["verdict"]  # noqa: E731
        self.assertEqual(v(arm(0.8, 20), arm(0.8, 100)), "confirmed")       # same, exactly 1/5
        self.assertEqual(v(arm(0.9, 20), arm(0.8, 100)), "confirmed")       # higher
        self.assertEqual(v(arm(0.5, 20), arm(0.8, 100)), "confirmed")       # lower by exactly D: within D
        self.assertEqual(v(arm(0.49, 20), arm(0.8, 100)), "refuted")        # lower by more than D
        self.assertEqual(v(arm(0.49, 500), arm(0.8, 100)), "refuted")       # refutation does not look at tokens
        self.assertEqual(v(arm(0.8, 21), arm(0.8, 100)), "neither")         # tokens above 1/5
        self.assertEqual(v(arm(0.7, 20), arm(0.8, 100), d=5), "refuted")
        self.assertEqual(v(arm(0.8, 50), arm(0.8, 100), ratio=(1, 2)), "confirmed")  # H-INTENT's ratio
        c = judge.compare(arm(0.6, 300), arm(0.8, 100), 30, (1, 5))
        self.assertEqual((round(c["detection_diff_pp"], 6), c["token_ratio"], c["message_token_ratio"]), (-20.0, 3.0, 2.0))
        self.assertIsNone(judge.compare(arm(None, 1), arm(0.5, 1), 30, (1, 5)))


class StubPost:
    def __init__(self, responses):
        self.responses, self.requests = list(responses), []

    def __call__(self, url, body, headers, stats=None, fold=None):
        self.requests.append(json.loads(json.dumps(body)))
        r = self.responses.pop(0)
        if isinstance(r, Exception):
            raise r
        return {"choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": r,
                                                                  "reasoning_content": "thinking"}}],
                "usage": {"prompt_tokens": 100, "completion_tokens": 7}}


def openai_make(post):
    return lambda kind, system, message: OpenAIAgent(kind.partition(":")[2], system, message, post=post, base="http://x")


class Model(unittest.TestCase):
    """The real-model path with a stub transport (as bench/m1/test_model_api.py)."""

    def setUp(self):
        self.ids = [f"{t}.{k}" for t in ("vb-mbpp-62-smallest", "vb-misc-sum") for k in ("restore", "weaken")]

    def test_one_request_per_change_with_the_frozen_prompt(self):
        post = StubPost(['{"verdict": "ok", "reason": "states the minimum"}',
                         '```json\n{"verdict": "defect", "reason": "only a lower bound"}\n```',
                         "I think this change is fine.",
                         RuntimeError("http://x: HTTP 400: prompt too long")])
        rows, s = harness.run("code", "openai:qwen-test", only=set(self.ids), make=openai_make(post))
        self.assertEqual([r["outcome"] for r in rows], ["passed", "detected", "malformed", "malformed"])
        self.assertEqual((s["detected"], s["defects"], s["false_alarms"], s["cleans"], s["malformed"]), (1, 2, 0, 2, 2))
        self.assertEqual(len(post.requests), 4)
        with open(os.path.join(HERE, "prompt.md")) as f:
            prompt = f.read()
        for req, cid in zip(post.requests, self.ids):
            c = next(x for x in CHANGES if x["id"] == cid)
            self.assertEqual(req["messages"], [{"role": "system", "content": prompt},
                                               {"role": "user", "content": views.message(c, all_views("code")[cid])}])
            self.assertNotIn("tools", req)
            self.assertNotIn(c["label"] + '"', json.dumps(req))
        self.assertEqual(rows[0]["model_usage"]["input_tokens"], 100)
        self.assertEqual(rows[0]["agent_settings"]["model"], "qwen-test")
        self.assertEqual((rows[2]["answer"], rows[2]["verdict"]), ("I think this change is fine.", None))
        self.assertEqual((rows[3]["stop"], rows[3]["answer"]), ("api_error", ""))

    def test_anthropic_agent(self):
        import types
        block = lambda **kw: types.SimpleNamespace(**kw)  # noqa: E731

        class Client:
            def __init__(self):
                self.requests, self.messages = [], self

            def create(self, **kw):
                self.requests.append(kw)
                return block(stop_reason="end_turn", usage=block(input_tokens=50, output_tokens=9),
                             content=[block(type="thinking", thinking="hm"),
                                      block(type="text", text='{"verdict": "defect", "reason": "r"}')])
        client = Client()
        rows, s = harness.run("code", "anthropic:sonnet-test", only={self.ids[1]},
                              make=lambda kind, system, message: AnthropicAgent("sonnet-test", system, message,
                                                                                client=client))
        self.assertEqual((rows[0]["outcome"], rows[0]["stop"], s["detected"]), ("detected", "end_turn", 1))
        self.assertEqual(client.requests[0]["tools"], [])
        self.assertTrue(client.requests[0]["system"].startswith("You are reviewing one change"))

    def test_unknown_agent_kind(self):
        with self.assertRaises(SystemExit):
            harness.run("code", "gemini", only={self.ids[0]})


class Results(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="bmb-review-")
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.ids = {c["id"] for c in CHANGES[:6]}

    def test_file_shape(self):
        path = os.path.join(self.tmp, "r.jsonl")
        rows, summary = harness.run("code", "oracle", out=path, only=self.ids)
        lines = harness.read(path)
        head = lines[0]
        self.assertEqual({k: head[k] for k in ("header", "task_set", "hypothesis", "arm", "agent")},
                         {"header": True, "task_set": "review", "hypothesis": "H-REVIEW", "arm": "code",
                          "agent": "oracle"})
        self.assertEqual((head["commit"], head["manifest"], head["changes"]),
                         (harness.run_commit(), harness.manifest_digest(), sorted(self.ids)))
        self.assertEqual(lines[1:-1], rows)
        self.assertEqual(lines[-1], summary)
        self.assertTrue(summary["summary"])
        self.assertEqual((summary["arm"], summary["agent"], summary["commit"]), ("code", "oracle", head["commit"]))
        with self.assertRaises(SystemExit):  # a finished run is not run again
            harness.run("code", "oracle", out=path, only=self.ids)

    def test_an_unfinished_run_resumes_without_reviewing_again(self):
        path = os.path.join(self.tmp, "r.jsonl")
        answer = '{"verdict": "ok", "reason": "r"}'

        class Stop(Exception):
            pass
        post = StubPost([answer, answer, Stop()])
        with self.assertRaises(Stop):
            harness.run("code", "openai:m", out=path, only=self.ids, make=openai_make(post))
        self.assertEqual(len(harness.read(path)), 3)  # header and two rows
        with self.assertRaises(SystemExit):  # another arm is another run
            harness.run("spec", "openai:m", out=path, only=self.ids, make=openai_make(StubPost([])))
        post = StubPost([answer] * 4)
        rows, s = harness.run("code", "openai:m", out=path, only=self.ids, make=openai_make(post))
        self.assertEqual((len(post.requests), len(rows), s["reviews"]), (4, 6, 6))
        self.assertEqual([r["id"] for r in rows], sorted(self.ids))

    def write(self, name, arm, agent, rate, tokens, commit="abc12345", n=None):
        ids = [c["id"] for c in CHANGES][:n]
        head = {"header": True, "task_set": "review", "hypothesis": "H-REVIEW", "arm": arm, "agent": agent,
                "commit": commit, "manifest": harness.manifest_digest(), "changes": ids}
        rows, seen = [], 0
        for c in CHANGES[:n]:
            verdict = "ok"
            if c["label"] == "defect":
                seen += 1
                verdict = "defect" if seen <= round(rate * DEFECTS) else "ok"
            rows.append({"id": c["id"], "arm": arm, "label": c["label"], "kind": c["kind"], "verdict": verdict,
                         "outcome": judge.outcome(c["label"], verdict), "view_tokens": tokens,
                         "message_tokens": tokens + 80})
        path = os.path.join(self.tmp, name)
        with open(path, "w") as f:
            for o in [head] + rows + [{"summary": True, **judge.summary(rows)}]:
                f.write(json.dumps(o) + "\n")
        return path

    def test_table_and_when_the_verdict_is_read(self):
        k = round(0.8 * DEFECTS)
        spec = self.write("s.jsonl", "spec", "openai:m", 0.8, 30)
        code = self.write("c.jsonl", "code", "openai:m", 1.0, 200)
        t = harness.table([spec, code])
        self.assertIn(f"| spec | openai:m | abc12345 | {k}/{DEFECTS} ({100 * k / DEFECTS:.1f}%) | "
                      f"0/{CLEANS} (0.0%) | 0/{N} | 30.0 | 110.0 |", t)
        self.assertIn(f"| code | openai:m | abc12345 | {DEFECTS}/{DEFECTS} (100.0%) | 0/{CLEANS} (0.0%) | 0/{N} | "
                      "200.0 | 280.0 |", t)
        self.assertIn(f"spec 대 code: 탐지율 차이 {100 * k / DEFECTS - 100:+.1f}%p (D = 30%p), 토큰 비 0.150 "
                      "(기준 ≤ 1/5; 의도 포함 0.393), 규칙의 값: confirmed", t)
        self.assertIn("| 종류 | 라벨 | spec (openai:m) | code (openai:m) |", t)
        self.assertIn(f"| body-only | clean (`defect` 답) | 0/{COMPOSITION[('clean', 'body-only')]} (0.0%) | ", t)
        self.assertRegex(t, r"\| contract-weakened \| defect \(`defect` 답\) \| \d+/45 \(.*%\) \| 45/45 \(100.0%\) \|")
        self.assertTrue(t.endswith("H-REVIEW 판정(openai:m): 확정 조건 충족\n"))
        self.assertTrue(harness.table([spec, code], d=10).endswith("H-REVIEW 판정(openai:m): 반증\n"))
        wide = self.write("w.jsonl", "spec", "openai:m", 1.0, 100)
        self.assertIn("확정 아님", harness.table([wide, code]))

        def why(*paths):
            last = harness.table(list(paths)).strip().split("\n")[-1]
            self.assertTrue(last.startswith("H-REVIEW 판정: not judged ("), last)
            return last
        self.assertIn("no run of arm `code`", why(spec))
        self.assertIn("different agents", why(spec, self.write("c2.jsonl", "code", "anthropic:m", 1.0, 200)))
        self.assertIn("different compiler commits", why(spec, self.write("c3.jsonl", "code", "openai:m", 1, 200, "ffff0000")))
        self.assertIn("uncommitted sources", why(self.write("s4.jsonl", "spec", "openai:m", 1, 30, "abc12345+dirty"),
                                                 self.write("c4.jsonl", "code", "openai:m", 1, 200, "abc12345+dirty")))
        self.assertIn("scripted agent `oracle`", why(self.write("s5.jsonl", "spec", "oracle", 1, 30),
                                                     self.write("c5.jsonl", "code", "oracle", 1, 200)))
        self.assertIn("not the whole set", why(self.write("s6.jsonl", "spec", "openai:m", 1, 30, n=10),
                                               self.write("c6.jsonl", "code", "openai:m", 1, 200, n=10)))
        self.assertIn("more than one run of arm `spec`", why(spec, spec, code))

    def test_scripted_runs_are_not_judged(self):
        paths = []
        for arm in ("spec", "code"):
            paths.append(os.path.join(self.tmp, arm + ".jsonl"))
            harness.run(arm, "oracle", out=paths[-1], only=self.ids)
        self.assertIn("H-REVIEW 판정: not judged (scripted agent `oracle`, not a model)", harness.table(paths))


class OtherSet(unittest.TestCase):
    """A second set is data: its own directory with `set.json` (other arms,
    other ratio), prompt, changes and MANIFEST, run by the same harness. The
    shape here is H-INTENT's (ADR-0078): code + spec against spec + examples."""

    def test_a_set_with_other_arms(self):
        tmp = tempfile.mkdtemp(prefix="bmb-review-")
        self.addCleanup(shutil.rmtree, tmp, True)
        d = os.path.join(tmp, "intent-test")
        os.makedirs(os.path.join(d, "changes"))
        cfg = {"hypothesis": "H-INTENT",
               "arms": {"code+spec": ["code", "contract"], "spec+examples": ["contract", "field:examples"]},
               "titles": {"field:examples": "Examples generated from the contract"},
               "compare": {"candidate": "spec+examples", "baseline": "code+spec", "token_ratio": [1, 2]},
               "frozen": []}
        with open(os.path.join(d, "set.json"), "w") as f:
            json.dump(cfg, f)
        with open(os.path.join(d, "prompt.md"), "w") as f:
            f.write("Review the contract against the intent.\n")
        four = [c for c in CHANGES if c["source"]["task"] in ("vb-misc-map", "vb-misc-sum")
                and "mutation" not in c["source"]]
        self.assertEqual(labels(four), (2, 2))
        for c in four:
            with open(os.path.join(d, "changes", c["id"] + ".json"), "w") as f:
                json.dump(dict(c, examples="f([1]) = 1 allowed\nf([1]) = 0 rejected\n"), f)
        saved = harness.SET
        harness.SET = d
        self.addCleanup(setattr, harness, "SET", saved)
        with open(os.path.join(d, "MANIFEST"), "w") as f:
            f.write(harness.manifest_text())
        harness.check_manifest()
        paths = []
        for arm in cfg["arms"]:
            paths.append(os.path.join(tmp, arm + ".jsonl"))
            rows, s = harness.run(arm, "oracle", out=paths[-1])
            self.assertEqual((s["task_set"], s["hypothesis"], s["detected"], s["false_alarms"]),
                             ("intent-test", "H-INTENT", 2, 0))
        c = four[0]
        view = views.view(d, cfg, dict(c, examples="E\n"), "spec+examples", BMB)
        self.assertEqual(view, "## " + views.TITLES["contract"] + "\n" + views.contract_text(c["after"])
                         + "\n## Examples generated from the contract\nE\n")
        self.assertTrue(views.view(d, cfg, c, "code+spec", BMB).startswith("## Code\n" + c["after"]))
        t = harness.table(paths)
        self.assertIn("spec+examples 대 code+spec", t)
        self.assertIn("(기준 ≤ 1/2;", t)
        self.assertIn("H-INTENT 판정: not judged", t)


if __name__ == "__main__":
    unittest.main()
