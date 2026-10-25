#!/usr/bin/env python3
"""BNB: BMB Navigation Bench (ADR-0034 G4, backlog 3A-05).

The same navigation questions over the same code structure, answered in two
arms with the same agent interface and budget rules (bench/m1/harness.py:
at most MAX_TOOL_CALLS tool calls, the `answer` tool counts as one):

  bmb   a bench/m3 workspace (gen.py) through `bmb mcp` (the existing MCP tools)
  rust  its Rust twin (twin.py) through file tools: list_dir, read_file (line
        ranges), grep (regex, one or more paths, context lines)

  bnb.py tasks --write                  regenerate tasks/ (then `manifest --write`)
  bnb.py manifest [--write]             check (or rewrite) the frozen-input hashes
  bnb.py run --sizes 100 1000 [--work DIR] [--agent oracle] [--arms bmb rust]
             [--check-truth] [--cargo-check] [--out F.jsonl] [--expect all-pass]
  bnb.py table F.jsonl..                Markdown tables (per size, per arm, per task)

Ground truth is computed from the BMB maps (`bmb query`, harness truth kinds
where they apply). --check-truth also computes every answer from the Rust
twin's text (analyze.py) and stops on any difference, and compares the whole
structure (call and `&mut` write edges, contract edges, io set, duplicate
groups) between the two.

Agents: `oracle` is a scripted agent per arm that uses the cheapest correct
strategy its tools allow (bmb: `query edges/impact/contracts/effects/
duplicates`; rust: grep + read_file), so its token counts bound an arm's cost
from below. `anthropic:MODEL` runs a real model (paid; AUTONOMY.md §4).

Cost: every tool response counts, tokens = ceil(bytes/4) (ADR-0021); request
bytes (the tool arguments) are recorded separately. Fixed per-session costs
(system prompt, tool descriptions) are not counted.

Python 3 standard library only.
"""
import argparse
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
ROOT = os.path.dirname(BENCH)
sys.path.insert(0, os.path.join(BENCH, "m1"))
sys.path.insert(0, os.path.join(BENCH, "m3"))
sys.path.insert(0, HERE)
import analyze  # noqa: E402
import gen  # noqa: E402
import harness  # noqa: E402
import twin  # noqa: E402

MAX_TOOL_CALLS = harness.MAX_TOOL_CALLS
DEPTH = "64"  # `edges` depth for closures: longer than any call chain here
REASONS = ["changed", "calls", "uses", "names"]


# ---------- frozen inputs ----------

def frozen_files():
    files = [os.path.join(HERE, "prompt-bmb.md"), os.path.join(HERE, "prompt-rust.md")]
    d = os.path.join(HERE, "tasks")
    return files + sorted(os.path.join(d, f) for f in os.listdir(d))


def manifest_text():
    return "".join(f"{hashlib.sha256(open(f, 'rb').read()).hexdigest()}  {os.path.relpath(f, HERE)}\n"
                   for f in frozen_files())


def check_manifest():
    path = os.path.join(HERE, "MANIFEST")
    if not os.path.exists(path) or open(path).read() != manifest_text():
        sys.exit("bnb: frozen inputs differ from MANIFEST: refusing to run (ADR-0021 §1)")


def load_tasks():
    d = os.path.join(HERE, "tasks")
    return [json.load(open(os.path.join(d, f))) for f in sorted(os.listdir(d))]


# ---------- workspaces ----------

def build(work, n):
    """BMB workspace and Rust twin of size n (same gen.py parameters)."""
    ws, tw = os.path.join(work, f"bmb{n:05d}"), os.path.join(work, f"rust{n:05d}")
    subprocess.run([sys.executable, os.path.join(BENCH, "m3", "gen.py"), ws, "--defs", str(n)],
                   check=True, capture_output=True)
    subprocess.run([sys.executable, os.path.join(HERE, "twin.py"), tw, "--defs", str(n)],
                   check=True, capture_output=True)
    return ws, tw


def cargo_check(tw):
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(tw, "target"), RUSTFLAGS="-D warnings")
    t = time.perf_counter()
    p = subprocess.run(["cargo", "check", "--quiet", "--offline", "--manifest-path", os.path.join(tw, "Cargo.toml")],
                       capture_output=True, text=True, env=env)
    return p.returncode, round(time.perf_counter() - t, 2), p.stderr[-2000:]


# ---------- ground truth from the BMB maps ----------

def bmb_rows(ws, *args):
    _, out = harness.bmb("query", *args, "--", ws)
    return harness.rows(out)


def htruth(t, ws):
    return harness.truth({"truth": t}, ws)


def truth_bmb(task, ws):
    op, d = task["op"], task.get("def")
    if op == "callers":
        return htruth({"query": ["edges", d, "in", "1"], "kinds": ["calls"], "field": "from",
                       "closure": bool(task.get("closure"))}, ws)
    if op == "callees":
        found = htruth({"query": ["edges", d, "out", "1"], "kinds": ["calls"], "field": "to",
                        "closure": bool(task.get("closure"))}, ws)
        return {x for x in found if "::" in x}  # built-in print_i64 is not a definition
    if op == "impact":
        return htruth({"query": ["impact", d, "iface"], "reasons": REASONS, "field": "path"}, ws)
    if op == "requires":
        return htruth({"query": ["contracts", d], "list": "required_by"}, ws)
    if op == "ensures":
        return htruth({"query": ["contracts", d], "list": "ensured_by"}, ws)
    if op == "writers":
        return htruth({"query": ["edges", d, "in", "1"], "kinds": ["writes"], "field": "from"}, ws)
    if op == "writes_closure":
        return htruth({"query": ["edges", d, "out", "1"], "kinds": ["writes"], "field": "to", "closure": True}, ws)
    if op == "io_all":
        return {r["def"] for r in bmb_rows(ws, "effects") if r.get("effect") == "io"}
    if op == "io_callers":
        return truth_bmb({"op": "callers", "def": d, "closure": True}, ws) & truth_bmb({"op": "io_all"}, ws)
    if op == "duplicates":
        return next((set(r["defs"]) - {d} for r in bmb_rows(ws, "duplicates") if "hash" in r and d in r["defs"]), set())
    raise ValueError(op)


def structure_bmb(ws):
    _, out = harness.bmb("map", "L2", ws)
    edges, contract = set(), set()
    for r in harness.rows(out):
        if r.get("kind") == "calls" and "::" in r["to"]:
            edges.add((r["from"], r["to"], "calls", None))
        elif r.get("kind") == "writes":
            edges.add((r["from"], r["to"], "writes", r["arg"]))
        elif r.get("kind") == "contract":
            contract.add((r["from"], r["to"]))
    groups = sorted(sorted(r["defs"]) for r in bmb_rows(ws, "duplicates") if "hash" in r)
    return {"edges": edges, "contract": contract, "io": truth_bmb({"op": "io_all"}, ws), "dup_groups": groups}


def structure_rust(g):
    # BMB `contract` edges also cover loop invariants (`// invariant:` here)
    contract = {(p, c) for p, v in g["defs"].items() for c in v["requires"] + v["ensures"] + v["invariants"]}
    return {"edges": g["edges"], "contract": contract, "io": analyze.truth({"op": "io_all"}, g),
            "dup_groups": g["dup_groups"]}


def check_truth(tasks, ws, tw):
    """Truth per task from both sides; returns (truths, list of differences)."""
    g = analyze.analyze(tw)
    diffs = []
    sb, sr = structure_bmb(ws), structure_rust(g)
    for k in sb:
        if sb[k] != sr[k]:
            a, b = (set(map(tuple, sb[k])), set(map(tuple, sr[k]))) if k == "dup_groups" else (sb[k], sr[k])
            diffs.append({"structure": k, "only_bmb": sorted(map(str, a - b))[:10],
                          "only_rust": sorted(map(str, b - a))[:10]})
    truths = {}
    for t in tasks:
        tb, tr = truth_bmb(t, ws), analyze.truth(t, g)
        truths[t["id"]] = tb
        if tb != tr:
            diffs.append({"task": t["id"], "only_bmb": sorted(tb - tr), "only_rust": sorted(tr - tb)})
    return truths, diffs


# ---------- Rust arm: file tools over the twin ----------

RUST_TOOLS = [
    {"name": "list_dir", "description": "List a directory of the crate (directories end with /).",
     "inputSchema": {"type": "object", "properties": {"path": {"type": "string", "description": "relative to the crate root; default ."}}}},
    {"name": "read_file", "description": "Read a file, optionally a line range (1-based, inclusive). Lines are prefixed with their number and a tab.",
     "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}, "start_line": {"type": "integer"},
                                                       "end_line": {"type": "integer"}}, "required": ["path"]}},
    {"name": "grep", "description": "Search files with a regular expression (Python re syntax), line by line. "
                                    "Output: `path:line:text` for matches, `path-line-text` for context lines, `--` between "
                                    "separate groups. `path` is a file or directory (recursive) or a list of them; default the whole crate.",
     "inputSchema": {"type": "object", "properties": {
         "pattern": {"type": "string"},
         "path": {"anyOf": [{"type": "string"}, {"type": "array", "items": {"type": "string"}}]},
         "context_before": {"type": "integer"}, "context_after": {"type": "integer"}}, "required": ["pattern"]}},
]


class RustTools:
    def __init__(self, root):
        self.root = os.path.realpath(root)
        self.tools = RUST_TOOLS

    def _path(self, p):
        full = os.path.realpath(os.path.join(self.root, p or "."))
        if full != self.root and not full.startswith(self.root + os.sep):
            raise ValueError(f"path outside the crate: {p}")
        return full

    def _files(self, p):
        full = self._path(p)
        if os.path.isfile(full):
            return [full]
        out = []
        for d, dirs, files in os.walk(full):
            dirs[:] = sorted(x for x in dirs if x != "target" and not x.startswith("."))
            out += [os.path.join(d, f) for f in sorted(files)]
        return sorted(out)

    def call(self, name, args):
        try:
            return getattr(self, "t_" + name)(**args), False
        except Exception as e:  # noqa: BLE001 (any failure is the agent's error message)
            return f"error: {e}", True

    def t_list_dir(self, path="."):
        full = self._path(path)
        return "\n".join(x + ("/" if os.path.isdir(os.path.join(full, x)) else "")
                         for x in sorted(os.listdir(full)) if x != "target") + "\n"

    def t_read_file(self, path, start_line=1, end_line=None):
        lines = open(self._path(path), encoding="utf-8").read().split("\n")
        if lines and lines[-1] == "":
            lines.pop()
        end = len(lines) if end_line is None else min(end_line, len(lines))
        return "".join(f"{i}\t{lines[i - 1]}\n" for i in range(max(1, start_line), end + 1))

    def t_grep(self, pattern, path=".", context_before=0, context_after=0):
        rx = re.compile(pattern)
        paths = path if isinstance(path, list) else [path]
        files = sorted({f for p in paths for f in self._files(p)})
        out = []
        for f in files:
            lines = open(f, encoding="utf-8").read().split("\n")
            if lines and lines[-1] == "":
                lines.pop()
            hits =[i for i, x in enumerate(lines) if rx.search(x)]
            if not hits:
                continue
            rel = os.path.relpath(f, self.root)
            shown, last = set(hits), None
            for i in hits:
                for j in range(max(0, i - context_before), min(len(lines), i + context_after + 1)):
                    shown.add(j)
            for j in sorted(shown):
                if last is not None and j != last + 1 and (context_before or context_after):
                    out.append("--")
                out.append(f"{rel}{':' if j in hits else '-'}{j + 1}{':' if j in hits else '-'}{lines[j]}")
                last = j
        return "\n".join(out) + "\n" if out else "(no matches)\n"

    def close(self):
        pass


class BmbTools:
    def __init__(self, ws):
        self.mcp = harness.Mcp(ws)
        self.tools = self.mcp.tools

    def call(self, name, args):
        return self.mcp.call(name, args)

    def close(self):
        self.mcp.close()


# ---------- oracle agents ----------

class GenAgent:
    """Adapter: a generator that yields lists of (tool, args) and receives the
    texts of their results, as the harness agent interface."""

    def __init__(self, g):
        self.g = g
        self.usage = {"input_tokens": 0, "output_tokens": 0}
        self.started = False

    def next_calls(self, last):
        try:
            if not self.started:
                self.started = True
                return next(self.g)
            return self.g.send([text for _, _, text, _ in last])
        except StopIteration:
            return []


def call(name, args):
    return (yield [(name, args)])[0]


def rows(text):
    return [json.loads(x) for x in text.splitlines() if x.startswith("{")]


def bfs(start, step):
    return analyze.closure(start, step)


def oracle_bmb(task):
    op, d = task["op"], task.get("def")

    def edges(direction, depth):
        return [r for r in rows((yield from call("query", {"name": "edges", "args": [d, direction, depth]})))
                if "from" in r]

    if op in ("callers", "writers"):
        es = yield from edges("in", "1" if not task.get("closure") else DEPTH)
        kind = "writes" if op == "writers" else "calls"
        step = lambda x: {e["from"] for e in es if e["to"] == x and e["kind"] == kind}  # noqa: E731
        ans = bfs(d, step) if task.get("closure") else step(d)
    elif op == "callees":
        es = yield from edges("out", DEPTH)
        ans = {x for x in bfs(d, lambda x: {e["to"] for e in es if e["from"] == x and e["kind"] == "calls"}) if "::" in x}
    elif op == "writes_closure":
        # level by level: a deep `edges` would also return every call and
        # contract edge below the written callees
        ans, frontier = set(), [d]
        while frontier:
            nxt = []
            for x in frontier:
                out = rows((yield from call("query", {"name": "edges", "args": [x, "out", "1"]})))
                for e in out:
                    if e.get("kind") == "writes" and e["from"] == x and e["to"] not in ans and e["to"] != d:
                        ans.add(e["to"])
                        nxt.append(e["to"])
            frontier = nxt
    elif op == "impact":
        ans = {r["path"] for r in rows((yield from call("query", {"name": "impact", "args": [d, "iface"]})))
               if r.get("reason") in REASONS}
    elif op in ("requires", "ensures"):
        r = [x for x in rows((yield from call("query", {"name": "contracts", "args": [d]}))) if "pred" in x]
        ans = set(r[0]["required_by" if op == "requires" else "ensured_by"]) if r else set()
    elif op == "io_all":
        ans = {r["def"] for r in rows((yield from call("query", {"name": "effects", "args": []}))) if "effect" in r}
    elif op == "io_callers":
        es = yield from edges("in", DEPTH)
        cl = bfs(d, lambda x: {e["from"] for e in es if e["to"] == x and e["kind"] == "calls"})
        io = set()
        if cl:
            io = {r["def"] for r in rows((yield from call("query", {"name": "effects", "args": []}))) if "effect" in r}
        ans = cl & io
    elif op == "duplicates":
        groups = rows((yield from call("query", {"name": "duplicates", "args": []})))
        ans = next((set(r["defs"]) - {d} for r in groups if "hash" in r and d in r["defs"]), set())
    else:
        raise ValueError(op)
    yield [("answer", {"defs": sorted(ans)})]


GREP_LINE = re.compile(r"^(.+?)([:-])(\d+)[:-](.*)$")
SKIP = ("debug_assert!", "//")


def unit_of(path):
    return path.split("::")[0]


def file_of(unit):
    return f"src/{unit}.rs"


class RustOracle:
    """Cheapest correct strategy with grep + read_file that we found, using
    what an agent learns from the first file it reads: module mU is
    src/mU.rs, every item starts with `pub fn` at column 0 and ends with `}`
    at column 0, other modules are imported as modules (`use crate::{m0,
    ..}`), so a call is `name(` inside the module or `mU::name(` outside it,
    and contracts are the `/// Requires:` / `/// Ensures:` lines and
    `debug_assert!` lines."""

    def __init__(self, task):
        self.task = task
        self.hdr = {}  # file -> [(line, name, header text)]

    def grep(self, pattern, path="src", before=0, after=0):
        args = {"pattern": pattern, "path": path}
        if before:
            args["context_before"] = before
        if after:
            args["context_after"] = after
        text = yield from call("grep", args)
        out = []
        for x in text.split("\n"):
            m = GREP_LINE.match(x)
            if m:
                out.append((m.group(1), int(m.group(3)), m.group(4), m.group(2) == ":"))
        self.last = out
        return out

    def headers(self, files):
        need = sorted({f for f in files if f not in self.hdr})
        if need:
            for f in need:
                self.hdr[f] = []
            for f, ln, text, _ in (yield from self.grep(r"^pub fn ", need if len(need) > 1 else need[0])):
                self.hdr[f].append((ln, re.match(r"pub fn (\w+)\(", text).group(1), text))

    def enclosing(self, f, ln):
        best = None
        for hl, name, _ in self.hdr[f]:
            if hl <= ln:
                best = name
        return f"{f[4:-3]}::{best}"

    def names(self, unit):
        return {n for _, n, _ in self.hdr.get(file_of(unit), [])}

    def read_fn(self, path):
        f = file_of(unit_of(path))
        yield from self.headers([f])
        hs = self.hdr[f]
        i = next(k for k, h in enumerate(hs) if h[1] == path.split("::")[1])
        args = {"path": f, "start_line": hs[i][0]}
        if i + 1 < len(hs):
            args["end_line"] = hs[i + 1][0] - 1
        text = yield from call("read_file", args)
        lines = [x.split("\t", 1)[1] for x in text.split("\n") if "\t" in x]
        return lines[:lines.index("}") + 1]

    def calls_in(self, unit, lines):
        """[(callee path, args, line)] of exec calls in a body."""
        out = []
        for line in lines[1:]:
            if line.strip().startswith(SKIP):
                continue
            for m in analyze.CALL.finditer(line):
                to = analyze.resolve(unit, self.names(unit), m)
                if to:
                    out.append((to, analyze.call_args(line, m.end() - 1), line))
        return out

    def level_callers(self, targets):
        by_unit = {}
        for t in targets:
            by_unit.setdefault(unit_of(t), set()).add(t.split("::")[1])
        qual = "|".join(f"{u}::(?:{'|'.join(sorted(ns))})" for u, ns in sorted(by_unit.items()))
        hits = []
        for f, ln, text, _ in (yield from self.grep(rf"(?<![\w:])(?:{qual})\(")):
            if not text.strip().startswith(SKIP):
                for m in re.finditer(r"(m\d+)::(\w+)\(", text):
                    if f"{m.group(1)}::{m.group(2)}" in targets:
                        hits.append((f, ln))
        own = [file_of(u) for u in sorted(by_unit)]
        bare = "|".join(sorted({n for ns in by_unit.values() for n in ns}))
        for f, ln, text, _ in (yield from self.grep(rf"(?<![\w.:])(?:{bare})\(", own if len(own) > 1 else own[0])):
            u = f[4:-3]
            if text.startswith("pub fn ") or text.strip().startswith(SKIP):
                continue
            if any(m.group(2) in by_unit[u] for m in analyze.CALL.finditer(text) if not m.group(1)):
                hits.append((f, ln))
        yield from self.headers({f for f, _ in hits})
        return {self.enclosing(f, ln) for f, ln in hits}

    def callers_closure(self, start):
        seen, frontier = set(), {start}
        while frontier:
            found = yield from self.level_callers(frontier)
            frontier = found - seen - {start}
            seen |= frontier
        return seen

    def io_all(self):
        direct = set()
        # the header is usually within a few lines above the output call
        found = yield from self.grep(r"println!", before=3)
        for f, ln, text, hit in found:
            if hit:
                heads = [(cl, t) for cf, cl, t, _ in found if cf == f and ln - 3 <= cl < ln and t.startswith("pub fn ")]
                if heads:
                    name = re.match(r"pub fn (\w+)\(", max(heads)[1]).group(1)
                    direct.add(f"{f[4:-3]}::{name}")
                else:
                    yield from self.headers([f])
                    direct.add(self.enclosing(f, ln))
        seen, frontier = set(direct), set(direct)
        while frontier:
            found = yield from self.level_callers(frontier)
            frontier = found - seen
            seen |= frontier
        return seen

    def run(self):
        t, op = self.task, self.task["op"]
        d = t.get("def")
        if op == "callers":
            ans = (yield from self.callers_closure(d)) if t.get("closure") else (yield from self.level_callers({d}))
        elif op == "impact":
            # a contract change reaches the definition and its direct callers
            ans = {d} | (yield from self.level_callers({d}))
        elif op == "writers":
            callers = yield from self.level_callers({d})
            yield from self.headers([file_of(unit_of(d))])
            sig = next(h for _, n, h in self.hdr[file_of(unit_of(d))] if n == d.split("::")[1])
            ans = callers if "&mut " in sig else set()  # only a `&mut` value can be passed to a `&mut` parameter
        elif op in ("callees", "writes_closure"):
            ans, frontier, seen = set(), [d], {d}
            while frontier:
                yield from self.headers({file_of(unit_of(x)) for x in frontier})
                nxt = []
                for x in frontier:
                    lines = yield from self.read_fn(x)
                    calls = self.calls_in(unit_of(x), lines)
                    if op == "writes_closure":
                        yield from self.headers({file_of(unit_of(c)) for c, _, _ in calls})
                        muts = {p.split(": ")[0] for p in analyze.split_args(re.match(r"pub fn \w+\((.*)\)", lines[0]).group(1))
                                if ": &mut " in p}
                        keep = []
                        for c, args, _ in calls:
                            sig = next(h for _, n, h in self.hdr[file_of(unit_of(c))] if n == c.split("::")[1])
                            ps = analyze.split_args(re.match(r"pub fn \w+\((.*)\)", sig).group(1))
                            if any(i < len(ps) and ": &mut " in ps[i] and a in muts for i, a in enumerate(args)):
                                keep.append(c)
                    else:
                        keep = [c for c, _, _ in calls]
                    for c in keep:
                        if c not in seen:
                            seen.add(c)
                            ans.add(c)
                            nxt.append(c)
                frontier = nxt
        elif op in ("requires", "ensures"):
            key = "Requires" if op == "requires" else "Ensures"
            u, n = d.split("::")
            hits = [(f, ln) for f, ln, _, _ in (yield from self.grep(rf"^/// {key}: .*(?<![\w.:]){n}\(", file_of(u)))]
            hits += [(f, ln) for f, ln, _, _ in (yield from self.grep(rf"^/// {key}: .*(?<![\w:]){u}::{n}\("))]
            yield from self.headers({f for f, _ in hits})
            ans = set()
            for f, ln in hits:
                hl, name, _ = next(h for h in self.hdr[f] if h[0] > ln)
                ans.add(f"{f[4:-3]}::{name}")
        elif op == "io_all":
            ans = yield from self.io_all()
        elif op == "io_callers":
            cl = yield from self.callers_closure(d)
            ans = (cl & (yield from self.io_all())) if cl else set()
        elif op == "duplicates":
            ans = yield from self.duplicates(d)
        else:
            raise ValueError(op)
        yield [("answer", {"defs": sorted(ans)})]

    def normal(self, unit, lines):
        name = re.match(r"pub fn (\w+)\(", lines[0]).group(1)
        out = [lines[0].replace(f"pub fn {name}(", "pub fn _(", 1)]
        for line in lines[1:]:
            out.append(analyze.CALL.sub(lambda m: (analyze.resolve(unit, self.names(unit), m) or m.group(0)[:-1]) + "(", line))
        return out

    def duplicates(self, d):
        lines = yield from self.read_fn(d)
        me = self.normal(unit_of(d), lines)
        groups = yield from self.grep("^" + re.escape(lines[1]) + "$", before=1, after=len(lines) - 2)
        by = {}
        for f, ln, text, _ in groups:
            by.setdefault(f, {})[ln] = text
        cands = []
        for f, ls in by.items():
            for ln in sorted(ls):
                cand = [ls.get(ln + k) for k in range(len(lines))]
                if None in cand or not cand[0].startswith("pub fn ") or cand[1] != lines[1]:
                    continue
                cands.append((f, cand))
        # resolving bare calls needs the module's item names
        need = {f for f, c in cands if any(not m.group(1) for x in c[1:] for m in analyze.CALL.finditer(x))}
        yield from self.headers(need | ({file_of(unit_of(d))}))
        ans = set()
        for f, c in cands:
            u = f[4:-3]
            path = f"{u}::{re.match(r'pub fn (\w+)', c[0]).group(1)}"
            if path != d and self.normal(u, c) == me:
                ans.add(path)
        return ans


def make_agent(kind, arm, task):
    if kind == "oracle":
        if arm == "bmb":
            return GenAgent(oracle_bmb(task))
        o = RustOracle(task)
        return GenAgent(o.run())
    if kind.startswith("anthropic:"):
        from anthropic_agent import AnthropicAgent  # noqa: E402
        prompt = open(os.path.join(HERE, f"prompt-{arm}.md")).read()
        return AnthropicAgent(kind.split(":", 1)[1], prompt, task["prompt"])
    sys.exit(f"unknown agent {kind}")


# ---------- sessions ----------

def run_task(task, arm, agent_kind, root, want):
    tools = BmbTools(root) if arm == "bmb" else RustTools(root)
    agent = make_agent(agent_kind, arm, task)
    if hasattr(agent, "set_tools"):
        agent.set_tools(tools.tools + [harness.ANSWER_TOOL])
    m = {"tool_calls": 0, "resp_bytes": 0, "req_bytes": 0, "limit_hit": False, "calls_by_tool": {}}
    answer, last = None, []
    t0 = time.perf_counter()
    try:
        while True:
            calls = agent.next_calls(last)
            if not calls:
                break
            last = []
            for name, args in calls:
                if m["tool_calls"] >= MAX_TOOL_CALLS:
                    m["limit_hit"] = True
                    break
                m["tool_calls"] += 1
                m["calls_by_tool"][name] = m["calls_by_tool"].get(name, 0) + 1
                m["req_bytes"] += len(json.dumps(args, ensure_ascii=False).encode())
                if name == "answer":
                    answer = args.get("defs", [])
                    text, err = '{"answered":true}', False
                else:
                    text, err = tools.call(name, args)
                    m["resp_bytes"] += len(text.encode())
                last.append((name, args, text, err))
            if m["limit_hit"]:
                break
    finally:
        tools.close()
    got = set(answer or [])
    tp = len(want & got)
    precision = tp / len(got) if got else (1.0 if not want else 0.0)
    recall = tp / len(want) if want else 1.0
    return {"id": task["id"], "op": task["op"], "arm": arm, "agent": agent_kind, **m,
            "tokens": math.ceil(m["resp_bytes"] / 4), "req_tokens": math.ceil(m["req_bytes"] / 4),
            "precision": round(precision, 3), "recall": round(recall, 3),
            "success": precision == 1.0 and recall == 1.0, "truth_size": len(want),
            "answered": answer is not None, "wall_s": round(time.perf_counter() - t0, 3),
            "model_usage": agent.usage, "stop": getattr(agent, "stop", None)}


# ---------- tasks (written once, then frozen) ----------

def make_tasks():
    """Questions about units m1 and m2 (present from N=100 on) plus three
    whole-program ones. Roles are chosen on the plan's graph at N=1000 (the
    neighbourhood of m1 and m2 is the same in every size >= 340, gen.py
    locality); the files are then frozen."""
    tmp = tempfile.mkdtemp(prefix="bnb-tasks-")
    try:
        tw = os.path.join(tmp, "t")
        twin.write(tw, gen.parse([tw, "--defs", "1000"]))
        g = analyze.analyze(tw)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    args = gen.parse(["-", "--defs", "20"])
    tasks = []

    def add(slug, op, prompt, **kw):
        tasks.append({"id": f"n{len(tasks) + 1:02d}-{slug}", "kind": "navigate", "op": op, **kw,
                      "prompt": prompt + " Use the `answer` tool with the full paths (module::name) of all such definitions."})

    def ncallers(p, closure=False):
        return len(analyze.truth({"op": "callers", "def": p, "closure": closure}, g))

    for u in (1, 2):
        unit = gen.Unit(u, args)
        p = lambda n: f"m{u}::{n}"  # noqa: E731
        leaves = [p(n) for n, k, _ in unit.defs if k == "leaf"]
        leaf = max(leaves, key=lambda x: (ncallers(x, True), x))
        mids = [p(n) for n, k, i in unit.defs if k == "chain" and i["layer"] == 1]
        mid = max(mids, key=lambda x: (ncallers(x), x))
        mid2 = max([p(n) for n, k, i in unit.defs if k == "chain" and i["layer"] == 2], key=lambda x: (ncallers(x), x))
        top = next(p(n) for n, k, i in unit.defs if k == "chain" and i["layer"] == args.depth - 1)
        show = next(p(n) for n, k, _ in unit.defs if k == "show")
        fill = next(p(n) for n, k, _ in unit.defs if k == "fill")
        reset = next(p(n) for n, k, _ in unit.defs if k == "reset")
        add(f"m{u}-callers", "callers", f"Which functions directly call `{leaf}`?", **{"def": leaf})
        add(f"m{u}-callers-transitive", "callers",
            f"Which functions call `{leaf}` directly or transitively (through other functions)?", **{"def": leaf, "closure": True})
        add(f"m{u}-callers-mid", "callers", f"Which functions directly call `{mid}`?", **{"def": mid})
        add(f"m{u}-callees-transitive", "callees",
            f"Which functions does `{top}` call, directly or transitively? Count only calls in function bodies, "
            "not contract predicates and not built-in output.", **{"def": top, "closure": True})
        add(f"m{u}-callees-show", "callees",
            f"Which functions does `{show}` call, directly or transitively? Count only calls in function bodies, "
            "not contract predicates and not built-in output.", **{"def": show, "closure": True})
        for x, slug in ((leaf, "impact-leaf"), (mid2, "impact-mid")):
            add(f"m{u}-{slug}", "impact",
                f"Suppose the contract (precondition or postcondition) of `{x}` changes. Which functions must be "
                f"checked again, including `{x}` itself? A function relies on the contracts of the functions it "
                "calls directly; nothing further up is affected.", **{"def": x})
        add(f"m{u}-requires", "requires", f"Which functions have the predicate `m{u}::inb` in their precondition?",
            **{"def": f"m{u}::inb"})
        add(f"m{u}-ensures", "ensures", f"Which functions have the predicate `m{u}::all_inb` in their postcondition?",
            **{"def": f"m{u}::all_inb"})
        add(f"m{u}-writers", "writers", f"Which functions pass a mutable reference (`&mut`) argument to `{fill}`?",
            **{"def": fill})
        add(f"m{u}-writes-transitive", "writes_closure",
            f"Which functions receive the mutable reference (`&mut`) argument of `{reset}`, directly or "
            f"transitively (passed on from function to function)? Do not count `{reset}` itself.", **{"def": reset})
        add(f"m{u}-io-callers", "io_callers",
            f"Which functions that produce output (print, directly or through the functions they call) call "
            f"`{leaf}` directly or transitively?", **{"def": leaf})
    add("io-all", "io_all", "Which functions produce output (print), directly or through the functions they call?")
    add("dup-inb", "duplicates",
        "Which other definitions are exact duplicates of `m1::inb` (same parameters, types and body; only the "
        "name or module differs)?", **{"def": "m1::inb"})
    post = next(f"m{u}::{n}" for u in range(1, 5) for n, k, i in gen.Unit(u, args).defs
                if k == "post" and i["tmpl"] == "abs")
    add("dup-post", "duplicates",
        f"Which other definitions are exact duplicates of `{post}` (same parameters, types and body; only the "
        "name or module differs)?", **{"def": post})
    return tasks


# ---------- commands ----------

def machine():
    rev = subprocess.run(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    dirty = bool(subprocess.run(["git", "-C", ROOT, "status", "--porcelain"], capture_output=True, text=True).stdout.strip())
    return {"kind": "machine", "commit": rev + ("+dirty" if dirty else ""), "bmb": harness.bmb_bin(),
            "manifest": hashlib.sha256(open(os.path.join(HERE, "MANIFEST"), "rb").read()).hexdigest()[:16]}


def cmd_run(a):
    check_manifest()
    tasks = load_tasks()
    if a.tasks:
        tasks = [t for t in tasks if t["id"] in set(a.tasks.split(","))]
    work = a.work or tempfile.mkdtemp(prefix="bnb-")
    os.makedirs(work, exist_ok=True)
    out = [machine()]
    failed = []
    for n in a.sizes:
        t = time.perf_counter()
        ws, tw = build(work, n)
        row = {"kind": "build", "size": n, "build_s": round(time.perf_counter() - t, 2),
               "bmb_bytes": sum(os.path.getsize(os.path.join(ws, "src", f)) for f in os.listdir(os.path.join(ws, "src"))),
               "rust_bytes": sum(os.path.getsize(os.path.join(tw, "src", f)) for f in os.listdir(os.path.join(tw, "src")))}
        if a.cargo_check:
            row["cargo_check_exit"], row["cargo_check_s"], err = cargo_check(tw)
            if row["cargo_check_exit"]:
                print(err, file=sys.stderr)
                failed.append(f"cargo check N={n}")
        if a.check_truth:
            truths, diffs = check_truth(tasks, ws, tw)
            row["truth_diffs"] = diffs
            if diffs:
                failed.append(f"truth differs N={n}: {json.dumps(diffs)[:1500]}")
        else:
            truths = {tk["id"]: truth_bmb(tk, ws) for tk in tasks}
        out.append(row)
        print(json.dumps(row, ensure_ascii=False), flush=True)
        for arm in a.arms:
            for tk in tasks:
                r = run_task(tk, arm, a.agent, ws if arm == "bmb" else tw, truths[tk["id"]])
                r["size"] = n
                out.append(r)
                print(json.dumps(r, ensure_ascii=False), flush=True)
                if a.expect == "all-pass" and (not r["success"] or r["limit_hit"]):
                    failed.append(f"{arm} {tk['id']} N={n}: P {r['precision']} R {r['recall']} limit {r['limit_hit']}")
    if a.out:
        with open(a.out, "a", encoding="utf-8") as f:
            for r in out:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
    if not a.work:
        shutil.rmtree(work, ignore_errors=True)
    if failed:
        sys.exit("bnb: " + "\n".join(failed))


def cmd_table(a):
    rows_ = [json.loads(x) for f in a.files for x in open(f, encoding="utf-8") if x.strip()]
    runs = [r for r in rows_ if "arm" in r]
    sizes = sorted({r["size"] for r in runs})
    arms = [x for x in ("bmb", "rust") if any(r["arm"] == x for r in runs)]
    builds = {r["size"]: r for r in rows_ if r.get("kind") == "build"}
    print("| N | BMB 원문 바이트 | Rust 원문 바이트 | cargo check | 정답 차이 |")
    print("| --- | --- | --- | --- | --- |")
    for n in sizes:
        b = builds.get(n, {})
        cc = f"exit {b['cargo_check_exit']}, {b['cargo_check_s']}s" if "cargo_check_exit" in b else "—"
        td = str(len(b["truth_diffs"])) if "truth_diffs" in b else "—"
        print(f"| {n} | {b.get('bmb_bytes', '—')} | {b.get('rust_bytes', '—')} | {cc} | {td} |")
    print()
    print("| N | 팔 | 성공 | 정밀도 평균 | 재현율 평균 | 응답 토큰 합 | 과제당 토큰 중앙값 | 과제당 토큰 최대 | 도구 호출 합 | 호출 최대 | 한도 도달 |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for n in sizes:
        for arm in arms:
            xs = [r for r in runs if r["size"] == n and r["arm"] == arm]
            if not xs:
                continue
            toks = sorted(r["tokens"] for r in xs)
            print(f"| {n} | {arm} | {sum(r['success'] for r in xs)}/{len(xs)} | "
                  f"{sum(r['precision'] for r in xs) / len(xs):.3f} | {sum(r['recall'] for r in xs) / len(xs):.3f} | "
                  f"{sum(toks)} | {toks[len(toks) // 2]} | {toks[-1]} | {sum(r['tool_calls'] for r in xs)} | "
                  f"{max(r['tool_calls'] for r in xs)} | {sum(r['limit_hit'] for r in xs)} |")
    if len(arms) == 2:
        print()
        head = " | ".join(f"N={n} bmb | N={n} rust | N={n} 비" for n in sizes)
        print(f"| 과제 | 정답 크기(N={sizes[-1]}) | {head} |")
        print("| --- | --- | " + " | ".join("---" for _ in range(3 * len(sizes))) + " |")
        ids = []
        for r in runs:
            if r["id"] not in ids:
                ids.append(r["id"])
        by = {(r["id"], r["arm"], r["size"]): r for r in runs}
        for i in ids:
            cells = []
            for n in sizes:
                b, rr = by.get((i, "bmb", n)), by.get((i, "rust", n))

                def cell(r):
                    if not r:
                        return "—"
                    return f"{r['tokens']} ({r['tool_calls']})" + ("" if r["success"] else " ✗")
                ratio = f"{rr['tokens'] / b['tokens']:.1f}" if b and rr and b["tokens"] else "—"
                cells += [cell(b), cell(rr), ratio]
            last = by.get((i, "bmb", sizes[-1])) or by.get((i, "rust", sizes[-1]))
            print(f"| {i} | {last['truth_size']} | " + " | ".join(cells) + " |")


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    tk = sub.add_parser("tasks")
    tk.add_argument("--write", action="store_true")
    mf = sub.add_parser("manifest")
    mf.add_argument("--write", action="store_true")
    rn = sub.add_parser("run")
    rn.add_argument("--sizes", type=int, nargs="+", default=[100])
    rn.add_argument("--work")
    rn.add_argument("--agent", default="oracle")
    rn.add_argument("--arms", nargs="+", default=["bmb", "rust"], choices=["bmb", "rust"])
    rn.add_argument("--tasks")
    rn.add_argument("--check-truth", action="store_true")
    rn.add_argument("--cargo-check", action="store_true")
    rn.add_argument("--out")
    rn.add_argument("--expect", choices=["all-pass"])
    tb = sub.add_parser("table")
    tb.add_argument("files", nargs="+")
    a = p.parse_args()
    if a.cmd == "tasks":
        tasks = make_tasks()
        if a.write:
            d = os.path.join(HERE, "tasks")
            shutil.rmtree(d, ignore_errors=True)
            os.makedirs(d)
            for t in tasks:
                with open(os.path.join(d, t["id"] + ".json"), "w", encoding="utf-8") as f:
                    f.write(json.dumps(t, ensure_ascii=False, indent=2) + "\n")
        for t in tasks:
            print(t["id"], t.get("def", ""))
    elif a.cmd == "manifest":
        if a.write:
            open(os.path.join(HERE, "MANIFEST"), "w").write(manifest_text())
        else:
            check_manifest()
            print("MANIFEST ok")
    elif a.cmd == "run":
        cmd_run(a)
    else:
        cmd_table(a)


if __name__ == "__main__":
    main()
