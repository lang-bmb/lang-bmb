#!/usr/bin/env python3
"""Definitions of the port twin, one per function or method (backlog 3P-02).

The twin is stage0's components C0~C4 (`compiler/src`, Rust) and their port
(`selfhost/`, BMB). A comprehension question's answer is a set of definitions,
so both sides need the same grain: a free function, or a method of an `impl`
block. `pair.py`'s splitter takes a whole `impl` block as one item, which is
too coarse for stage0 (most of the checker is `impl Checker`).

Paths:
    BMB   unit::name, unit::Type::name, <unit::Type as unit::Trait>::name
          (the paths `bmb map` prints; `check` compares the two)
    Rust  module::name, module::Type::name (a method of `impl Type` or of
          `impl Trait for Type`); the module is the file under `src/` with
          `::` for `/` (`check::text::Checker::text_arg`)

Both sides are formatter output (`bmb fmt`, rustfmt), so indentation decides
where an item starts and ends. Test modules (`#[cfg(test)]`) are left out.
"""
import os
import re

# stage0's files of the components the port has (dev-docs/health/components.json,
# C0~C4), without `escape.rs`: it is not ported yet.
RUST_FILES = ["source.rs", "diag.rs", "sha256.rs", "json.rs", "lexer.rs", "parser.rs", "ast.rs", "fmt.rs",
              "identity.rs", "check.rs", "check/text.rs", "moves.rs", "consts.rs", "closures.rs", "refs.rs",
              "stdlib.rs"]

FN = r"(?:pub(?:\([a-z]+\))? )?(?:const )?(?:unsafe )?fn ([A-Za-z_]\w*)"
BMB_FN = r"(?:spec |proof )?fn ([A-Za-z_]\w*)"


def strip_comments(text):
    """Rust text without `//` comments (string contents are kept: a
    diagnostic code is a string)."""
    out = []
    for ln in text.split("\n"):
        s = re.sub(r'"(?:\\.|[^"\\])*"', lambda m: m.group(0).replace("//", "\x00\x00"), ln)
        out.append(s.split("//", 1)[0].replace("\x00\x00", "//").rstrip())
    return "\n".join(out)


def _items(lines, starts, rest=None):
    """{path: text}: `starts(i, line)` gives (path, indent) where an item
    begins; it ends at the first later line that is `indent}` (or on its own
    line when it has no body). With `rest` (a list), an item takes the
    attribute lines right above it and every other line goes to `rest`."""
    out, i = {}, 0
    while i < len(lines):
        hit = starts(i, lines[i])
        if hit is None:
            if rest is not None:
                rest.append(lines[i])
            i += 1
            continue
        path, ind = hit
        attrs = []
        while rest and rest[-1].startswith(ind + "#["):
            attrs.insert(0, rest.pop())
        j = i
        while j < len(lines):
            ln = lines[j]
            one_line = j == i and ln.rstrip().endswith("}") and ln.count("{") == ln.count("}") and "{" in ln
            no_body = ln.rstrip().endswith(";") and "{" not in "".join(lines[i:j + 1])
            if ln == ind + "}" or one_line or no_body:
                break
            j += 1
        out[path] = "\n".join(attrs + lines[i:j + 1])
        i = j + 1
    return out


def rust_files(src):
    """Every `.rs` file under `src`, as paths relative to it."""
    out = []
    for d, dirs, files in os.walk(src):
        dirs.sort()
        out += [os.path.relpath(os.path.join(d, f), src) for f in sorted(files) if f.endswith(".rs")]
    return sorted(out)


def rust_index(root, files=RUST_FILES, src=os.path.join("compiler", "src"), rest=False):
    """{path: text without comments} of the functions and methods of
    `root/src/FILE`. With `rest`, a function has its attributes and each
    file's other lines are `module::<rest>` (what `pair.py` compares to
    find what a session changed)."""
    out = {}
    for f in files:
        mod = f[:-3].replace(os.sep, "::").replace("/", "::")
        with open(os.path.join(root, src, f), encoding="utf-8", errors="replace") as h:
            text = strip_comments(h.read())
        lines = text.split("#[cfg(test)]")[0].split("\n")
        other = [] if rest else None
        owner = [None]

        def starts(i, ln):
            m = re.match(r"impl(?:<[^>]*>)? (?:[A-Za-z_][\w:]*(?:<[^>]*>)? for )?([A-Za-z_]\w*)", ln)
            if m:
                owner[0] = m.group(1)
                return None
            if ln == "}":
                owner[0] = None
                return None
            m = re.match(FN, ln)
            if m:
                return f"{mod}::{m.group(1)}", ""
            m = re.match("    " + FN, ln) if owner[0] else None
            if m:
                return f"{mod}::{owner[0]}::{m.group(1)}", "    "
            return None

        out.update(_items(lines, starts, other))
        if rest:
            out[f"{mod}::<rest>"] = "\n".join(x for x in other if x.strip())
    return out


def bmb_index(root, d="selfhost", rest=False, heads=None):
    """{path: text} of the functions and methods of `root/d/*.bmb`. With
    `rest`, as `rust_index`; a summary is no part of any text (ADR-0117:
    no hash reads it). `heads` (a dict) receives, per method, the line
    that opens its `impl` block."""
    out = {}
    src = os.path.join(root, d)
    for f in sorted(os.listdir(src)):
        if not f.endswith(".bmb"):
            continue
        with open(os.path.join(src, f), encoding="utf-8") as h:
            lines = [ln for ln in h.read().split("\n") if not re.match(r"\s*#\[summary\(", ln)] if rest \
                else h.read().split("\n")
        state = {"unit": None, "owner": None}
        other = [] if rest else None

        def starts(i, ln):
            m = re.match(r"mod (\w+) \{", ln)
            if m:
                state["unit"] = m.group(1)
                return None
            m = re.match(r"    impl(?:<[^>]*>)? (?:([\w:]+) for )?(\w+)", ln)
            if m:
                state["owner"] = (m.group(2), m.group(1))
                state["head"] = ln
                return None
            if ln == "    }":
                state["owner"] = None
                return None
            unit = state["unit"]
            if state["owner"] is None:
                m = re.match("    " + BMB_FN, ln)
                return (f"{unit}::{m.group(1)}", "    ") if m else None
            m = re.match("        " + BMB_FN, ln)
            if not m:
                return None
            ty, tr = state["owner"]
            full = tr if tr is None or "::" in tr else f"{unit}::{tr}"
            path = f"{unit}::{ty}::{m.group(1)}" if tr is None else f"<{unit}::{ty} as {full}>::{m.group(1)}"
            if heads is not None:
                heads[path] = state["head"]
            return path, "        "

        out.update(_items(lines, starts, other))
        if rest:
            out[f"{state['unit']}::<rest>"] = "\n".join(x for x in other if x.strip())
    return out


def simple(path):
    return path.rsplit("::", 1)[-1]


def mentions(index, pattern, own=None):
    """The definitions whose text matches `pattern`. `own` (a path) only
    counts when the match is after its first line (a function does not call
    itself by being declared)."""
    pat = re.compile(pattern)
    out = set()
    for path, text in index.items():
        body = text.split("\n", 1)[1] if path == own and "\n" in text else ("" if path == own else text)
        if pat.search(body):
            out.add(path)
    return out


def call_pattern(name):
    """A call of the function or method `name`: `name(`, `name::<T>(`."""
    return r"(?<![\w])" + re.escape(name) + r"\s*(?:::<[^>]*>)?\("


def callers(index, path):
    """The definitions that call `path`, by its name in their text. Exact
    only when the name is one definition's (`unique`) and no field or local
    has it; `port_tasks.py check` compares the BMB side with the map."""
    return mentions(index, call_pattern(simple(path)), own=path)


def unique(index, name):
    return sum(1 for p in index if simple(p) == name) == 1
