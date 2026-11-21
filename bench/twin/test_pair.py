#!/usr/bin/env python3
"""Gate of the port-twin harness (backlog 3P-01): the item splitter, the Rust
tools' sandbox, and `pair.py check` on the fixture (twins agree on the
corpus, every oracle succeeds, every weak control fails its check)."""
import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import pair  # noqa: E402


class Items(unittest.TestCase):
    def test_rust_items_take_docs_and_bodies(self):
        text = ('//! m\nuse a::b;\n\n/// Doc.\npub fn f(x: i64) -> i64 {\n    if x > 0 { "}" } else { \'{\' };\n'
                '    x\n}\n\npub const N: usize = 3;\nfn g() {}\n')
        it = pair.rust_items(text, "m")
        self.assertEqual(sorted(it), ["m::<rest>", "m::N", "m::f", "m::g"])
        self.assertTrue(it["m::f"].startswith("/// Doc.\npub fn f") and it["m::f"].endswith("    x\n}"))
        self.assertEqual(it["m::<rest>"], "//! m\nuse a::b;")

    def test_bmb_items_by_unit_indent(self):
        text = ('#[summary("u")]\nmod u {\n    spec fn p(x: i64) -> bool { x > 0 }\n\n    #[effects(pure)]\n'
                '    fn f(x: i64) -> i64\n        requires p(x),\n    {\n        x\n    }\n}\n')
        it = pair.split_items(text, "u", "    ", pair.BMB_ITEM)
        self.assertEqual(sorted(it), ["u::<rest>", "u::f", "u::p"])
        self.assertTrue(it["u::f"].startswith("    #[effects(pure)]\n    fn f") and it["u::f"].endswith("    }"))

    def test_scope(self):
        before = {"a::f": "1", "a::g": "2"}
        after = {"a::f": "1x", "a::g": "2", "a::h": "3"}
        task = {"scope": {"rust": ["a::f"]}}
        self.assertEqual(pair.out_of_scope(task, "rust", before, after), ["a::h"])
        task["scope"]["new"] = True
        self.assertEqual(pair.out_of_scope(task, "rust", before, after), [])


class Comments(unittest.TestCase):
    """The Rust arm without comments (ADR-0106)."""

    def test_strip_keeps_code_and_inner_doc(self):
        src = ("//! m doc\n/// d\npub fn f<'a>(x: &'a str) -> char { // tail\n"
               '    let s = "// no /* \\" x"; let r = r#"y // " z"#; let b = b"//";\n'
               "    /* a /* nested */ b */ let c = '\\''; let q = '\"'; let u = '\\u{2f}';\n"
               "    /** block doc */\n    c\n}\n")
        want = ("//! m doc\npub fn f<'a>(x: &'a str) -> char {\n"
                '    let s = "// no /* \\" x"; let r = r#"y // " z"#; let b = b"//";\n'
                "     let c = '\\''; let q = '\"'; let u = '\\u{2f}';\n    c\n}\n")
        self.assertEqual(pair.strip_comments(src), want)

    def test_inner_doc_must_be_the_summary(self):
        self.assertEqual(pair.inner_doc("//! vm: the stack\n//!\n//! codes: 1\nfn f() {}\n"), "vm: the stack codes: 1")
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, "vm.bmb"), "w") as f:
                f.write('#[summary("vm: the stack")]\nmod vm {\n}\n')
            self.assertEqual(pair.summaries(d), {"vm": "vm: the stack"})
        self.assertEqual(pair.doc_parity("stk"), [])

    def test_docs_condition_keeps_outer_docs_only(self):
        src = "//! m\n/// d one\n/// d two\n//// not a doc\n#[inline]\nfn f() {} // tail\n/* b */\n"
        self.assertEqual(pair.strip_comments(src, docs=True), "//! m\n/// d one\n/// d two\n#[inline]\nfn f() {}\n")
        bmb = "mod m {\n    /// d one\n    /// d two\n    #[effects(pure)]\n    fn f() -> () {\n    }\n}\n"
        self.assertEqual(pair.strip_summaries(bmb), "mod m {\n    #[effects(pure)]\n    fn f() -> () {\n    }\n}\n")
        self.assertEqual(pair.item_docs(src), {"f": ["d one d two"]})
        self.assertEqual(pair.item_docs(bmb), {"f": ["d one d two"]})
        self.assertEqual(pair.item_docs("/// a\npub(crate) const N: usize = 1;\n/// b\nspec fn g() -> int;\n"),
                         {"N": ["a"], "g": ["b"]})

    def test_rust_docs_may_only_add_the_contract_in_words(self):
        ok = lambda extra: bool(pair.EXTRA.fullmatch(extra))  # noqa: E731
        self.assertTrue(ok(""))
        self.assertTrue(ok(" Requires `lo < hi <= s.len()`."))
        self.assertTrue(ok(" Requires x > 0. Ensures the result is 1.5 times x."))
        self.assertFalse(ok(" Also note the cache."))
        self.assertFalse(ok(" Requires x > 0. See vm."))

    def test_scope_ignores_comment_edits(self):
        a = pair.rust_items(pair.strip_comments("/// old\nfn f() {}\n"), "m")
        b = pair.rust_items(pair.strip_comments("/// new words\nfn f() {} // and here\n"), "m")
        self.assertEqual(pair.changed(a, b), [])


class RustTools(unittest.TestCase):
    def test_sandbox_and_edit(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "src"))
            with open(os.path.join(d, "src", "a.rs"), "w") as f:
                f.write("x x y\n")
            t = pair.RustTools(d, os.path.join(d, "target"))
            self.assertTrue(t.call("read_file", {"path": "../etc/passwd"})[1])
            self.assertTrue(t.call("write_file", {"path": "Cargo.toml", "content": ""})[1])
            self.assertTrue(t.call("edit_file", {"path": "src/a.rs", "old": "x", "new": "z"})[1])
            self.assertFalse(t.call("edit_file", {"path": "src/a.rs", "old": "y", "new": "z"})[1])
            self.assertEqual(t.call("read_file", {"path": "src/a.rs"})[0], "1\tx x z\n")
            self.assertEqual(t.call("grep", {"pattern": "z$"})[0], "src/a.rs:1:x x z\n")


class Gate(unittest.TestCase):
    def test_check(self):
        r = subprocess.run([sys.executable, os.path.join(HERE, "pair.py"), "check"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr[-4000:])


class Port(unittest.TestCase):
    """The comprehension tasks on the port twin (backlog 3P-02)."""

    def test_rust_index_names_methods_by_their_type(self):
        import index
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "compiler", "src"))
            with open(os.path.join(d, "compiler", "src", "m.rs"), "w") as f:
                f.write('fn free(x: i64) -> i64 {\n    x // "E0001"\n}\n\nimpl<\'a> T<\'a> {\n    pub fn get(&self) -> i64 {\n'
                        '        free(self.x)\n    }\n}\n\nimpl Tr for U {\n    fn run(&mut self) {\n        self.n += 1;\n'
                        '    }\n}\n\n#[cfg(test)]\nmod tests {\n    fn t() {}\n}\n')
            it = index.rust_index(d, ["m.rs"])
        self.assertEqual(sorted(it), ["m::T::get", "m::U::run", "m::free"])
        self.assertNotIn("E0001", it["m::free"])  # a comment is not the function's text
        self.assertEqual(index.callers(it, "m::free"), {"m::T::get"})

    def test_tasks_are_generated_and_the_index_agrees_with_the_map(self):
        r = subprocess.run([sys.executable, os.path.join(HERE, "port_tasks.py"), "check"], capture_output=True,
                           text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_change_tasks_are_generated(self):
        r = subprocess.run([sys.executable, os.path.join(HERE, "port_change.py"), "check"], capture_output=True,
                           text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_swapping_two_parameters_follows_the_calls(self):
        import port_change
        text = ('fn f(&self, a: i64, b: Vec<u8>) -> i64 {\n    self.f(1, g(2, 3)) + T::f(self, x, "a,b)")\n}\n'
                'fn h() {\n    f::<T>(\n        p,\n        q[1],\n    );\n}')
        got, names = port_change.swapped(text, "f")
        self.assertEqual(names, ("a", "b"))
        self.assertEqual(got, 'fn f(&self, b: Vec<u8>, a: i64) -> i64 {\n    self.f(g(2, 3), 1) + T::f(self, "a,b)", x)\n}\n'
                              'fn h() {\n    f::<T>(\n        q[1],\n        p,\n    );\n}')
        # A closure in an argument list is not cut: no task is made of it.
        self.assertIsNone(port_change.swapped("fn f(a: i64, b: i64) {}\nfn g() { f(1, h(|x, y| x)) }", "f"))

    def test_edits_replace_stretches_that_occur_once(self):
        import port_change
        old, new = "a\nb\nc\nb\nd", "a\nB\nc\nb\nD"
        text = old
        for o, n in port_change.edits_between(old, new):
            self.assertEqual(text.count(o), 1)
            text = text.replace(o, n, 1)
        self.assertEqual(text, new)
        text = "x(1)\ny\nx(1)"
        for o, n in port_change.edits_for(text, r"x\(", "z("):
            self.assertEqual(text.count(o), 1)
            text = text.replace(o, n, 1)
        self.assertEqual(text, "z(1)\ny\nz(1)")

    def test_replay_tasks_are_generated(self):
        r = subprocess.run([sys.executable, os.path.join(HERE, "port_replay.py"), "check"], capture_output=True,
                           text=True)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_the_statement_that_reports_a_code_is_cut_out(self):
        import port_replay
        rust = ('fn f(&mut self) {\n    if bad {\n        self.diags.push(\n            Diag::new("E0001", sp, "a; b")\n'
                '                .fix("c", None),\n        );\n        return None;\n    }\n}')
        self.assertEqual(port_replay.without(rust, "E0001"), 'fn f(&mut self) {\n    if bad {\n        return None;\n    }\n}')
        bmb = 'Some(o) => { self.err("E0001", "a {"); self.bump(); },'
        self.assertEqual(port_replay.without(bmb, "E0001"), 'Some(o) => { self.bump(); },')
        # Not a statement of its own: a `let`, an arm's value, a code reported twice.
        self.assertIsNone(port_replay.without('let d = Diag::new("E0001", sp);', "E0001"))
        self.assertIsNone(port_replay.without('match x { None => self.err("E0001", sp), }', "E0001"))
        self.assertIsNone(port_replay.without('a("E0001"); b("E0001");', "E0001"))

    def test_a_history_runs_its_steps_in_one_copy(self):
        tasks = [t for t in pair.load_tasks(os.path.join(HERE, "port")) if t.get("family") == "history"]
        self.assertGreaterEqual(len(tasks), 4)
        for t in tasks:
            names = [c["name"] for st in t["steps"] for c in st["cases"]]
            self.assertEqual(len(names), len(set(names)), t["id"])
            self.assertTrue(all(st["cases"] for st in t["steps"]), t["id"])

    def test_oracles_pass_and_weak_controls_fail(self):
        tasks = {t["id"]: t for t in pair.load_tasks(os.path.join(HERE, "port"))}
        self.assertGreaterEqual(len(tasks), 20)
        for family in ("callers", "diag"):
            t = next(t for t in tasks.values() if t["family"] == family)
            for arm in ("bmb", "rust"):
                self.assertTrue(pair.run_task(t, arm, "oracle")["success"], (t["id"], arm))
                self.assertFalse(pair.run_task(t, arm, "weak0", t["weak"][0][arm])["success"], (t["id"], arm))



class Report(unittest.TestCase):
    def rows(self, d, name, bmb_ok, changed):
        rows = []
        for arm, ok in (("bmb", bmb_ok), ("rust", True)):
            rows.append({"id": "r01-rename-f", "kind": "change", "arm": arm, "agent": "a", "docs": "none",
                         "success": ok, "tokens": 400 if arm == "bmb" else 100, "tool_calls": 1,
                         "changed": changed if arm == "bmb" else ["m::f", "m::g"], "stop": "stop",
                         "model_usage": {"input_tokens": 10, "output_tokens": 10}})
        path = os.path.join(d, name)
        with open(path, "w") as f:
            f.write("".join(json.dumps(r) + "\n" for r in rows))
        return path

    def test_diff_and_worked(self):
        import report
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, "r01-rename-f.json"), "w") as f:
                json.dump({"id": "r01-rename-f", "scope": {"bmb": ["m::f", "m::g"], "rust": ["m::f", "m::g"]}}, f)
            a = self.rows(d, "one.jsonl", True, [])
            b = self.rows(d, "two.jsonl", True, ["m::f", "m::g"])
            text = report.diff([a, b])
            self.assertIn("| refactor | 1/1, 1/1 | 1/1, 1/1 | +0.0 |", text)
            self.assertIn("4.00", text)  # read tokens, BMB / Rust
            w = report.worked(d, [a, b])
            # Judged successful, but the first run's BMB session changed nothing.
            self.assertIn("| one | bmb | 1/1 | 0 | 1 | 0 | 0 |", w)
            self.assertIn("| two | bmb | 1/1 | 1 | 0 | 0 | 0 |", w)
            self.assertIn("| one | rust | 1/1 | 1 | 0 | 0 | 0 |", w)

if __name__ == "__main__":
    unittest.main()
