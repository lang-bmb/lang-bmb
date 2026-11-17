#!/usr/bin/env python3
"""Refactoring tasks whose reference solutions are written by hand (backlog
3P-03): the families no rule of `port_change.py` can solve on both sides.

    inline    a function goes; its body takes the place of every call
    extract   a part of a function becomes a new function that it calls
    move      a function goes to another unit and every call follows

A solution is written here as replacements of text in the twin's files at
the tasks' commit; each must match exactly once, so a solution that no
longer fits its commit stops `make` and is not silently dropped. The tool
calls (`edit_file`, `write`) and the scopes are derived from them.
"""
import os
import re
import tempfile
from collections import defaultdict

import index


def close(text, at):
    """The index of the bracket that closes `text[at]` (strings skipped)."""
    depth, i = 0, at
    while i < len(text):
        c = text[i]
        if c == '"':
            i += 1
            while text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError("unbalanced")


def calls_replaced(text, name, fresh):
    """`text` with every call `name(ARG)` replaced by `fresh(ARG)` (a
    function of the one argument's text); the declaration stays."""
    out, at = "", 0
    for m in re.finditer(index.call_pattern(name), text):
        if m.start() < at or text[max(0, m.start() - 3):m.start()] == "fn ":
            continue
        end = close(text, m.end() - 1)
        out += text[at:m.start()] + fresh(text[m.end():end])
        at = end + 1
    return out + text[at:]


def once(text, old, new, where):
    if text.count(old) != 1:
        raise ValueError(f"{where}: {text.count(old)} matches of {old!r}")
    return text.replace(old, new, 1)


class Hand:
    """One task being written: the files and definitions as they will be."""

    def __init__(self, twin, change):
        self.twin, self.change = twin, change
        self.files = {}                      # Rust file -> new text
        self.defs, self.heads = {}, {}       # BMB path -> new text; new methods' impl heads
        self.gone = []                       # BMB paths removed
        self.order = []                      # BMB units, in the order of the writes

    # Rust
    def rust(self, f, old=None, new=None, fn=None):
        text = self.files.get(f, self.twin.files[f])
        self.files[f] = fn(text) if fn else once(text, old, new, f)

    # BMB
    def bmb(self, path, old=None, new=None, fn=None):
        text = self.defs.get(path, self.twin.b[path])
        self.defs[path] = fn(text) if fn else once(text, old, new, path)
        self._unit(path)

    def add(self, path, text, head=None):
        self.defs[path] = text
        if head:
            self.heads[path] = head
        self._unit(path)

    def remove(self, path):
        self.gone.append(path)

    def _unit(self, path):
        u = self.change.unit_of(path)
        if u not in self.order:
            self.order.append(u)

    def task(self, family, name, prompt, names):
        ch, twin = self.change, self.twin
        r_calls = []
        for f in sorted(self.files):
            # A removal takes its line break with it (no empty line stays
            # in the function before it).
            r_calls += [["edit_file", {"path": "src/" + f, "old": o if n else o + "\n", "new": n}]
                        for o, n in ch.edits_between(twin.files[f], self.files[f])]
        heads = {**twin.heads, **self.heads}
        units = defaultdict(list)
        for p in self.defs:
            units[ch.unit_of(p)].append(p)
        b_calls = [["write", {"unit": u, "code": ch.bmb_code(sorted(units[u]), self.defs, heads)}]
                   for u in self.order]
        gone = defaultdict(list)
        for p in self.gone:
            gone[ch.unit_of(p)].append(p)
        b_calls += [["write", {"unit": u, "remove": ps}] for u, ps in sorted(gone.items())]
        return {"family": family, "name": name,
                "prompt": {arm: prompt.format(**names[arm]) for arm in ("bmb", "rust")},
                "scope": {"rust": self.rust_scope(), "bmb": sorted(set(self.defs) | set(self.gone))},
                "oracle": {"rust": r_calls, "bmb": b_calls}}

    def rust_scope(self):
        """The functions whose text differs once the files are as written."""
        def indexed(files):
            tmp = tempfile.mkdtemp(prefix="port-hand-")
            src = os.path.join(tmp, "compiler", "src")
            for f, text in files.items():
                os.makedirs(os.path.dirname(os.path.join(src, f)), exist_ok=True)
                with open(os.path.join(src, f), "w", encoding="utf-8") as h:
                    h.write(text)
            return index.rust_index(tmp, sorted(files), rest=True)

        before = indexed({f: self.twin.files[f] for f in self.files})
        after = indexed(self.files)
        out = sorted(p for p in set(before) | set(after) if before.get(p) != after.get(p))
        if any(p.endswith("::<rest>") for p in out):
            raise ValueError(f"a solution changes lines outside functions: {out}")
        return out


INLINE = ("Inline the function `{target}`: replace every call of it by its body and remove the function. The "
          "program's behaviour must not change; change nothing else.")
EXTRACT = ("In the function `{target}`, move {what} into a new function `{new}` in the same {place} and call it "
           "from `{target}`. The program's behaviour must not change; change nothing else.")
MOVE = ("Move the function `{target}` to `{to}` (the same name and body) and update every call of it. The "
        "program's behaviour must not change; change nothing else.")


def inline_type_def(twin, ch):
    h = Hand(twin, ch)
    h.rust("fmt.rs", "            self.type_def(t, depth + 1);", "            self.type_def_as(t, depth + 1, &t.name.name);")
    h.rust("fmt.rs", "    pr.type_def(&t, 0);", "    pr.type_def_as(&t, 0, &t.name.name);")
    h.rust("fmt.rs", "    pub(crate) fn type_def(&mut self, t: &TypeDef, depth: usize) {\n"
                     "        self.type_def_as(t, depth, &t.name.name);\n    }\n\n", "")
    h.bmb("fmt::Printer::module", "self.type_def(&m.types[types[i]], depth.saturating_add(1));",
          "self.type_def_as(&m.types[types[i]], depth.saturating_add(1), m.types[types[i]].name.name.as_str());")
    h.remove("fmt::Printer::type_def")
    return h.task("inline", "type_def", INLINE, {"rust": {"target": "fmt::Printer::type_def"},
                                                 "bmb": {"target": "fmt::Printer::type_def"}})


def inline_sha(twin, ch):
    h = Hand(twin, ch)

    def rust(arg):
        arg = arg.strip()
        assert arg.startswith("&")
        return f"crate::sha256::hex({arg[1:]}.as_bytes())"

    h.rust("identity.rs", fn=lambda t: calls_replaced(t, "sha", rust))
    h.rust("identity.rs", "fn sha(s: &str) -> String {\n    crate::sha256::hex(s.as_bytes())\n}\n\n", "")
    for p in ("identity::Ids::finish", "identity::Ids::method_order"):
        h.bmb(p, fn=lambda t: calls_replaced(t, "sha", lambda a: f"sha256::hex({a.strip()}.as_bytes())"))
    h.remove("identity::sha")
    return h.task("inline", "sha", INLINE, {"rust": {"target": "identity::sha"}, "bmb": {"target": "identity::sha"}})


def extract_reserved_name(twin, ch):
    h = Hand(twin, ch)
    h.rust("check.rs", '        || matches!(n, "f64" | "bool" | "char" | "String" | "str" | "Self")\n}\n',
           '        || primitive_name(n)\n}\n\n'
           'fn primitive_name(n: &str) -> bool {\n'
           '    matches!(n, "f64" | "bool" | "char" | "String" | "str" | "Self")\n}\n')
    cond = 'n == "f64" || n == "bool" || n == "char" || n == "String" || n == "str" || n == "Self"'
    h.add("types::primitive_name", "    #[effects(pure)]\n    fn primitive_name(n: &str) -> bool {\n"
                                   f"        {cond}\n    }}")
    h.bmb("types::reserved_name", cond, "primitive_name(n)")
    what = "the test for the names that are not integer types (`f64`, `bool`, `char`, `String`, `str`, `Self`)"
    return h.task("extract", "reserved_name", EXTRACT,
                  {"rust": {"target": "check::reserved_name", "new": "primitive_name", "what": what, "place": "file"},
                   "bmb": {"target": "types::reserved_name", "new": "primitive_name", "what": what, "place": "unit"}})


def extract_rationale_line(twin, ch):
    h = Hand(twin, ch)
    h.rust("fmt.rs", '            let args: Vec<String> = a.args.iter().map(attr_arg).collect();\n'
                     '            self.line(depth, &format!("#[rationale({})]", args.join(", ")));\n',
           '            self.line(depth, &rationale_text(a));\n')
    h.rust("fmt.rs", "fn attr_arg(x: &AttrArg) -> String {",
           'fn rationale_text(a: &Attr) -> String {\n'
           '    let args: Vec<String> = a.args.iter().map(attr_arg).collect();\n'
           '    format!("#[rationale({})]", args.join(", "))\n}\n\n'
           "fn attr_arg(x: &AttrArg) -> String {")
    h.add("fmt::rationale_text",
          '    #[effects(alloc)]\n    fn rationale_text(a: &ast::Attr) -> String {\n'
          '        let mut row: String = String::from("#[rationale(");\n'
          '        row.push_str(attr_args(a.args.as_slice()).as_str());\n'
          '        row.push_str(")]");\n        row\n    }')
    h.bmb("fmt::Printer::rationale_line",
          '            let mut row: String = String::from("#[rationale(");\n'
          '            row.push_str(attr_args(a.args.as_slice()).as_str());\n'
          '            row.push_str(")]");\n            self.line(depth, row.as_str());',
          '            self.line(depth, rationale_text(a).as_str());')
    what = "the building of the line's text (`#[rationale(..)]`)"
    return h.task("extract", "rationale_line", EXTRACT,
                  {"rust": {"target": "fmt::Printer::rationale_line", "new": "fmt::rationale_text", "what": what,
                            "place": "file"},
                   "bmb": {"target": "fmt::Printer::rationale_line", "new": "fmt::rationale_text", "what": what,
                           "place": "unit"}})


def move_sha(twin, ch):
    h = Hand(twin, ch)
    h.rust("sha256.rs", "\n#[cfg(test)]\n", "\npub fn sha(s: &str) -> String {\n    hex(s.as_bytes())\n}\n\n#[cfg(test)]\n")
    h.rust("identity.rs", fn=lambda t: calls_replaced(t, "sha", lambda a: f"crate::sha256::sha({a})"))
    h.rust("identity.rs", "fn sha(s: &str) -> String {\n    crate::sha256::hex(s.as_bytes())\n}\n\n", "")
    h.add("sha256::sha", "    #[effects(alloc)]\n    fn sha(s: &str) -> String {\n        hex(s.as_bytes())\n    }")
    for p in ("identity::Ids::finish", "identity::Ids::method_order"):
        h.bmb(p, fn=lambda t: calls_replaced(t, "sha", lambda a: f"sha256::sha({a})"))
    h.remove("identity::sha")
    return h.task("move", "sha", MOVE, {"rust": {"target": "identity::sha", "to": "sha256::sha"},
                                        "bmb": {"target": "identity::sha", "to": "sha256::sha"}})


# `move_sha` is written but not a task: after its BMB solution every
# definition is proved and `bmb build` still refuses the program (backlog
# 4D-37).
TASKS = [inline_type_def, inline_sha, extract_reserved_name, extract_rationale_line]


def tasks(twin, change):
    return [f(twin, change) for f in TASKS]
