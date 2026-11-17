#!/usr/bin/env python3
"""Comprehension tasks on the port twin (backlog 3P-02, ADR-0104: H-COMPREHEND).

    port_tasks.py make COMMIT      write bench/twin/port/c*.json for that commit
    port_tasks.py check            the committed tasks are what `make` writes for
                                   their commit, and the text method agrees with
                                   the map where both can answer

The twin is stage0's components C0~C4 (Rust, `compiler/src`) and their port
(BMB, `selfhost/`) at one commit (`index.py`). A question asks for a set of
definitions. Both sides answer the same question, each in its own names: the
port is not a line-by-line copy (630 Rust functions, 1,426 BMB ones), so the
two answers differ in size and in names.

Question families (ADR-0104 decision 1: callers, impact, the path of a
diagnostic, where a state is kept):

    callers   the direct callers of a function both sides have under one name
    impact    its callers up to two levels (who a change of it reaches first)
    diag      the functions that name a diagnostic code in their own text
    state     the functions that change one field of the checker's or the
              printer's state

Where the answers come from (no model, no hand-written answer):

    BMB   callers and impact: the map's call edges (`bmb map L2`), exact.
          diag and state: the definition texts (`index.py`).
    Rust  all four: the definition texts. Callers by name are exact only for
          a name that one definition has and no field or local shares; `make`
          takes only targets whose name is unique on both sides and for which
          the same text method, run on the BMB side, gives exactly the map's
          answer (the method is checked where the truth is known).

The choice of targets is mechanical, to keep the author out of it: every
candidate that meets the family's rule, in sorted order, shuffled by a fixed
seed, at most `PER_FILE` per Rust file and family. The candidates and the rule are
in this file; the seed is `SEED`.

A person checks a sample of the answers before the first run (the backlog
row's condition): `bench/twin/port/REVIEW.md` lists what to look at.
"""
import argparse
import json
import os
import random
import re
import subprocess
import sys
import tempfile
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(os.path.dirname(HERE), "m1"))
import harness  # noqa: E402
import index  # noqa: E402

OUT = os.path.join(HERE, "port")
SEED = 20261117
COUNT = {"callers": 7, "impact": 5, "diag": 7, "state": 5}
# At most this many tasks of a family per Rust file (a diagnostic is filed under the first file that reports it:
# nearly all are the parser's or the checker's).
PER_FILE = {"callers": 2, "impact": 2, "diag": 4, "state": 2}
# Answer sizes a question may have, on both sides: more than one definition
# (a single one is a lookup), few enough to list.
SIZE = {"callers": (3, 8), "impact": (5, 12), "diag": (2, 7), "state": (2, 9)}

# The fields of the checker's and the printer's state both sides have under
# one name, that no other structure of the twin has: (field, Rust owner, BMB
# owner).
STATE = [
    ("open_lets", "check::Checker", "body::Body"),
    ("typed_uses", "check::Checker", "body::Body"),
    ("spec_at", "check::Checker", "body::Body"),
    ("gcalls", "check::Checker", "body::Body"),
    ("self_head", "check::Checker", "decls::Checker"),
    ("binders", "fmt::IdMode", "fmt::Names"),
    ("next_label", "fmt::IdMode", "fmt::Names"),
]

FORMAT = {
    "bmb": "Answer with the full paths as the map shows them (for example `body::Body::call`).",
    "rust": "Answer with `module::function`, or `module::Type::method` for a method of an `impl` block; the module "
            "is the file under `src/` with `::` for `/` (for example `check::Checker::call`, "
            "`check::text::Checker::text_arg`).",
}
PROMPT = {
    "callers": "Which functions call `{target}` directly? Leave out `{target}` itself unless it calls itself.",
    "impact": "Which functions call `{target}` directly, or call one of its direct callers (its callers up to two "
              "levels)? Leave out `{target}` itself unless it is called from one of them.",
    "diag": "Which functions report the diagnostic code `{code}`, that is, write the code in their own body? "
            "Leave out the table that lists every code.",
    "state": "Which functions change the field `{field}` of `{owner}`: assign it, add to it, remove from it, clear "
             "it, or pass it on as `&mut`?",
}


def write_pattern(field):
    """A change of `.field`: an assignment, a mutating method, an element
    assignment, or `&mut x.field`."""
    return (r"\." + field + r"\b\s*(?:=(?!=)|[+\-]=|\[[^\]]*\]\s*=(?!=)|\.(?:insert|push|push_str|extend|entry|remove"
            r"|clear|retain|pop|append|truncate|get_mut|iter_mut)\()|&mut \w+\." + field + r"\b")


def tree_of(commit):
    """The twin's files at `commit`, under a new directory."""
    tmp = tempfile.mkdtemp(prefix="port-tasks-")
    tar = subprocess.run(["git", "archive", commit, "selfhost", "compiler/src"], cwd=ROOT, capture_output=True,
                         check=True).stdout
    subprocess.run(["tar", "-x", "-C", tmp], input=tar, check=True)
    return tmp


def map_calls(tree):
    """{callee: {caller}} of the port, from the map's L2 call edges."""
    files = sorted(os.path.join(tree, "selfhost", f) for f in os.listdir(os.path.join(tree, "selfhost"))
                   if f.endswith(".bmb"))
    r = subprocess.run([harness.bmb_bin(), "map", "L2", *files], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"bmb map failed: {r.stdout[-1000:]}{r.stderr[-1000:]}")
    out = defaultdict(set)
    for ln in r.stdout.splitlines():
        d = json.loads(ln)
        if d.get("kind") == "calls":
            out[d["to"]].add(d["from"])
    return out


def map_fns(tree):
    """The paths of the port's functions, as the map lists them (L0)."""
    d = os.path.join(tree, "selfhost")
    files = sorted(os.path.join(d, f) for f in os.listdir(d) if f.endswith(".bmb"))
    r = subprocess.run([harness.bmb_bin(), "map", "L0", *files], capture_output=True, text=True)
    out = set()
    for ln in r.stdout.splitlines():
        row = json.loads(ln)
        # The library's units are not the port's text.
        if row.get("kind") == "def" and row.get("def_kind") in ("fn", "spec fn", "proof fn") \
                and not row["path"].startswith("std::"):
            out.add(row["path"])
    return out


def two_levels(direct, of):
    out = set(direct)
    for c in direct:
        out |= of(c)
    return out


def candidates(tree):
    """{family: [(Rust file, task fields)]} in sorted order."""
    r, b = index.rust_index(tree), index.bmb_index(tree)
    calls = map_calls(tree)
    by_map = lambda p: {c for c in calls.get(p, set()) if c in b}  # noqa: E731
    out = {k: [] for k in COUNT}

    def fits(family, rs, bs):
        lo, hi = SIZE[family]
        return lo <= len(rs) <= hi and lo <= len(bs) <= hi

    names = sorted({index.simple(p) for p in r} & {index.simple(p) for p in b})
    for n in names:
        if not (index.unique(r, n) and index.unique(b, n)):
            continue
        rk = next(p for p in r if index.simple(p) == n)
        bk = next(p for p in b if index.simple(p) == n)
        rs, bs = index.callers(r, rk), by_map(bk)
        # The text method on the BMB side must give the map's answer.
        if index.callers(b, bk) != bs:
            continue
        mod = rk.split("::")[0]
        if fits("callers", rs, bs):
            out["callers"].append((mod, {"names": {"rust": {"target": rk}, "bmb": {"target": bk}},
                                         "truth": {"rust": sorted(rs), "bmb": sorted(bs)}}))
        # Two levels: every direct caller's name must be as reliable.
        if all(index.unique(r, index.simple(c)) for c in rs) and \
                all(index.callers(b, c) == by_map(c) for c in bs):
            rs2 = two_levels(rs, lambda c: index.callers(r, c))
            bs2 = two_levels(bs, by_map)
            if fits("impact", rs2, bs2):
                out["impact"].append((mod, {"names": {"rust": {"target": rk}, "bmb": {"target": bk}},
                                            "truth": {"rust": sorted(rs2), "bmb": sorted(bs2)}}))
    codes = sorted(set(re.findall(r'"([EW]\d{4})"', " ".join(r.values()))))
    for c in codes:
        lit = '"' + c + '"'
        rs = index.mentions(r, lit)
        bs = {p for p in index.mentions(b, lit) if not p.startswith("diag::")}
        if fits("diag", rs, bs):
            mod = sorted(rs)[0].split("::")[0]
            out["diag"].append((mod, {"names": {"rust": {"code": c}, "bmb": {"code": c}},
                                      "truth": {"rust": sorted(rs), "bmb": sorted(bs)}}))
    for field, ro, bo in STATE:
        rs, bs = index.mentions(r, write_pattern(field)), index.mentions(b, write_pattern(field))
        if fits("state", rs, bs):
            out["state"].append((ro.split("::")[0] + ":" + field,
                                 {"names": {"rust": {"field": field, "owner": ro}, "bmb": {"field": field, "owner": bo}},
                                  "truth": {"rust": sorted(rs), "bmb": sorted(bs)}}))
    return out


def chosen(cands):
    """The tasks: per family the first `COUNT` of the seeded shuffle, at
    most `PER_FILE` per Rust file."""
    tasks = []
    for family in COUNT:
        pool = list(cands[family])
        random.Random(f"{SEED}:{family}").shuffle(pool)
        seen, picked = defaultdict(int), []
        for mod, t in pool:
            if len(picked) == COUNT[family]:
                break
            if seen[mod] >= PER_FILE[family]:
                continue
            seen[mod] += 1
            picked.append(t)
        if len(picked) < COUNT[family]:
            sys.exit(f"{family}: only {len(picked)} candidates of {COUNT[family]}")
        tasks += [(family, t) for t in picked]
    return tasks


def oracle(family, t, arm):
    """A session that reaches the answer with the arm's tools."""
    n = t["names"][arm]
    if arm == "bmb":
        # The map has no text search: the oracle of a `diag` or `state`
        # question only answers.
        if family in ("diag", "state"):
            return [["answer", {"defs": t["truth"][arm]}]]
        first = ["query", {"name": "edges", "args": [n["target"], "in", "1" if family == "callers" else "2"]}]
    else:
        pat = {"callers": lambda: index.call_pattern(index.simple(n["target"])),
               "impact": lambda: index.call_pattern(index.simple(n["target"])),
               "diag": lambda: '"' + n["code"] + '"',
               "state": lambda: r"\." + n["field"] + r"\b"}[family]()
        first = ["grep", {"pattern": pat}]
    return [first, ["answer", {"defs": t["truth"][arm]}]]


def pattern_of(family, names):
    """What a definition's text holds when it is in the answer (for the two
    call families: when it is a direct caller)."""
    if family in ("callers", "impact"):
        return index.call_pattern(index.simple(names["target"]))
    return re.escape('"' + names["code"] + '"') if family == "diag" else write_pattern(names["field"])


def review(tree, commit, tasks):
    """REVIEW.md: per family the first two tasks with, for each definition
    of an answer, the line of its text that puts it there."""
    idx = {"rust": index.rust_index(tree), "bmb": index.bmb_index(tree)}
    out = ["# Port-twin comprehension tasks: the sample a person checks (3P-02)\n",
           f"Generated by `port_tasks.py make` for commit `{commit}`; do not edit. Before the first model run a "
           "person reads this sample: is each listed definition an answer to the question, and is one missing? "
           "The Rust side is `compiler/src` (the files of `index.py`), the BMB side `selfhost/`. For each definition "
           "the line that puts it in the answer is shown. For `impact` the lines are those of the direct callers "
           "(level one); a level-two definition shows the line that calls one of them. The BMB answers of "
           "`callers` and `impact` are the map's call edges; everything else is this text search.\n"]
    shown = defaultdict(int)
    for family, tid, t in tasks:
        if shown[family] == 2:
            continue
        shown[family] += 1
        out.append(f"## {tid}\n")
        for arm in ("rust", "bmb"):
            out.append(f"**{arm}.** {PROMPT[family].format(**t['names'][arm])}\n")
            pat = re.compile(pattern_of(family, t["names"][arm]))
            direct = [d for d in t["truth"][arm] if pat.search(idx[arm][d])]
            second = re.compile("|".join(index.call_pattern(index.simple(d)) for d in direct)) if direct else None
            for d in t["truth"][arm]:
                m = pat.search(idx[arm][d]) or (second.search(idx[arm][d]) if second else None)
                text = idx[arm][d]
                line = text[text.rfind("\n", 0, m.start()) + 1:text.find("\n", m.start())].strip() if m else "?"
                out.append(f"- `{d}`: `{line[:140]}`")
            out.append("")
    return "\n".join(out)


def make(commit):
    commit = subprocess.run(["git", "rev-parse", commit], cwd=ROOT, capture_output=True, text=True,
                            check=True).stdout.strip()
    tree = tree_of(commit)
    out = {}
    listed = []
    for family, t in chosen(candidates(tree)):
        key = t["names"]["rust"].get("target") or t["names"]["rust"].get("code") or t["names"]["rust"]["field"]
        tid = f"c{len(out) + 1:02d}-{family}-{index.simple(key).lower()}"
        task = {
            "id": tid,
            "kind": "comprehend",
            "family": family,
            "fixture": "port",
            "commit": commit,
            "prompt": {arm: PROMPT[family].format(**t["names"][arm]) + " " + FORMAT[arm] for arm in ("bmb", "rust")},
            "truth": t["truth"],
            "oracle": {arm: oracle(family, t, arm) for arm in ("bmb", "rust")},
            # A weak control: one definition short.
            "weak": [{"fails": "answer",
                      **{arm: [["answer", {"defs": t["truth"][arm][1:]}]] for arm in ("bmb", "rust")}}],
        }
        out[tid + ".json"] = json.dumps(task, indent=1, ensure_ascii=False) + "\n"
        listed.append((family, tid, t))
    out["REVIEW.md"] = review(tree, commit, listed)
    return out


def cmd_make(a):
    files = make(a.commit)
    os.makedirs(OUT, exist_ok=True)
    for f in os.listdir(OUT):
        if re.fullmatch(r"c\d\d-.*\.json|REVIEW\.md", f):
            os.remove(os.path.join(OUT, f))
    for name, text in files.items():
        with open(os.path.join(OUT, name), "w", encoding="utf-8") as h:
            h.write(text)
    print(f"wrote {len(files) - 1} tasks and REVIEW.md to {os.path.relpath(OUT, ROOT)}")


def cmd_check(a):
    have = {f: open(os.path.join(OUT, f), encoding="utf-8").read() for f in sorted(os.listdir(OUT))
            if re.fullmatch(r"c\d\d-.*\.json|REVIEW\.md", f)}
    if len(have) < 2:
        sys.exit("no tasks")
    commits = {json.loads(t)["commit"] for f, t in have.items() if f.endswith(".json")}
    if len(commits) != 1:
        sys.exit(f"tasks of several commits: {sorted(commits)}")
    commit = commits.pop()
    want = make(commit)
    # The index names the port's functions as the map does.
    tree = tree_of(commit)
    mine, theirs = set(index.bmb_index(tree)), map_fns(tree)
    problems = [f"index and map disagree on {p}" for p in sorted(mine ^ theirs)][:20]
    problems += [f"{f}: differs from what `make` writes" for f in sorted(set(have) | set(want))
                 if have.get(f) != want.get(f)]
    for p in problems:
        print(p, file=sys.stderr)
    if problems:
        sys.exit(1)
    print(f"ok: {len(have) - 1} tasks and REVIEW.md are what `make` writes; the index has the map's {len(theirs)} functions")


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
