#!/usr/bin/env python3
"""M1 measurement harness (ADR-0021, backlog 1E-03, 1E-04).

  harness.py manifest [--write]      check (or rewrite) the frozen-input hashes
  harness.py run --agent AGENT [--tasks ID,..] [--out FILE] [--expect all-pass|weak-fail]

Agents:
  oracle            submits each task's reference solution through MCP (CI check
                    that every task is solvable and the harness works end to end)
  weak              submits `weak_reference` solutions; they must be rejected
  anthropic:MODEL   a real model through the Anthropic Messages API (paid; needs
                    ANTHROPIC_API_KEY and human approval, AUTONOMY.md §4)

Standard library only.
"""
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
MAX_TOOL_CALLS = 40
READ_TOOLS = {"query", "map"}


def bmb_bin():
    return os.environ.get("BMB", os.path.join(ROOT, "target", "debug", "bmb"))


# ---------- frozen inputs ----------

def frozen_files():
    files = [os.path.join(HERE, "prompt.md")]
    for sub in ("seed/src", "tasks"):
        d = os.path.join(HERE, sub)
        files += sorted(os.path.join(d, f) for f in os.listdir(d))
    return files


def manifest_text():
    lines = []
    for f in frozen_files():
        digest = hashlib.sha256(open(f, "rb").read()).hexdigest()
        lines.append(f"{digest}  {os.path.relpath(f, HERE)}")
    return "\n".join(lines) + "\n"


def check_manifest():
    path = os.path.join(HERE, "MANIFEST")
    if not os.path.exists(path) or open(path).read() != manifest_text():
        sys.exit("frozen inputs differ from MANIFEST: refusing to run (ADR-0021 §1)")


# ---------- bmb ----------

def bmb(*args, input=None):
    r = subprocess.run([bmb_bin(), *args], capture_output=True, text=True, input=input)
    return r.returncode, r.stdout


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
        self.p = subprocess.Popen([bmb_bin(), "mcp", root], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, text=True)
        self.next = 1
        self.request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                    "clientInfo": {"name": "bmb-harness", "version": "1"}})
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
        self.p.stdin.flush()
        self.tools = self.request("tools/list", {})["result"]["tools"]

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
    if kind.startswith("anthropic:"):
        sys.path.insert(0, HERE)
        from anthropic_agent import AnthropicAgent  # noqa: E402
        prompt = open(os.path.join(HERE, "prompt.md")).read()
        return AnthropicAgent(kind.split(":", 1)[1], prompt, task["prompt"])
    sys.exit(f"unknown agent {kind}")


# ---------- judging ----------

def truth(task, workspace):
    t = task["truth"]
    found = set()
    for q in [t["query"]] + ([t["also"]] if "also" in t else []):
        _, out = bmb("query", *q, "--", workspace)
        for r in rows(out):
            if "list" in t:
                found |= set(r.get(t["list"], []))
            elif "kinds" in t and r.get("kind") in t["kinds"]:
                found.add(r[t["field"]])
            elif "effects" in t and r.get("effects") in t["effects"]:
                found.add(r[t["field"]])
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
    code, out = bmb("verify", workspace)
    verified = code == 0
    _, surf = bmb("query", "surface", task["def"], "--", workspace)
    found = [r for r in rows(surf) if r.get("level") == "L1"]
    sig_ok = bool(found) and strip_result_name(found[0]["signature"]) == strip_result_name(task["signature"])
    accepted = False
    if verified and sig_ok:
        _, l0 = bmb("map", "L0", workspace)
        specs = sorted(r["path"] for r in rows(l0) if r.get("def_kind") == "spec fn")
        a = task["accept"]
        _, params, ret_ty = parse_signature(task["signature"])
        uses = ", ".join(["ref_post"] + (["ref_pre"] if a["pre"] else []) + specs)
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
            code, _ = bmb("verify", workspace, f.name)
            accepted = code == 0
        finally:
            os.unlink(f.name)
    return {"verified": verified, "signature_ok": sig_ok, "spec_accepted": accepted,
            "success": verified and sig_ok and accepted}


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
    shutil.copytree(os.path.join(HERE, "seed"), ws)
    mcp = Mcp(ws)
    agent = make_agent(agent_kind, task, ws)
    if hasattr(agent, "set_tools"):
        agent.set_tools(mcp.tools + ([ANSWER_TOOL] if task["kind"] == "navigate" else []))
    m = {"tool_calls": 0, "read_calls": 0, "read_bytes": 0, "commits": 0, "e02": 0, "limit_hit": False}
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
                last.append((name, args, text, err))
            if m["limit_hit"]:
                break
    finally:
        mcp.close()
    if task["kind"] == "write":
        verdict = judge_write(task, ws)
    else:
        verdict = judge_navigate(task, ws, answer)
    _, st = bmb("query", "status", "--", ws)
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
    }


# ---------- summary (backlog 1E-12) ----------

def summarize(paths):
    """Markdown tables of the ADR-0021 §4 metrics, one column per results file."""
    runs = []
    for p in paths:
        recs = [json.loads(l) for l in open(p) if l.strip()]
        head = next((r for r in recs if r.get("summary")), {})
        runs.append((os.path.basename(p), head, [r for r in recs if not r.get("summary")]))

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
        ]

    out = ["| 지표 | " + " | ".join(name for name, _, _ in runs) + " |",
           "| --- |" + " --- |" * len(runs)]
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
                why = [k for k in ("verified", "signature_ok", "spec_accepted") if not r[k]]
                cells.append("실패: " + ", ".join(why))
            else:
                cells.append(f"실패: P {r['precision']}, R {r['recall']}")
        out.append(f"| {tid} | " + " | ".join(cells) + " |")
    return "\n".join(out) + "\n"


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    mf = sub.add_parser("manifest")
    mf.add_argument("--write", action="store_true")
    rn = sub.add_parser("run")
    rn.add_argument("--agent", required=True)
    rn.add_argument("--tasks")
    rn.add_argument("--out")
    rn.add_argument("--expect", choices=["all-pass", "weak-fail"])
    sm = sub.add_parser("summarize")
    sm.add_argument("results", nargs="+")
    args = ap.parse_args()

    if args.cmd == "summarize":
        sys.stdout.write(summarize(args.results))
        return

    if args.cmd == "manifest":
        if args.write:
            open(os.path.join(HERE, "MANIFEST"), "w").write(manifest_text())
        else:
            check_manifest()
            print("MANIFEST ok")
        return

    check_manifest()
    tdir = os.path.join(HERE, "tasks")
    tasks = [json.load(open(os.path.join(tdir, f))) for f in sorted(os.listdir(tdir))]
    if args.tasks:
        wanted = set(args.tasks.split(","))
        tasks = [t for t in tasks if t["id"] in wanted]
    if args.agent == "weak":
        tasks = [t for t in tasks if "weak_reference" in t]
    results = [run_task(t, args.agent) for t in tasks]
    for r in results:
        print(json.dumps(r, ensure_ascii=False))
    write = [r for r in results if r["kind"] == "write"]
    nav = [r for r in results if r["kind"] == "navigate"]
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip()
    dirty = bool(subprocess.run(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True).stdout.strip())
    summary = {
        "summary": True, "agent": args.agent, "tasks": len(results),
        # Results depend on the compiler (diagnostics, hints), so record it.
        "commit": commit + ("+dirty" if dirty else ""),
        "write_success": f"{sum(r['success'] for r in write)}/{len(write)}",
        "navigate_success": f"{sum(r['success'] for r in nav)}/{len(nav)}",
        "navigate_precision": round(sum(r["precision"] for r in nav) / len(nav), 3) if nav else None,
        "navigate_recall": round(sum(r["recall"] for r in nav) / len(nav), 3) if nav else None,
    }
    print(json.dumps(summary, ensure_ascii=False))
    if args.out:
        with open(args.out, "w") as f:
            for r in results + [summary]:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
    if args.expect == "all-pass" and not all(r["success"] for r in results):
        sys.exit("expected every task to pass")
    if args.expect == "weak-fail" and (not results or any(r["success"] for r in results)):
        sys.exit("expected every weak solution to be rejected")


if __name__ == "__main__":
    main()
