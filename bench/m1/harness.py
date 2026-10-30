#!/usr/bin/env python3
"""M1 measurement harness (ADR-0021, backlog 1E-03, 1E-04).

  harness.py [--set DIR] manifest [--write]   check (or rewrite) the frozen-input hashes
  harness.py [--set DIR] run --agent AGENT [--tasks ID,..] [--out FILE] [--expect all-pass|weak-fail] [--jobs N]
  harness.py summarize RESULTS.jsonl..        metric table, one column per file

--set names the task-set directory (prompt.md, seed/, tasks/, MANIFEST); the
default is this directory (`m1`). Task sets are never mixed (ADR-0021 §1,
ADR-0025): `summarize` refuses files from different sets.

Agents:
  oracle            submits each task's reference solution through MCP (CI check
                    that every task is solvable and the harness works end to end)
  weak              submits `weak_reference` solutions; they must be rejected
  openai:MODEL      a real model, OpenAI-compatible Chat Completions
  anthropic:MODEL   a real model, Anthropic Messages API
                    (models and endpoints: ADR-0066, bench/m1/model_api.py)

Standard library only.
"""
import argparse
import atexit
import concurrent.futures
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
# The task set in use (backlog 2A-04). `main` changes it with --set.
SET = HERE
MAX_TOOL_CALLS = 40
READ_TOOLS = {"query", "map"}

sys.path.insert(0, HERE)
import bsb  # noqa: E402
import resultlog  # noqa: E402
from model_api import context_log  # noqa: E402


def run_commit():
    """HEAD, with `+dirty` when sources differ from it. Result files of earlier
    sets in the same run are not source changes."""
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(["git", "status", "--porcelain", "--", ".", ":!bench/*/results/*"], cwd=ROOT,
                           capture_output=True, text=True).stdout.strip()
    return commit + ("+dirty" if dirty else "")


def bmb_bin():
    return os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


# ---------- frozen inputs ----------

def set_name():
    return os.path.basename(os.path.normpath(SET))


def set_config():
    """`set.json` of the task set, when it has one (backlog 2D-24, T1-06):
    `mcp_flags` for `bmb mcp` (e.g. `--write-tool`) and `hide_tools`, tools the
    model is not shown (still callable, so scripted agents keep their calls).
    Frozen with the set."""
    path = os.path.join(SET, "set.json")
    if not os.path.exists(path):
        return {}
    with open(path) as f:
        return json.load(f)


def frozen_files():
    files = [os.path.join(SET, "prompt.md")]
    if os.path.exists(os.path.join(SET, "set.json")):
        files.append(os.path.join(SET, "set.json"))
    for sub in ("seed/src", "tasks"):
        d = os.path.join(SET, sub)
        files += sorted(os.path.join(d, f) for f in os.listdir(d))
    return files


def manifest_text():
    lines = []
    for f in frozen_files():
        digest = hashlib.sha256(open(f, "rb").read()).hexdigest()
        lines.append(f"{digest}  {os.path.relpath(f, SET)}")
    return "\n".join(lines) + "\n"


def manifest_digest():
    return hashlib.sha256(open(os.path.join(SET, "MANIFEST"), "rb").read()).hexdigest()


def check_manifest():
    path = os.path.join(SET, "MANIFEST")
    if not os.path.exists(path) or open(path).read() != manifest_text():
        sys.exit("frozen inputs differ from MANIFEST: refusing to run (ADR-0021 §1)")


# ---------- bmb ----------

def bmb(*args, input=None):
    r = subprocess.run([bmb_bin(), *args], capture_output=True, text=True, input=input)
    return r.returncode, r.stdout


# ---------- proof cache (backlog INF-08) ----------
#
# Every `bmb` call on a session's workspace goes through the workspace's proof
# cache (`.bmb-cache/`, the one `bmb mcp` keeps). Answers are the same bytes
# with or without it (compiler tests/incremental.rs, tests/cache.rs); only
# proved obligations are stored, keyed by the exact solver input, so a judge
# that hits the cache gets the verdict a fresh proof would. The agent writes
# the cache only by proving through MCP. Each session starts from a copy of a
# cache warmed once per process on the seed, so the seed is proved once.

_seed_cache = {}
_seed_lock = threading.Lock()


def cache_dir(workspace):
    return os.path.join(workspace, ".bmb-cache")


def cache_args(workspace):
    return ["--cache", cache_dir(workspace)]


def seed_cache():
    """A proof cache warmed on this task set's seed (built once per process)."""
    with _seed_lock:
        if SET not in _seed_cache:
            tmp = tempfile.mkdtemp(prefix="bmb-seed-cache-")
            atexit.register(shutil.rmtree, tmp, True)
            ws = os.path.join(tmp, "ws")
            shutil.copytree(os.path.join(SET, "seed"), ws)
            bmb("verify", *cache_args(ws), ws)
            _seed_cache[SET] = cache_dir(ws)
        return _seed_cache[SET]


def rows(text):
    out = []
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("{"):
            out.append(json.loads(line))
    return out


class Mcp:
    """A `bmb mcp` subprocess spoken to over JSON-RPC lines."""

    def __init__(self, root):
        cfg = set_config()
        self.p = subprocess.Popen([bmb_bin(), "mcp", *cfg.get("mcp_flags", []), root], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, text=True)
        self.next = 1
        self.request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                    "clientInfo": {"name": "bmb-harness", "version": "1"}})
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
        self.p.stdin.flush()
        hidden = set(cfg.get("hide_tools", []))
        self.tools = [x for x in self.request("tools/list", {})["result"]["tools"] if x["name"] not in hidden]

    def request(self, method, params):
        msg = {"jsonrpc": "2.0", "id": self.next, "method": method, "params": params}
        self.next += 1
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()
        return json.loads(self.p.stdout.readline())

    def call(self, name, args):
        r = self.request("tools/call", {"name": name, "arguments": args})["result"]
        return r["content"][0]["text"], r.get("isError", False)

    def close(self):
        self.p.stdin.close()
        self.p.wait()


ANSWER_TOOL = {
    "name": "answer",
    "description": "Submit the answer to a question: the full paths (unit::name) of the definitions asked for.",
    "inputSchema": {"type": "object", "properties": {"defs": {"type": "array", "items": {"type": "string"}}},
                    "required": ["defs"]},
}


# ---------- agents ----------

class ScriptAgent:
    """Plays a fixed list of tool calls."""

    def __init__(self, calls):
        self.calls = list(calls)
        self.usage = {"input_tokens": 0, "output_tokens": 0}

    def next_calls(self, _last_results):
        return [self.calls.pop(0)] if self.calls else []


def ops_to_calls(ops):
    calls = [("tx_begin", {})]
    for op in ops:
        if op["op"] == "unit":
            calls.append(("tx_unit", {"tx": "$tx", "path": op["path"], "summary": op["summary"]}))
        elif op["op"] == "put":
            calls.append(("tx_put", {"tx": "$tx", "unit": op["unit"], "code": op["code"]}))
        elif op["op"] == "remove":
            calls.append(("tx_remove", {"tx": "$tx", "def": op["def"]}))
        else:
            calls.append(("tx_remove_unit", {"tx": "$tx", "path": op["path"]}))
    calls += [("tx_commit", {"tx": "$tx"}), ("verify", {})]
    return calls


def make_agent(kind, task, workspace):
    if kind == "oracle":
        if task["kind"] == "write":
            return ScriptAgent(ops_to_calls(task["reference"]))
        return ScriptAgent([("answer", {"defs": sorted(truth(task, workspace))})])
    if kind == "weak":
        return ScriptAgent(ops_to_calls(task.get("weak_reference", [])))
    sys.path.insert(0, HERE)
    from model_api import make_model_agent  # noqa: E402
    if agent := make_model_agent(kind, open(os.path.join(SET, "prompt.md")).read(), task["prompt"]):
        return agent
    sys.exit(f"unknown agent {kind}")


# ---------- judging ----------

def truth_rows(t, query, workspace):
    found = set()
    _, out = bmb("query", *cache_args(workspace), *query, "--", workspace)
    for r in rows(out):
        if "list" in t:
            found |= set(r.get(t["list"], []))
        elif "kinds" in t and r.get("kind") in t["kinds"]:
            found.add(r[t["field"]])
        elif "effects" in t and r.get("effects") in t["effects"]:
            found.add(r[t["field"]])
        elif "reasons" in t and r.get("reason") in t["reasons"]:
            found.add(r[t["field"]])
    return found


def truth(task, workspace):
    """The answer set, computed from the maps of the session's final workspace.

    `query` (and `also`: one more query, or a list of them) are `bmb query`
    arguments; rows are kept by `list`, `kinds`, `effects` or `reasons`
    (`impact` rows). With `closure`, the query is repeated with each found
    definition in place of `query[1]` until nothing new appears (transitive
    callers or callees, m1b).
    """
    t = task["truth"]
    also = t.get("also", [])
    if also and isinstance(also[0], str):
        also = [also]
    found = set()
    for q in [t["query"]] + also:
        found |= truth_rows(t, q, workspace)
    if t.get("closure"):
        todo = sorted(found)
        while todo:
            d = todo.pop(0)
            for x in sorted(truth_rows(t, [t["query"][0], d, *t["query"][2:]], workspace)):
                if x not in found:
                    found.add(x)
                    todo.append(x)
    return found


def strip_result_name(sig):
    return re.sub(r"-> \((\w+): (.*)\)$", r"-> \2", sig.strip())


def parse_signature(sig):
    m = re.match(r"fn (\w+)\((.*)\) -> (.*)$", sig.strip())
    name, params, ret = m.group(1), m.group(2), m.group(3)
    m2 = re.match(r"\((\w+): (.*)\)$", ret)
    ret_ty = m2.group(2) if m2 else ret
    return name, params, ret_ty


def judge_write(task, workspace):
    """Whole-workspace proof, then signature and spec strength of every target:
    `def` and, for multi-unit tasks (m1b), each entry of `also_check`."""
    code, out = bmb("verify", *cache_args(workspace), workspace)
    verified = code == 0
    cost = proof_cost(out)
    sig_ok = accepted = True
    rejected = []
    for target in [task] + task.get("also_check", []):
        s, a, why = judge_target(target, workspace, verified)
        sig_ok, accepted = sig_ok and s, accepted and a
        if why:
            rejected.append({"def": target["def"], "reason": why})
    verdict = {"verified": verified, "signature_ok": sig_ok, "spec_accepted": accepted,
               "success": verified and sig_ok and accepted,
               "rlimit_total": cost[0], "rlimit_max": cost[1]}
    if rejected:
        # Why the spec check failed, for analysis only (the verdict is the same):
        # `refuted` when the solver found a counterexample to the implication,
        # `undecided` when it could not decide (backlog T1-14: an agent contract
        # equivalent to the hidden one was rejected because the implication
        # needed induction over a recursive spec fn).
        verdict["spec_rejections"] = rejected
    if "frozen" in task:
        # Given definitions the hidden contract refers to (BVB): the agent may
        # use them but not change them, or the spec check would read its version.
        intact = frozen_intact(task["frozen"], workspace)
        verdict["given_intact"] = intact
        verdict["success"] = verdict["success"] and intact
    return verdict


def def_hashes(workspace, cache=True):
    _, l0 = bmb("map", *(cache_args(workspace) if cache else []), "L0", workspace)
    return {r["path"]: r.get("hash") for r in rows(l0) if r.get("kind") == "def"}


def frozen_intact(paths, workspace):
    """Every path in `paths` has the same definition hash as in the seed."""
    seed, now = def_hashes(os.path.join(SET, "seed"), cache=False), def_hashes(workspace)
    return all(p in seed and now.get(p) == seed[p] for p in paths)


def proof_cost(verify_out, prefix=None):
    """(sum of `rlimit`, largest single-obligation `rlimit_max`) over the
    definitions of a `bmb verify` output, or of those whose path starts with
    `prefix` (backlog INF-11: the proof cost of the bench load)."""
    rs = [r for r in rows(verify_out) if "rlimit" in r and (prefix is None or r["def"].startswith(prefix))]
    return sum(r["rlimit"] for r in rs), max((r.get("rlimit_max", 0) for r in rs), default=0)


def spec_rejection(verify_out):
    """`undecided` if the solver could not decide the hidden check, else `refuted`."""
    return "undecided" if "solver could not decide" in verify_out else "refuted"


def judge_target(task, workspace, verified):
    """(signature ok, hidden reference contract proved, why it was rejected or
    None) for one target."""
    _, surf = bmb("query", *cache_args(workspace), "surface", task["def"], "--", workspace)
    found = [r for r in rows(surf) if r.get("level") == "L1"]
    sig_ok = bool(found) and strip_result_name(found[0]["signature"]) == strip_result_name(task["signature"])
    accepted, why = False, None
    if verified and sig_ok:
        _, l0 = bmb("map", *cache_args(workspace), "L0", workspace)
        specs = sorted(r["path"] for r in rows(l0) if r.get("def_kind") == "spec fn")
        a = task["accept"]
        _, params, ret_ty = parse_signature(task["signature"])
        # Every predicate of the hidden contract is unfolded: `ref_pre`,
        # `ref_post` and, for ported specs (BVB), their helpers.
        hidden_specs = re.findall(r"spec fn (\w+)", a["specs"])
        if not a["pre"]:
            hidden_specs = [n for n in hidden_specs if n != "ref_pre"]
        uses = ", ".join(sorted(set(hidden_specs) | {"ref_post"}, key=lambda n: (n != "ref_post", n != "ref_pre", n))
                         + specs)
        req = f"    requires {a['pre']},\n" if a["pre"] else ""
        ret = f"(r: {ret_ty})" if ret_ty != "()" else "()"
        # The hidden unit takes a name the agent's workspace does not use.
        units = {r["path"] for r in rows(l0) if r.get("kind") == "unit"}
        hidden = "bmb_accept"
        while hidden in units:
            hidden += "_"
        accept = (
            f'#[summary("hidden acceptance check")]\nmod {hidden} {{\n'
            + a["specs"] + "\n\n"
            + f"#[effects(pure)]\n#[uses({uses})]\nfn check({params}) -> {ret}\n{req}"
            + f"    ensures {a['post']},\n{{\n    {task['def']}({a['args']})\n}}\n}}\n"
        )
        with tempfile.NamedTemporaryFile("w", suffix=".bmb", delete=False) as f:
            f.write(accept)
        try:
            code, out = bmb("verify", *cache_args(workspace), workspace, f.name)
            accepted = code == 0
            why = None if accepted else spec_rejection(out)
        finally:
            os.unlink(f.name)
    return sig_ok, accepted, why


# ---------- BSB: specification mutant kill rate (backlog 2A-10) ----------

_bsb_sets = {}
_bsb_lock = threading.Lock()


def bsb_ready():
    """Whether this task set has hidden mutant sets (checked once per set
    against `mutants/MANIFEST`; a mismatch stops the run)."""
    with _bsb_lock:
        if SET not in _bsb_sets:
            _bsb_sets[SET] = bsb.check_manifest(SET)
        return _bsb_sets[SET]


def spec_kill(task, workspace, verdict):
    """Mutant kill rate of the workspace's contracts (`bsb.py`): `bsb_mutants`,
    `bsb_killed`, `bsb_kill_rate` and the surviving mutants. Without a
    verified workspace and the required signatures the contracts are not
    judged and kill nothing. Empty when the set has no mutants for the task."""
    data = bsb.load(SET, task["id"]) if bsb_ready() else None
    if data is None:
        return {}
    targets = [(t["def"], t["signature"], t["mutants"]) for t in data["targets"]]
    total = sum(len(m) for _, _, m in targets)
    if verdict["verified"] and verdict["signature_ok"]:
        _, l0 = bmb("map", *cache_args(workspace), "L0", workspace)
        specs = sorted(r["path"] for r in rows(l0) if r.get("def_kind") == "spec fn")
        units = {r["path"] for r in rows(l0) if r.get("kind") == "unit"}
        outs = []

        def verify(f):
            outs.append(bmb("verify", *cache_args(workspace), workspace, f)[1])
            return outs[-1]
        killed = bsb.kill_check(targets, specs, units, verify)
        hidden = bsb.hidden_unit_name(units)
        cost = proof_cost(outs[0], hidden + "::") if outs else (0, 0)
    else:
        killed = [[False] * len(m) for _, _, m in targets]
        cost = (0, 0)
    survivors = [f"{d}:{m['id']}" for (d, _, ms), ks in zip(targets, killed)
                 for m, k in zip(ms, ks) if not k]
    n = sum(sum(ks) for ks in killed)
    return {"bsb_mutants": total, "bsb_killed": n,
            "bsb_kill_rate": round(n / total, 3) if total else None, "bsb_survivors": survivors,
            "bsb_rlimit_total": cost[0], "bsb_rlimit_max": cost[1]}


def judge_navigate(task, workspace, answer):
    want = truth(task, workspace)
    got = set(answer or [])
    tp = len(want & got)
    # An empty answer is exact only when nothing is to be found.
    precision = tp / len(got) if got else (1.0 if not want else 0.0)
    recall = tp / len(want) if want else 1.0
    return {"precision": round(precision, 3), "recall": round(recall, 3),
            "success": precision == 1.0 and recall == 1.0, "truth": sorted(want), "answer": sorted(got)}


# ---------- sessions ----------

def run_task(task, agent_kind):
    tmp = tempfile.mkdtemp(prefix=f"bmb-{task['id']}-")
    ws = os.path.join(tmp, "ws")
    shutil.copytree(os.path.join(SET, "seed"), ws)
    if os.path.isdir(seed_cache()):
        shutil.copytree(seed_cache(), cache_dir(ws))
    mcp = Mcp(ws)
    agent = make_agent(agent_kind, task, ws)
    if hasattr(agent, "set_tools"):
        agent.set_tools(mcp.tools + ([ANSWER_TOOL] if task["kind"] == "navigate" else []))
    m = {"tool_calls": 0, "read_calls": 0, "read_bytes": 0, "commits": 0, "e02": 0, "limit_hit": False}
    # With BMB_CONTEXT_LOG=1 (model_api.context_log), every call's tool and
    # result size, in order (backlog 2D-25): with the agent's per-request
    # tokens they decompose each turn's context. Off by default.
    calls_log = []
    tx = None
    answer = None
    last = []
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
                args = {k: (tx if v == "$tx" else v) for k, v in args.items()}
                if name == "answer":
                    answer = args.get("defs", [])
                    text, err = '{"answered":true}', False
                else:
                    text, err = mcp.call(name, args)
                if name == "tx_begin" and not err:
                    tx = json.loads(text)["tx"]
                if name == "tx_commit":
                    m["commits"] += 1
                if name in READ_TOOLS:
                    m["read_calls"] += 1
                    m["read_bytes"] += len(text.encode())
                m["e02"] += len(re.findall(r'"code":"E02\d\d"', text))
                calls_log.append({"name": name, "bytes": len(text.encode()), "error": bool(err)})
                last.append((name, args, text, err))
            if m["limit_hit"]:
                break
    finally:
        mcp.close()
    # Before judging: the acceptance check proves an extra file, whose new
    # `spec fn`s change every query, so `status` here hits the session's cache.
    _, st = bmb("query", *cache_args(ws), "status", "--", ws)
    if task["kind"] == "write":
        verdict = judge_write(task, ws)
        verdict.update(spec_kill(task, ws, verdict))
    else:
        verdict = judge_navigate(task, ws, answer)
    summary = [r for r in rows(st) if r.get("summary")]
    status = [r for r in rows(st) if "status" in r]
    shutil.rmtree(tmp, ignore_errors=True)
    tokens = m["read_bytes"] // 4 + (1 if m["read_bytes"] % 4 else 0)
    return {
        "id": task["id"], "kind": task["kind"], "agent": agent_kind, **verdict, **m,
        "read_tokens_per_query": round(tokens / m["read_calls"], 1) if m["read_calls"] else 0,
        "e02_per_commit": round(m["e02"] / m["commits"], 3) if m["commits"] else 0,
        "unproved_obligations": sum(r.get("failed", 0) for r in status),
        "io_defs": summary[0]["io_defs"] if summary else 0,
        "model_usage": agent.usage,
        "agent_settings": getattr(agent, "settings", None),
        "stop": getattr(agent, "stop", None),
        **({"calls": calls_log} if context_log() else {}),
    }


# ---------- summary (backlog 1E-12) ----------

def summarize(paths):
    """Markdown tables of the ADR-0021 §4 metrics, one column per results file.

    Files from different task sets are refused (ADR-0025 §4); results written
    before backlog 2A-04 carry no `task_set` and are `m1`."""
    runs = []
    for p in paths:
        recs = [json.loads(l) for l in open(p) if l.strip()]
        head = next((r for r in recs if r.get("summary")), {})
        runs.append((os.path.basename(p), head, [r for r in recs if not r.get("summary")]))
    sets = sorted({h.get("task_set", "m1") for _, h, _ in runs})
    if len(sets) > 1:
        sys.exit(f"results from different task sets ({', '.join(sets)}) are not compared (ADR-0025)")

    def rejections(xs, reason):
        return sum(1 for r in xs if any(j["reason"] == reason for j in r.get("spec_rejections", [])))

    def ratio(xs):
        return f"{sum(1 for r in xs if r['success'])}/{len(xs)}" if xs else "—"

    def mean(xs, key):
        return f"{sum(r[key] for r in xs) / len(xs):.3f}" if xs else "—"

    def metrics(tasks):
        w = [r for r in tasks if r["kind"] == "write"]
        n = [r for r in tasks if r["kind"] == "navigate"]
        reads = sum(r["read_calls"] for r in tasks)
        rbytes = sum(r["read_bytes"] for r in tasks)
        commits = sum(r["commits"] for r in tasks)
        usage = [r.get("model_usage") or {} for r in tasks]
        return [
            ("작성 1회 성공률", ratio(w)),
            ("탐색 1회 성공률", ratio(n)),
            ("탐색 정밀도(평균)", mean(n, "precision")),
            ("탐색 재현율(평균)", mean(n, "recall")),
            ("질의당 토큰(ceil(bytes/4), 전체 평균)", f"{-(-rbytes // 4) / reads:.1f}" if reads else "—"),
            ("Rust 습관 오류율(E02xx / tx_commit)", f"{sum(r['e02'] for r in tasks) / commits:.3f}" if commits else "—"),
            ("미증명 의무(합)", str(sum(r["unproved_obligations"] for r in tasks))),
            ("`io` 정의(과제당 평균)", mean(tasks, "io_defs")),
            ("도구 호출(합)", str(sum(r["tool_calls"] for r in tasks))),
            ("한도 도달 과제", str(sum(1 for r in tasks if r["limit_hit"]))),
            ("모델 입력 토큰(합)", str(sum(u.get("input_tokens", 0) for u in usage))),
            ("모델 출력 토큰(합)", str(sum(u.get("output_tokens", 0) for u in usage))),
            # BSB (ADR-0034 G3): killed / hidden mutants, over the write tasks that have them.
            ("명세 변이체 판별률", bsb.rate_text(sum(r["bsb_killed"] for r in w if "bsb_mutants" in r),
                                         sum(r["bsb_mutants"] for r in w if "bsb_mutants" in r))),
            # Why the spec-strength check rejected (backlog T0-06): `undecided` means the
            # judge's solver could not decide, which measures the judge, not the contract.
            ("명세 강도 거부: 반박(반례)", str(rejections(w, "refuted"))),
            ("명세 강도 거부: 미결정(해법기)", str(rejections(w, "undecided"))),
        ]

    out = ["| 지표 | " + " | ".join(name for name, _, _ in runs) + " |",
           "| --- |" + " --- |" * len(runs)]
    out.append("| 과제 집합 | " + " | ".join(h.get("task_set", "m1") for _, h, _ in runs) + " |")
    out.append("| 에이전트 | " + " | ".join(str(h.get("agent", "—")) for _, h, _ in runs) + " |")
    out.append("| 컴파일러 커밋 | " + " | ".join(str(h.get("commit", "—"))[:12] for _, h, _ in runs) + " |")
    cols = [metrics(t) for _, _, t in runs]
    for i, (label, _) in enumerate(cols[0]):
        out.append(f"| {label} | " + " | ".join(c[i][1] for c in cols) + " |")

    ids = sorted({r["id"] for _, _, t in runs for r in t})
    out += ["", "| 과제 | " + " | ".join(name for name, _, _ in runs) + " |",
            "| --- |" + " --- |" * len(runs)]
    for tid in ids:
        cells = []
        for _, _, t in runs:
            r = next((r for r in t if r["id"] == tid), None)
            if r is None:
                cells.append("—")
            elif r["success"]:
                cells.append("성공")
            elif r["kind"] == "write":
                why = [k for k in ("verified", "signature_ok", "spec_accepted", "given_intact") if not r.get(k, True)]
                reasons = sorted({j["reason"] for j in r.get("spec_rejections", [])})
                cells.append("실패: " + ", ".join(why) + (f" ({', '.join(reasons)})" if reasons else ""))
            else:
                cells.append(f"실패: P {r['precision']}, R {r['recall']}")
        out.append(f"| {tid} | " + " | ".join(cells) + " |")
    return "\n".join(out) + "\n"


def main():
    global SET
    ap = argparse.ArgumentParser()
    ap.add_argument("--set", default=HERE, help="task-set directory (default: bench/m1)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    mf = sub.add_parser("manifest")
    mf.add_argument("--write", action="store_true")
    rn = sub.add_parser("run")
    rn.add_argument("--agent", required=True)
    rn.add_argument("--tasks")
    rn.add_argument("--out")
    rn.add_argument("--expect", choices=["all-pass", "weak-fail"])
    # Sessions are independent (own workspace copy, own `bmb mcp`), so CI runs
    # them in parallel; results keep task order.
    rn.add_argument("--jobs", type=int, default=1)
    sm = sub.add_parser("summarize")
    sm.add_argument("results", nargs="+")
    args = ap.parse_args()
    SET = os.path.abspath(args.set)

    if args.cmd == "summarize":
        sys.stdout.write(summarize(args.results))
        return

    if args.cmd == "manifest":
        if args.write:
            open(os.path.join(SET, "MANIFEST"), "w").write(manifest_text())
        else:
            check_manifest()
            print("MANIFEST ok")
        return

    check_manifest()
    bsb_ready()
    tdir = os.path.join(SET, "tasks")
    tasks = [json.load(open(os.path.join(tdir, f))) for f in sorted(os.listdir(tdir))]
    if args.tasks:
        wanted = set(args.tasks.split(","))
        tasks = [t for t in tasks if t["id"] in wanted]
    if args.agent == "weak":
        tasks = [t for t in tasks if "weak_reference" in t]
    # Results depend on the compiler (diagnostics, hints), so record it.
    commit = run_commit()
    log = resultlog.ResultLog(args.out, {"agent": args.agent, "task_set": set_name(), "manifest": manifest_digest(),
                                         "commit": commit, "tasks": [t["id"] for t in tasks]})

    def session(t):
        r = run_task(t, args.agent)
        log.add(r)
        return r

    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        list(pool.map(session, [t for t in tasks if not log.done(t["id"])]))
    results = [log.rows[t["id"]] for t in tasks]
    for r in results:
        print(json.dumps(r, ensure_ascii=False))
    write = [r for r in results if r["kind"] == "write"]
    nav = [r for r in results if r["kind"] == "navigate"]
    summary = {
        "summary": True, "agent": args.agent, "tasks": len(results),
        "task_set": set_name(), "manifest": manifest_digest(), "commit": commit,
        "write_success": f"{sum(r['success'] for r in write)}/{len(write)}",
        "navigate_success": f"{sum(r['success'] for r in nav)}/{len(nav)}",
        "navigate_precision": round(sum(r["precision"] for r in nav) / len(nav), 3) if nav else None,
        "navigate_recall": round(sum(r["recall"] for r in nav) / len(nav), 3) if nav else None,
    }
    if any("bsb_mutants" in r for r in write):
        summary["bsb_kill"] = bsb.rate_text(sum(r.get("bsb_killed", 0) for r in write),
                                            sum(r.get("bsb_mutants", 0) for r in write))
        with open(os.path.join(bsb.mutants_dir(SET), "MANIFEST"), "rb") as f:
            summary["bsb_manifest"] = hashlib.sha256(f.read()).hexdigest()
    log.finish(results, summary)
    print(json.dumps(summary, ensure_ascii=False))
    if args.expect == "all-pass" and not all(r["success"] for r in results):
        sys.exit("expected every task to pass")
    if args.expect == "weak-fail" and (not results or any(r["success"] for r in results)):
        sys.exit("expected every weak solution to be rejected")


if __name__ == "__main__":
    main()
