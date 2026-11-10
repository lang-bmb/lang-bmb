#!/usr/bin/env python3
"""Gate of the port-twin harness (backlog 3P-01): the item splitter, the Rust
tools' sandbox, and `pair.py check` on the fixture (twins agree on the
corpus, every oracle succeeds, every weak control fails its check)."""
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


if __name__ == "__main__":
    unittest.main()
