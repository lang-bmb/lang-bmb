#!/usr/bin/env python3
"""Refactoring tasks on the port twin (backlog 3P-03, ADR-0104: H-REFACTOR).

    port_change.py make COMMIT     write bench/twin/port/r*.json for that commit
    port_change.py check           the committed tasks are what `make` writes for
                                   their commit

A task asks for a change that keeps the program's behaviour. It succeeds
when three things hold (`pair.py`): the arm's program builds (BMB: every
obligation proved; Rust: `cargo build` and `cargo test`), it prints the
reference's bytes on the fixture's corpus, and nothing outside the task's
scope changed (functions and methods, `index.py`).

Families:

    rename   a function both sides have under one name gets a new name
             and every use follows
    swap     the first two parameters of such a function (after `self`)
             change places, in its declaration and in every call

Each task carries a reference solution per arm (`oracle`, generated here,
not written by hand; the families of `port_hand.py` are, as replacements of
text that must fit the commit) and two weak solutions that must fail: one also edits
a function outside the scope, one also changes the program's output.

The choice of targets is mechanical, as in `port_tasks.py`: the candidates
that meet the family's rule in sorted order, a seeded shuffle, at most
`PER_FILE` per Rust file.
"""
import argparse
import json
import os
import random
import re
import subprocess
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import index  # noqa: E402
import pair  # noqa: E402
import port_hand  # noqa: E402
import port_tasks  # noqa: E402

OUT = port_tasks.OUT
SEED = 20261117
COUNT = {"rename": 10, "swap": 6}
PER_FILE = {"rename": 3, "swap": 2}
# A reference solution touches this many functions at most (a session has
# sixty tool calls) and at least two (the function and one use).
USES = (2, 6)

# Targets left out after the shuffle (the others keep their places), with
# the reason (found by `pair.py check`).
EXCLUDE = {
    "clone_blocked": "the Rust function declares a function inside it with the same parameter names: "
                     "the cut of argument lists takes the inner declaration for a call",
}

PROMPT = {
    "rename": "Rename the function `{target}` to `{new}` and update every use of it. The program's behaviour "
              "must not change; change nothing else.",
    "swap": "Change the signature of the function `{target}`: its first two parameters{after} change places "
            "(`{a}` and `{b}`). Update every call of it. The program's behaviour must not change; change nothing "
            "else.",
}


def word(name):
    return r"(?<![\w])" + re.escape(name) + r"(?![\w])"


def edits_for(text, pattern, new):
    """[(old, new)] for `edit_file`: each replaces one stretch of whole
    lines that occurs once in the file as it is when its turn comes, and
    together they replace every match of `pattern`."""
    out = []
    pat = re.compile(pattern)
    while True:
        lines = text.split("\n")
        at = next((i for i, ln in enumerate(lines) if pat.search(ln)), None)
        if at is None:
            return out
        lo, hi = at, at + 1
        while text.count("\n".join(lines[lo:hi])) != 1:
            if hi < len(lines):
                hi += 1
            elif lo > 0:
                lo -= 1
            else:
                raise ValueError("no unique stretch")
        old = "\n".join(lines[lo:hi])
        fresh = pat.sub(new, old)
        out.append((old, fresh))
        text = text.replace(old, fresh, 1)


def edits_between(old, new):
    """[(old, new)] for `edit_file` that turn the text `old` into `new`:
    one per changed stretch of lines, each widened until it occurs once in
    the file as it is when its turn comes."""
    import difflib
    a, b = old.split("\n"), new.split("\n")
    out, shift, cur = [], 0, list(a)
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        lo, hi = i1 + shift, i2 + shift
        fresh = b[j1:j2]
        text = "\n".join(cur)
        while hi == lo or text.count("\n".join(cur[lo:hi])) != 1:
            if hi < len(cur):
                fresh = fresh + [cur[hi]]
                hi += 1
            elif lo > 0:
                lo -= 1
                fresh = [cur[lo]] + fresh
            else:
                raise ValueError("no unique stretch")
        out.append(("\n".join(cur[lo:hi]), "\n".join(fresh)))
        cur[lo:hi] = fresh
        shift += len(fresh) - (hi - lo)
    assert "\n".join(cur) == new
    return out


def arguments(text, at):
    """The spans of the arguments of the list that opens at `text[at]`
    (a `(`), and the index of its `)`. None when the list is not plain
    (a closure's `|`, an unbalanced bracket)."""
    assert text[at] == "("
    depth, i, start, spans = 0, at, at + 1, []
    while i < len(text):
        c = text[i]
        if c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif c == "'":
            m = re.compile(r"'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^\\'\n])'").match(text, i)
            i = m.end() - 1 if m else i
        elif c == "|":
            return None
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                if text[start:i].strip():
                    spans.append((start, i))
                return spans, i
        elif c == "<" and text.startswith("::<", i - 2):
            j = text.find(">", i)
            if j < 0 or re.search(r"[(\[{<]", text[i + 1:j]):
                return None
            i = j
        elif c == "," and depth == 1:
            spans.append((start, i))
            start = i + 1
        i += 1
    return None


def swapped(text, name):
    """`text` with the first two parameters of `name` (after `self`) and
    the matching two arguments of every call exchanged, and the two
    parameters' names; None when a declaration or a call is not plain."""
    lists = []  # (argument spans, how many lead)
    names = None
    for m in re.finditer(index.call_pattern(name), text):
        at = m.end() - 1
        got = arguments(text, at)
        if got is None:
            return None
        spans, _ = got
        head = text[max(0, m.start() - 3):m.start()]
        decl = head == "fn "
        first = text[spans[0][0]:spans[0][1]].strip() if spans else ""
        receiver = re.fullmatch(r"(?:&(?:mut )?|mut )?self(?:: .*)?", first, re.S) is not None
        if decl:
            lead = 1 if receiver else 0
            method = receiver
            if len(spans) < lead + 2:
                return None
            names = tuple(text[a:b].strip().split(":")[0].replace("mut ", "").strip() for a, b in spans[lead:lead + 2])
        else:
            # `x.f(a, b)` has no receiver in the list; `T::f(x, a, b)` has.
            lead = None if not head.endswith("::") else "path"
        lists.append((spans, decl, lead))
    if names is None:
        return None
    method = any(d and lead == 1 for _, d, lead in lists)
    out, pieces = text, []
    for spans, decl, lead in lists:
        skip = lead if decl else (1 if lead == "path" and method else 0)
        if len(spans) < skip + 2:
            return None
        pieces.append((spans[skip], spans[skip + 1]))
    for (a0, a1), (b0, b1) in sorted(pieces, reverse=True):
        x, y = out[a0:a1], out[b0:b1]
        lx, ly = x[:len(x) - len(x.lstrip())], y[:len(y) - len(y.lstrip())]
        out = out[:a0] + lx + y.strip() + out[a1:b0] + ly + x.strip() + out[b1:]
    return out, names


def unit_of(path):
    return (path[1:] if path.startswith("<") else path).split("::")[0]


def bmb_code(paths, texts, heads):
    """The `code` of one `write`: the definitions `paths` of one unit,
    methods inside their `impl` blocks."""
    free = [texts[p] for p in paths if p not in heads]
    blocks = defaultdict(list)
    for p in paths:
        if p in heads:
            blocks[heads[p]].append(texts[p])
    parts = free + [head + "\n" + "\n\n".join(ms) + "\n    }" for head, ms in sorted(blocks.items())]
    return "\n\n".join(parts) + "\n"


class Twin:
    """The two sides at one commit, as a session finds them."""

    def __init__(self, commit):
        self.tree = port_tasks.tree_of(commit)
        self.r = index.rust_index(self.tree)
        self.heads = {}
        self.b = index.bmb_index(self.tree, rest=True, heads=self.heads)
        self.bf = {p: t for p, t in self.b.items() if not p.endswith("::<rest>")}
        # The crate as the Rust arm gets it in the measured documentation
        # condition (`none`, ADR-0144: no comment but `//!`).
        src = os.path.join(self.tree, "compiler", "src")
        self.files = {}
        for f in index.rust_files(src):
            with open(os.path.join(src, f), encoding="utf-8") as h:
                self.files[f] = pair.strip_comments(h.read())
        self.rall = index.rust_index(self.tree, index.rust_files(src), rest=True)
        # The words of each text: a name is looked for only where it occurs.
        words = lambda texts: {k: set(re.findall(r"\w+", t)) for k, t in texts.items()}  # noqa: E731
        self.fwords, self.rwords, self.bwords = words(self.files), words(self.rall), words(self.b)

    def scopes(self, name):
        """The definitions of each side that have the word `name`."""
        return (sorted(p for p in self.rall if name in self.rwords[p]),
                sorted(p for p in self.b if name in self.bwords[p]))

    def files_with(self, name):
        return [(f, self.files[f]) for f in sorted(self.files) if name in self.fwords[f]]

    def clean(self, name):
        """Every occurrence of the word `name`, on both sides, is the
        function's declaration or a call of it: a rename is then the
        replacement of the word."""
        call = index.call_pattern(name)
        for text in ("\n".join(t for _, t in self.files_with(name)),
                     "\n".join(self.b[p] for p in self.b if name in self.bwords[p])):
            words = len(re.findall(word(name), text))
            calls = len(re.findall(call, text))
            decls = len(re.findall(r"fn " + re.escape(name) + r"\s*(?:<[^>]*>)?\(", text))
            if decls != 1 or words != calls:
                return False
        return True


def weak(arm, twin, kind):
    """Calls that spoil a solution: `scope` edits a function no task
    names, `corpus` changes what every diagnostic prints."""
    if arm == "rust":
        old, new = {"scope": ('String::from("sha256:")', '"sha256:".to_string()'),
                    "corpus": ('\\"severity\\":', '\\"sev\\":')}[kind]
        f = {"scope": "sha256.rs", "corpus": "diag.rs"}[kind]
        return [["edit_file", {"path": "src/" + f, "old": o, "new": n}]
                for o, n in edits_for(twin.files[f], re.escape(old), new.replace("\\", "\\\\"))]
    if kind == "scope":
        text = re.sub(word("code"), "digit_code", twin.b["sha256::push_digit"])
        return [["write", {"unit": "sha256", "code": text + "\n"}]]
    text = twin.b["diag::render"].replace('\\"severity\\":', '\\"sev\\":')
    return [["write", {"unit": "diag", "code": text + "\n"}]]


def rename(twin, name, rk, bk):
    new = name + "_renamed"
    sub = lambda t: re.sub(word(name), new, t)  # noqa: E731
    r_scope, b_scope = twin.scopes(name)
    if any(p.endswith("::<rest>") for p in r_scope + b_scope):
        return None
    if not (USES[0] <= len(r_scope) <= USES[1] and USES[0] <= len(b_scope) <= USES[1]):
        return None
    r_new, b_new = rk[:-len(name)] + new, bk[:-len(name)] + new
    r_calls = []
    for f, text in twin.files_with(name):
        r_calls += [["edit_file", {"path": "src/" + f, "old": o, "new": n}]
                    for o, n in edits_for(text, word(name), new)]
    # BMB: the unit of the function first (the new name is added and the
    # uses in that unit follow; the old one stays until nothing calls it),
    # then the other units, then the old name goes.
    texts = {(b_new if p == bk else p): sub(twin.b[p]) for p in b_scope}
    heads = dict(twin.heads)
    if bk in heads:
        heads[b_new] = heads[bk]
    units = defaultdict(list)
    for p in texts:
        units[unit_of(p)].append(p)
    home = unit_of(bk)
    b_calls = [["write", {"unit": u, "code": bmb_code(sorted(units[u]), texts, heads)}]
               for u in [home] + sorted(u for u in units if u != home)]
    b_calls.append(["write", {"unit": home, "remove": [bk]}])
    return {"names": {"rust": {"target": rk, "new": new}, "bmb": {"target": bk, "new": new}},
            "scope": {"rust": sorted(r_scope + [r_new]), "bmb": sorted(b_scope + [b_new])},
            "oracle": {"rust": r_calls, "bmb": b_calls}}


def swap(twin, name, rk, bk):
    r_scope, b_scope = twin.scopes(name)
    if any(p.endswith("::<rest>") for p in r_scope + b_scope):
        return None
    if not (USES[0] <= len(r_scope) <= USES[1] and USES[0] <= len(b_scope) <= USES[1]):
        return None
    # Rust: the files as they are afterwards, as edits.
    r_calls, r_names = [], None
    for f, text in twin.files_with(name):
        got = swapped(text, name)
        if got is None:
            # A file with calls only has no declaration: the names come
            # from the declaring file.
            got = swapped_calls(text, name, twin.files, rk)
            if got is None:
                return None
        new_text, names = got
        r_names = r_names or names
        r_calls += [["edit_file", {"path": "src/" + f, "old": o, "new": n}] for o, n in edits_between(text, new_text)]
    # BMB: every definition of the scope as one text, so the declaration is
    # seen with its calls; then cut back into definitions.
    sep = "\n\x00\n"
    got = swapped(sep.join(twin.b[p] for p in b_scope), name)
    if got is None or r_names is None:
        return None
    b_texts = dict(zip(b_scope, got[0].split(sep)))
    b_names = got[1]
    units = defaultdict(list)
    for p in b_scope:
        units[unit_of(p)].append(p)
    home = unit_of(bk)
    if len(units) == 1:
        b_calls = [["write", {"unit": home, "code": bmb_code(b_scope, b_texts, twin.heads)}]]
        extra = []
    else:
        # `write` takes one unit at a time and the program must check after
        # each: the new signature goes in under another name, the calls move
        # to it unit by unit, the old one goes; then the same again back to
        # the name.
        tmp = name + "_swapped"
        ren = lambda t, a, b: re.sub(word(a), b, t)  # noqa: E731
        heads = dict(twin.heads)
        b_tmp = bk[:-len(name)] + tmp
        if bk in heads:
            heads[b_tmp] = heads[bk]
        one = {(b_tmp if p == bk else p): ren(b_texts[p], name, tmp) for p in b_scope}
        two = {p: b_texts[p] for p in b_scope}
        order = [home] + sorted(u for u in units if u != home)
        key = lambda u, texts: sorted(p for p in texts if unit_of(p) == u)  # noqa: E731
        b_calls = [["write", {"unit": u, "code": bmb_code(key(u, one), one, heads)}] for u in order]
        b_calls.append(["write", {"unit": home, "remove": [bk]}])
        b_calls += [["write", {"unit": u, "code": bmb_code(key(u, two), two, heads)}] for u in order]
        b_calls.append(["write", {"unit": home, "remove": [b_tmp]}])
        extra = []
    return {"names": {"rust": {"target": rk, "a": r_names[0], "b": r_names[1],
                               "after": " after `self`" if "::" in rk[rk.index("::") + 2:] else ""},
                      "bmb": {"target": bk, "a": b_names[0], "b": b_names[1],
                              "after": " after `self`" if bk in twin.heads else ""}},
            "scope": {"rust": r_scope, "bmb": sorted(b_scope + extra)},
            "oracle": {"rust": r_calls, "bmb": b_calls}}


def swapped_calls(text, name, files, rk):
    """`swapped` for a file that only calls `name`: the declaration is
    read from the file that has it."""
    decl = next((t for t in files.values() if re.search(r"fn " + re.escape(name) + r"\s*(?:<[^>]*>)?\(", t)), None)
    if decl is None:
        return None
    m = re.search(r"fn " + re.escape(name) + r"\s*(?:<[^>]*>)?\(", decl)
    got = arguments(decl, m.end() - 1)
    if got is None:
        return None
    head = decl[m.start():got[1] + 1]
    both = swapped(head + "\n\x00\n" + text, name)
    if both is None:
        return None
    return both[0].split("\n\x00\n", 1)[1], both[1]


def candidates(twin):
    out = {k: [] for k in COUNT}
    names = sorted({index.simple(p) for p in twin.r} & {index.simple(p) for p in twin.bf})
    for n in names:
        if not (index.unique(twin.r, n) and index.unique(twin.bf, n) and twin.clean(n)):
            continue
        rk = next(p for p in twin.r if index.simple(p) == n)
        bk = next(p for p in twin.bf if index.simple(p) == n)
        t = rename(twin, n, rk, bk)
        if t:
            out["rename"].append((rk.split("::")[0], t))
        t = swap(twin, n, rk, bk)
        if t:
            out["swap"].append((rk.split("::")[0], t))
    return out


def make(commit):
    commit = subprocess.run(["git", "rev-parse", commit], cwd=ROOT, capture_output=True, text=True,
                            check=True).stdout.strip()
    twin = Twin(commit)
    cands = candidates(twin)
    out = {}
    for family in COUNT:
        pool = list(cands[family])
        random.Random(f"{SEED}:{family}").shuffle(pool)
        seen, picked = defaultdict(int), []
        for mod, t in pool:
            if len(picked) == COUNT[family]:
                break
            if index.simple(t["names"]["rust"]["target"]) in EXCLUDE:
                continue
            if seen[mod] >= PER_FILE[family]:
                continue
            seen[mod] += 1
            picked.append(t)
        if len(picked) < COUNT[family]:
            sys.exit(f"{family}: only {len(picked)} candidates of {COUNT[family]}")
        for t in picked:
            tid = f"r{len(out) + 1:02d}-{family}-{index.simple(t['names']['rust']['target']).lower()}"
            task = {
                "id": tid,
                "kind": "change",
                "family": family,
                "fixture": "port",
                "commit": commit,
                "prompt": {arm: PROMPT[family].format(**t["names"][arm]) for arm in ("bmb", "rust")},
                "scope": t["scope"],
                "oracle": t["oracle"],
                "weak": [{"fails": k, **{arm: t["oracle"][arm] + weak(arm, twin, k) for arm in ("bmb", "rust")}}
                         for k in ("scope", "corpus")],
            }
            out[tid + ".json"] = json.dumps(task, indent=1, ensure_ascii=False) + "\n"
    me = sys.modules[__name__]
    for t in port_hand.tasks(twin, me):
        tid = f"r{len(out) + 1:02d}-{t['family']}-{t['name']}"
        task = {"id": tid, "kind": "change", "family": t["family"], "fixture": "port", "commit": commit,
                "prompt": t["prompt"], "scope": t["scope"], "oracle": t["oracle"],
                "weak": [{"fails": k, **{arm: t["oracle"][arm] + weak(arm, twin, k) for arm in ("bmb", "rust")}}
                         for k in ("scope", "corpus")]}
        out[tid + ".json"] = json.dumps(task, indent=1, ensure_ascii=False) + "\n"
    return out


def cmd_make(a):
    files = make(a.commit)
    for f in os.listdir(OUT):
        if re.fullmatch(r"r\d\d-.*\.json", f):
            os.remove(os.path.join(OUT, f))
    for name, text in files.items():
        with open(os.path.join(OUT, name), "w", encoding="utf-8") as h:
            h.write(text)
    print(f"wrote {len(files)} tasks to {os.path.relpath(OUT, ROOT)}")


def cmd_check(a):
    have = {f: open(os.path.join(OUT, f), encoding="utf-8").read() for f in sorted(os.listdir(OUT))
            if re.fullmatch(r"r\d\d-.*\.json", f)}
    if not have:
        sys.exit("no tasks")
    commits = {json.loads(t)["commit"] for t in have.values()}
    if len(commits) != 1:
        sys.exit(f"tasks of several commits: {sorted(commits)}")
    want = make(commits.pop())
    problems = [f"{f}: differs from what `make` writes" for f in sorted(set(have) | set(want))
                if have.get(f) != want.get(f)]
    for p in problems:
        print(p, file=sys.stderr)
    if problems:
        sys.exit(1)
    print(f"ok: {len(have)} tasks are what `make` writes")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    mk = sub.add_parser("make")
    mk.add_argument("commit")
    sub.add_parser("check")
    a = ap.parse_args()
    {"make": cmd_make, "check": cmd_check}[a.cmd](a)


if __name__ == "__main__":
    main()
