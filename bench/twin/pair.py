#!/usr/bin/env python3
"""Port-twin harness (ADR-0104, backlog 3P-01).

One task, two arms, the same statement, model and tool-call budget:

  bmb   a BMB workspace through `bmb mcp --write-tool` (maps, queries, `write`,
        `verify`, `build`; the transaction tools hidden, ADR-0101)
  rust  the same program as a Rust crate through file tools: list_dir,
        read_file (line ranges), grep, write_file, edit_file, cargo_build,
        cargo_test (no shell)

A twin is a fixture directory with `bmb/*.bmb`, `rust/` (a crate) and
`corpus/NAME.in` with `NAME.out`: both programs read standard input and must
print the `.out` bytes (the differential corpus of the port, ADR-0037). The
port twin of 3P (stage0 and its BMB port) has the same shape; `stk` is the
small fixture the gate runs.

Task kinds (JSON files under tasks/):

  comprehend  `answer` with definition paths (`module::name` in both arms);
              judged by precision and recall against `truth`
  change      a behaviour-preserving refactoring or a feature; judged after the
              session, hidden from the agent:
                1. build: `bmb build` (every obligation proved) / `cargo build`
                   and `cargo test`
                2. corpus: every case prints its expected bytes (the fixture
                   corpus, plus the task's own `cases`)
                3. scope: the definitions that changed, appeared or went are
                   within `scope` (BMB: nameless identity hashes of `bmb hash`;
                   Rust: the text of each top-level item, anything outside an
                   item counts as `module::<rest>`)

Agents: `oracle` replays the task's scripted calls for the arm, and so does
each of its `weak` controls (the gate: every oracle succeeds, every weak
control fails the check it names: answer, build, corpus or scope);
`openai:MODEL` and `anthropic:MODEL` run a real model (ADR-0066).

  pair.py check [--tasks DIR]               twins agree on the corpus, oracles
                                            pass, weak controls fail (the gate)
  pair.py run --agent KIND [--arms bmb rust] [--tasks DIR] [--only ID..]
              [--out F.jsonl]               sessions, one JSON row each

Cost as in BNB: every tool response counts, tokens = ceil(bytes/4) (ADR-0021).

Python 3 standard library only.
"""
import argparse
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
import harness  # noqa: E402

MAX_TOOL_CALLS = 60
RUN_TIMEOUT = 20  # seconds per corpus case
CARGO_TIMEOUT = 600
OUTPUT_LIMIT = 20000
WRITE_LIMIT = 200000
HIDDEN = ["tx_begin", "tx_unit", "tx_put", "tx_remove", "tx_remove_unit", "tx_commit", "tx_abort"]


def bmb_bin():
    return harness.bmb_bin()


# ---------- fixtures ----------

def fixture_dir(name):
    return os.path.join(HERE, "fixtures", name)


def corpus(name):
    """[(case, input bytes, expected bytes)] of a fixture, sorted by name."""
    d = os.path.join(fixture_dir(name), "corpus")
    out = []
    for f in sorted(os.listdir(d)):
        if f.endswith(".in"):
            case = f[:-3]
            with open(os.path.join(d, f), "rb") as a, open(os.path.join(d, case + ".out"), "rb") as b:
                out.append((case, a.read(), b.read()))
    return out


def task_cases(task):
    cases = corpus(task["fixture"]) if task.get("fixture_corpus", True) else []
    for c in task.get("cases", []):
        cases.append((c["name"], c["in"].encode(), c["out"].encode()))
    return cases


def strip_comments(text):
    """Rust source without its comments (ADR-0106): `//` and `///` lines and
    tails, `/* */` blocks (nested) and `/** */`, except the module's inner
    doc `//!`, which mirrors the BMB unit's `#[summary]`. Strings, byte
    strings, raw strings (`r#".."#`), chars and lifetimes are kept as they
    are. A line left empty by a removed comment goes; other lines lose
    their trailing spaces."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if text.startswith("//!", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(text[i:j])
            i = j
        elif text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j < 0 else j
            out.append("\0")  # a removed comment: its line may go
        elif text.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif text.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append("\0")
        elif (m := re.compile(r'b?r(#*)"').match(text, i)) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            end = text.find('"' + m.group(1), m.end())
            end = n if end < 0 else end + 1 + len(m.group(1))
            out.append(text[i:end])
            i = end
        elif c == '"' or (c == "b" and text.startswith('b"', i) and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_"))):
            j = i + (2 if c == "b" else 1)
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            out.append(text[i:j + 1])
            i = j + 1
        elif c == "'":
            # A char (`'x'`, `'\n'`, `'\u{..}'`, `b'x'` reaches here at its
            # quote) or a lifetime (`'a`, no closing quote right after).
            m = re.compile(r"'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^\\'\n])'").match(text, i)
            j = m.end() if m else i + 1
            out.append(text[i:j])
            i = j
        else:
            out.append(c)
            i += 1
    lines = []
    for ln in "".join(out).split("\n"):
        if "\0" in ln:
            ln = ln.replace("\0", "").rstrip()
            if not ln.strip():
                continue
        lines.append(ln)
    return "\n".join(lines)


def strip_crate(crate):
    """Every `.rs` file of `crate` through `strip_comments`."""
    for d, _, files in os.walk(crate):
        for f in files:
            if f.endswith(".rs"):
                p = os.path.join(d, f)
                with open(p, encoding="utf-8") as h:
                    text = h.read()
                with open(p, "w", encoding="utf-8") as h:
                    h.write(strip_comments(text))


# The documentation condition of the Rust arm (ADR-0106): `strip`, the
# measured condition, or `keep`, the control.
RUST_COMMENTS = os.environ.get("PAIR_RUST_COMMENTS", "strip")


def materialize(task, arm, dest):
    """The arm's starting copy of the fixture under `dest`: a workspace
    (`bmb import`) or a crate (its comments stripped unless the condition is
    `keep`, ADR-0106). A task with `commit` takes the fixture from that
    commit (`git archive`), so a task replays a fixed state."""
    src = os.path.join(fixture_dir(task["fixture"]), arm)
    if task.get("commit"):
        rel = os.path.relpath(src, ROOT)
        tmp = tempfile.mkdtemp(prefix="pair-src-")
        tar = subprocess.run(["git", "archive", task["commit"], rel], cwd=ROOT, capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", tmp], input=tar, check=True)
        src = os.path.join(tmp, rel)
    if arm == "bmb":
        files = sorted(os.path.join(src, f) for f in os.listdir(src) if f.endswith(".bmb"))
        r = subprocess.run([bmb_bin(), "import", dest, *files], capture_output=True, text=True)
        if r.returncode != 0:
            raise RuntimeError(f"bmb import failed: {r.stdout}{r.stderr}")
    else:
        shutil.copytree(src, dest, ignore=shutil.ignore_patterns("target"))
        if RUST_COMMENTS == "strip":
            strip_crate(dest)
    return dest


def ws_files(ws):
    d = os.path.join(ws, "src")
    return sorted(os.path.join(d, f) for f in os.listdir(d) if f.endswith(".bmb"))


# ---------- the BMB arm ----------

class BmbTools:
    """`bmb mcp --write-tool DIR`, the official write surface (ADR-0101)."""

    def __init__(self, ws):
        self.p = subprocess.Popen([bmb_bin(), "mcp", "--write-tool", ws], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, text=True)
        self.next = 1
        self.request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                    "clientInfo": {"name": "bmb-pair", "version": "1"}})
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
        self.p.stdin.flush()
        self.tools = [x for x in self.request("tools/list", {})["result"]["tools"] if x["name"] not in HIDDEN]

    def request(self, method, params):
        msg = {"jsonrpc": "2.0", "id": self.next, "method": method, "params": params}
        self.next += 1
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()
        return json.loads(self.p.stdout.readline())

    def call(self, name, args):
        if name not in {t["name"] for t in self.tools}:
            return f"unknown tool {name}", True
        r = self.request("tools/call", {"name": name, "arguments": args})["result"]
        return r["content"][0]["text"], r.get("isError", False)

    def close(self):
        self.p.stdin.close()
        self.p.wait()


# ---------- the Rust arm ----------

def _schema(props, required):
    return {"type": "object", "properties": props, "required": required}


RUST_TOOLS = [
    {"name": "list_dir", "description": "List a directory of the crate (directories end with /).",
     "inputSchema": _schema({"path": {"type": "string", "description": "relative to the crate root; default ."}}, [])},
    {"name": "read_file",
     "description": "Read a file, optionally a line range (1-based, inclusive). Lines are prefixed with their number and a tab.",
     "inputSchema": _schema({"path": {"type": "string"}, "start_line": {"type": "integer"},
                             "end_line": {"type": "integer"}}, ["path"])},
    {"name": "grep",
     "description": "Search files with a regular expression (Python re syntax), line by line. Output: `path:line:text` "
                    "for matches, `path-line-text` for context lines, `--` between separate groups. `path` is a file or "
                    "directory (recursive) or a list of them; default the whole crate.",
     "inputSchema": _schema({"pattern": {"type": "string"},
                             "path": {"anyOf": [{"type": "string"}, {"type": "array", "items": {"type": "string"}}]},
                             "context_before": {"type": "integer"}, "context_after": {"type": "integer"}},
                            ["pattern"])},
    {"name": "write_file", "description": "Create or replace a .rs file under src/ or tests/ with the given content.",
     "inputSchema": _schema({"path": {"type": "string"}, "content": {"type": "string"}}, ["path", "content"])},
    {"name": "edit_file",
     "description": "Replace the one occurrence of `old` in a .rs file under src/ or tests/ with `new` (an error when "
                    "`old` occurs zero or several times).",
     "inputSchema": _schema({"path": {"type": "string"}, "old": {"type": "string"}, "new": {"type": "string"}},
                            ["path", "old", "new"])},
    {"name": "cargo_build", "description": "Run `cargo build --all-targets`. Returns the exit status and compiler output.",
     "inputSchema": _schema({}, [])},
    {"name": "cargo_test",
     "description": "Run `cargo test` (debug build), optionally only tests whose name contains FILTER. Returns the exit "
                    "status and output.",
     "inputSchema": _schema({"filter": {"type": "string"}}, [])},
]


def cargo(crate, target_dir, args, timeout=CARGO_TIMEOUT):
    env = {**os.environ, "CARGO_TARGET_DIR": target_dir, "CARGO_TERM_COLOR": "never", "RUST_BACKTRACE": "0",
           "CARGO_NET_OFFLINE": "true"}
    try:
        r = subprocess.run(["cargo", *args, "--offline"], cwd=crate, capture_output=True, text=True, env=env,
                           timeout=timeout)
        return r.returncode, r.stderr + r.stdout
    except subprocess.TimeoutExpired:
        return -1, f"timed out after {timeout} s"


class RustTools:
    def __init__(self, crate, target_dir):
        self.root = os.path.realpath(crate)
        self.target_dir = target_dir
        self.tools = RUST_TOOLS

    def _path(self, p, write=False):
        if not isinstance(p, str) or os.path.isabs(p):
            raise ValueError(f"not a relative path: {p}")
        full = os.path.realpath(os.path.join(self.root, p or "."))
        if full != self.root and not full.startswith(self.root + os.sep):
            raise ValueError(f"path outside the crate: {p}")
        rel = os.path.relpath(full, self.root).split(os.sep)
        if rel[0] in ("target", ".git"):
            raise ValueError(f"not a crate file: {p}")
        if write and not (rel[0] in ("src", "tests") and full.endswith(".rs")):
            raise ValueError("only .rs files under src/ or tests/ may be written")
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
            return getattr(self, "t_" + name)(**args)
        except Exception as e:  # noqa: BLE001 (any failure is the agent's error message)
            return f"error: {e}", True

    def t_list_dir(self, path="."):
        full = self._path(path)
        return "".join(x + ("/" if os.path.isdir(os.path.join(full, x)) else "") + "\n"
                       for x in sorted(os.listdir(full)) if x not in ("target", ".git")), False

    def t_read_file(self, path, start_line=1, end_line=None):
        lines = open(self._path(path), encoding="utf-8", errors="replace").read().split("\n")
        if lines and lines[-1] == "":
            lines.pop()
        end = len(lines) if end_line is None else min(end_line, len(lines))
        return "".join(f"{i}\t{lines[i - 1]}\n" for i in range(max(1, start_line), end + 1)), False

    def t_grep(self, pattern, path=".", context_before=0, context_after=0):
        rx = re.compile(pattern)
        paths = path if isinstance(path, list) else [path]
        out = []
        for f in sorted({f for p in paths for f in self._files(p)}):
            lines = open(f, encoding="utf-8", errors="replace").read().split("\n")
            if lines and lines[-1] == "":
                lines.pop()
            hits = [i for i, x in enumerate(lines) if rx.search(x)]
            if not hits:
                continue
            rel = os.path.relpath(f, self.root)
            shown, last = set(hits), None
            for i in hits:
                shown.update(range(max(0, i - context_before), min(len(lines), i + context_after + 1)))
            for j in sorted(shown):
                if last is not None and j != last + 1 and (context_before or context_after):
                    out.append("--")
                sep = ":" if j in hits else "-"
                out.append(f"{rel}{sep}{j + 1}{sep}{lines[j]}")
                last = j
        return ("\n".join(out) + "\n" if out else "(no matches)\n"), False

    def t_write_file(self, path, content):
        p = self._path(path, write=True)
        if not isinstance(content, str) or len(content.encode()) > WRITE_LIMIT:
            return f"error: content must be a string of at most {WRITE_LIMIT} bytes", True
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w", encoding="utf-8") as f:
            f.write(content)
        return f"wrote {path} ({len(content.encode())} bytes)\n", False

    def t_edit_file(self, path, old, new):
        p = self._path(path, write=True)
        text = open(p, encoding="utf-8").read()
        n = text.count(old) if old else 0
        if n != 1:
            return f"error: `old` occurs {n} times in {path}", True
        with open(p, "w", encoding="utf-8") as f:
            f.write(text.replace(old, new, 1))
        return f"edited {path}\n", False

    def _cargo(self, args):
        code, out = cargo(self.root, self.target_dir, args)
        if len(out) > OUTPUT_LIMIT:
            out = out[:OUTPUT_LIMIT] + f"\n... ({len(out) - OUTPUT_LIMIT} more characters)"
        return f"exit status: {code}\n{out}", code != 0

    def t_cargo_build(self):
        return self._cargo(["build", "--all-targets"])

    def t_cargo_test(self, filter=None):  # noqa: A002 (the tool's argument name)
        return self._cargo(["test"] + ([filter] if isinstance(filter, str) and filter else []))

    def close(self):
        pass


# ---------- scope: which definitions changed ----------

def bmb_defs(ws):
    """{definition path: its normal-form text} of the workspace. The workspace
    holds normal form only (ADR-0019), so the text of a definition changes
    exactly when its own code does. Not the identity hash of `bmb hash`:
    that one names callees by their identities (ADR-0059 §1), so a changed
    callee changes every caller's identity too, which is not an edit of the
    caller."""
    out = {}
    for f in ws_files(ws):
        with open(f, encoding="utf-8") as h:
            text = h.read()
        m = re.search(r"^mod ([A-Za-z_][A-Za-z0-9_]*) \{$", text, re.M)
        unit = m.group(1) if m else os.path.basename(f)[:-4]
        out.update(split_items(text, unit, "    ", BMB_ITEM))
    return out


ITEM = r"(?:pub(?:\([a-z]+\))? )?(?:unsafe )?(fn|const|static|struct|enum|type|trait) ([A-Za-z_][A-Za-z0-9_]*)"
RUST_ITEM = re.compile("^" + ITEM)
BMB_ITEM = re.compile(r"^(?:spec )?(fn|const|struct|enum|type|trait) ([A-Za-z_][A-Za-z0-9_]*)")


def _item_end(lines, i):
    """The last line of the item that starts at line i: a `;` item or the
    line whose braces close the first `{` (strings and chars skipped)."""
    depth, opened = 0, False
    for j in range(i, len(lines)):
        s = re.sub(r'"(?:\\.|[^"\\])*"', '""', lines[j])
        s = re.sub(r"'(?:\\.|[^'\\])'", "''", s)
        s = s.split("//", 1)[0]
        for c in s:
            if c == "{":
                depth, opened = depth + 1, True
            elif c == "}":
                depth -= 1
        if opened and depth <= 0:
            return j
        if not opened and s.rstrip().endswith(";"):
            return j
    return len(lines) - 1


def split_items(text, module, indent, item):
    """{`module::name`: item text (with its doc and attribute lines)} of the
    items at `indent`, and `module::<rest>` for every other line. `impl`
    blocks are items named `module::impl HEAD`."""
    lines = text.split("\n")
    items, rest, i, docs = {}, [], 0, []
    while i < len(lines):
        ln = lines[i]
        body = ln[len(indent):] if ln.startswith(indent) and not ln.startswith(indent + " ") else None
        if body is not None and (body.startswith("///") or body.startswith("#[")):
            docs.append(ln)
            i += 1
            continue
        m = item.match(body) if body is not None else None
        head = None
        if m:
            head = m.group(2)
        elif body is not None and re.match(r"impl\b", body):
            head = body.rstrip(" {")
        if head is not None:
            end = _item_end(lines, i)
            items[f"{module}::{head}"] = "\n".join(docs + lines[i:end + 1])
            docs, i = [], end + 1
            continue
        rest += docs + [ln]
        docs = []
        i += 1
    items[f"{module}::<rest>"] = "\n".join(x for x in rest if x.strip())
    return items


def rust_items(text, module):
    """The top-level items of one Rust file (an approximation of BMB's
    definitions: macros, `mod` and `use` lines are the rest)."""
    return split_items(text, module, "", RUST_ITEM)


def rust_defs(crate):
    out = {}
    src = os.path.join(crate, "src")
    for d, dirs, files in os.walk(src):
        dirs.sort()
        for f in sorted(files):
            if f.endswith(".rs"):
                rel = os.path.relpath(os.path.join(d, f), src)[:-3].replace(os.sep, "::")
                with open(os.path.join(d, f), encoding="utf-8", errors="replace") as h:
                    # Judged without comments: BMB's normal form drops those an
                    # agent writes, so neither arm's scope counts them (ADR-0106).
                    out.update(rust_items(strip_comments(h.read()), rel))
    return out


def changed(before, after):
    return sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))


def out_of_scope(task, arm, before, after):
    sc = task.get("scope", {})
    allowed = set(sc.get(arm, []))
    new_ok = sc.get("new", False)
    return [k for k in changed(before, after)
            if k not in allowed and not (new_ok and k not in before)
            and not (k.endswith("::<rest>") and not after.get(k) and not before.get(k))]


# ---------- the hidden judge of a change ----------

def run_cases(exe, cases, cwd=None):
    passed, failures = 0, []
    for name, inp, want in cases:
        try:
            r = subprocess.run([exe], input=inp, capture_output=True, timeout=RUN_TIMEOUT, cwd=cwd)
            ok = r.returncode == 0 and r.stdout == want
        except subprocess.TimeoutExpired:
            ok = False
        if ok:
            passed += 1
        else:
            failures.append(name)
    return passed, failures


def judge_change(task, arm, root, before, work):
    out = {"built": False, "tests": None, "corpus_passed": 0, "corpus_total": 0, "corpus_failures": [],
           "out_of_scope": []}
    cases = task_cases(task)
    out["corpus_total"] = len(cases)
    if arm == "bmb":
        exe = os.path.join(work, "judge-exe")
        r = subprocess.run([bmb_bin(), "build", *ws_files(root), "-o", exe], capture_output=True, text=True)
        out["built"] = r.returncode == 0
        after = bmb_defs(root)
    else:
        tgt = os.path.join(work, "judge-target")
        code, _ = cargo(root, tgt, ["build"])
        out["built"] = code == 0
        if out["built"]:
            code, _ = cargo(root, tgt, ["test"])
            out["tests"] = code == 0
        exe = os.path.join(tgt, "debug", task.get("rust_bin", task["fixture"]))
        after = rust_defs(root)
    if out["built"]:
        out["corpus_passed"], out["corpus_failures"] = run_cases(exe, cases)
    out["out_of_scope"] = out_of_scope(task, arm, before, after)
    out["changed"] = changed(before, after)
    out["success"] = (out["built"] and out["tests"] is not False and out["corpus_passed"] == len(cases)
                      and not out["out_of_scope"])
    return out


# ---------- agents ----------

class Scripted:
    """Replays [[tool, args], ..] one call per turn (the oracle and weak arms)."""

    def __init__(self, calls):
        self.calls = list(calls)
        self.usage = {"input_tokens": 0, "output_tokens": 0}

    def next_calls(self, last):
        if not self.calls:
            return []
        name, args = self.calls.pop(0)
        return [(name, args)]


def prompt(arm, task):
    with open(os.path.join(HERE, f"prompt-{arm}.md"), encoding="utf-8") as f:
        system = f.read()
    return system, task["prompt"]


def make_agent(kind, arm, task, script=None):
    if script is not None:
        return Scripted(script)
    if kind == "oracle":
        return Scripted(task["oracle"][arm])
    from model_api import make_model_agent  # noqa: E402
    system, user = prompt(arm, task)
    if agent := make_model_agent(kind, system, user):
        return agent
    sys.exit(f"unknown agent {kind}")


# ---------- sessions ----------

def run_task(task, arm, agent_kind, script=None):
    work = tempfile.mkdtemp(prefix=f"pair-{task['id']}-{arm}-")
    try:
        root = materialize(task, arm, os.path.join(work, "code"))
        before = bmb_defs(root) if arm == "bmb" else rust_defs(root)
        tools = BmbTools(root) if arm == "bmb" else RustTools(root, os.path.join(work, "target"))
        agent = make_agent(agent_kind, arm, task, script)
        comprehend = task["kind"] == "comprehend"
        if hasattr(agent, "set_tools"):
            agent.set_tools(tools.tools + ([harness.ANSWER_TOOL] if comprehend else []))
        m = {"tool_calls": 0, "resp_bytes": 0, "req_bytes": 0, "limit_hit": False, "calls_by_tool": {},
             "tool_errors": 0}
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
                    if name == "answer" and comprehend:
                        answer = args.get("defs", [])
                        text, err = '{"answered":true}', False
                    else:
                        text, err = tools.call(name, args)
                        m["resp_bytes"] += len(text.encode())
                    m["tool_errors"] += int(bool(err))
                    last.append((name, args, text, err))
                if m["limit_hit"]:
                    break
        finally:
            tools.close()
        row = {"id": task["id"], "kind": task["kind"], "fixture": task["fixture"], "arm": arm, "agent": agent_kind,
               **({"rust_comments": RUST_COMMENTS} if arm == "rust" else {}),
               **m, "tokens": math.ceil(m["resp_bytes"] / 4), "req_tokens": math.ceil(m["req_bytes"] / 4),
               "wall_s": round(time.perf_counter() - t0, 3), "model_usage": agent.usage,
               "stop": getattr(agent, "stop", None)}
        if comprehend:
            want, got = set(task["truth"]), set(answer or [])
            tp = len(want & got)
            row.update(precision=round(tp / len(got), 3) if got else (1.0 if not want else 0.0),
                       recall=round(tp / len(want), 3) if want else 1.0, answered=answer is not None)
            row["success"] = row["precision"] == 1.0 and row["recall"] == 1.0
        else:
            row.update(judge_change(task, arm, root, before, work))
        return row
    finally:
        shutil.rmtree(work, ignore_errors=True)


# ---------- tasks, check, run ----------

def load_tasks(d):
    out = []
    for f in sorted(os.listdir(d)):
        if f.endswith(".json"):
            with open(os.path.join(d, f), encoding="utf-8") as h:
                out.append(json.load(h))
    return out


def check_twins(fixture):
    """Both programs of a fixture print every corpus case's expected bytes."""
    problems = []
    work = tempfile.mkdtemp(prefix="pair-twin-")
    try:
        exe = os.path.join(work, "bmb-exe")
        src = os.path.join(fixture_dir(fixture), "bmb")
        files = sorted(os.path.join(src, f) for f in os.listdir(src) if f.endswith(".bmb"))
        r = subprocess.run([bmb_bin(), "build", *files, "-o", exe], capture_output=True, text=True)
        if r.returncode != 0:
            return [f"{fixture}: bmb build failed: {r.stdout[-2000:]}"]
        cases = corpus(fixture)
        arms = [("bmb", exe)]
        # The Rust twin as written and without its comments (ADR-0106): the
        # stripped crate is the measured one and must be the same program.
        for cond in ("keep", "strip"):
            crate = os.path.join(work, f"crate-{cond}")
            shutil.copytree(os.path.join(fixture_dir(fixture), "rust"), crate, ignore=shutil.ignore_patterns("target"))
            if cond == "strip":
                strip_crate(crate)
            tgt = os.path.join(work, f"target-{cond}")
            code, out = cargo(crate, tgt, ["build"])
            if code != 0:
                return [f"{fixture}: cargo build ({cond} comments) failed: {out[-2000:]}"]
            arms.append((f"rust ({cond} comments)", os.path.join(tgt, "debug", fixture)))
        for arm, e in arms:
            n, fails = run_cases(e, cases)
            if fails:
                problems.append(f"{fixture}: {arm} differs from the corpus on {fails}")
        if not cases:
            problems.append(f"{fixture}: empty corpus")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return problems


WEAK_CHECKS = {
    "corpus": lambda r: r.get("built") and r["corpus_passed"] < r["corpus_total"],
    "scope": lambda r: bool(r.get("out_of_scope")),
    "build": lambda r: not r.get("built"),
    "answer": lambda r: not r["success"],
}


def cmd_check(a):
    tasks = load_tasks(a.tasks)
    problems = []
    for fx in sorted({t["fixture"] for t in tasks}):
        problems += check_twins(fx)
    for t in tasks:
        for arm in ("bmb", "rust"):
            r = run_task(t, arm, "oracle")
            if not r["success"]:
                problems.append(f"{t['id']} {arm}: oracle failed: {json.dumps(r, ensure_ascii=False)[:1500]}")
            for k, weak in enumerate(t.get("weak", [])):
                w = run_task(t, arm, f"weak{k}", weak[arm])
                why = weak["fails"]
                if w["success"] or not WEAK_CHECKS[why](w):
                    problems.append(f"{t['id']} {arm}: weak control {k} did not fail its {why} check: "
                                    f"{json.dumps(w, ensure_ascii=False)[:1500]}")
            print(f"{t['id']} {arm}: oracle {'ok' if r['success'] else 'FAILED'}", file=sys.stderr)
    for p in problems:
        print(p, file=sys.stderr)
    if problems:
        sys.exit(1)
    print(f"ok: {len(tasks)} tasks, both arms")


def cmd_run(a):
    tasks = [t for t in load_tasks(a.tasks) if not a.only or t["id"] in a.only]
    out = open(a.out, "a", encoding="utf-8") if a.out else sys.stdout
    for t in tasks:
        for arm in a.arms:
            r = run_task(t, arm, a.agent)
            out.write(json.dumps(r, ensure_ascii=False) + "\n")
            out.flush()


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    ck = sub.add_parser("check")
    ck.add_argument("--tasks", default=os.path.join(HERE, "tasks"))
    rn = sub.add_parser("run")
    rn.add_argument("--agent", default="oracle")
    rn.add_argument("--arms", nargs="+", default=["bmb", "rust"], choices=["bmb", "rust"])
    rn.add_argument("--tasks", default=os.path.join(HERE, "tasks"))
    rn.add_argument("--only", nargs="*")
    rn.add_argument("--out")
    a = ap.parse_args()
    {"check": cmd_check, "run": cmd_run}[a.cmd](a)


if __name__ == "__main__":
    main()
