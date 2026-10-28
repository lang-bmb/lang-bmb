#!/usr/bin/env python3
"""MCP stdio proxy between a CLI agent and `bmb mcp` (pilot runs, ADR-0058 T0).

  mcp_proxy.py BMB WORKSPACE METRICS.json write|navigate

Forwards every JSON-RPC line to `bmb mcp WORKSPACE` and keeps the measurements
the harness keeps in its own loop (`bench/m1/harness.py` run_task): tool calls,
read calls and bytes, commits, E02xx diagnostics, the tool-call limit. For
navigation tasks it adds the harness's `answer` tool and records the answer.
The metrics file is rewritten after every call, so it survives a killed agent.
"""
import json
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "m1"))
import harness  # noqa: E402

# The API adapter hands the model every tool result whole. The CLI moves a
# larger result to a file and shows a 2KB preview, and a pilot agent has no tool
# to read files: the first pilot (2026-10-28) lost `map L1` of the m1b seed
# (59,442 bytes) that way. A tool may raise the threshold through `_meta`.
RESULT_SIZE_META = "anthropic/maxResultSizeChars"
RESULT_SIZE_CHARS = 2_000_000

bmb, ws, metrics_path, kind = sys.argv[1:5]
child = subprocess.Popen([bmb, "mcp", ws], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
m = {"tool_calls": 0, "read_calls": 0, "read_bytes": 0, "commits": 0, "e02": 0, "limit_hit": False,
     "answer": None, "calls": []}


def save():
    with open(metrics_path + ".tmp", "w") as f:
        json.dump(m, f)
    os.replace(metrics_path + ".tmp", metrics_path)


def forward(msg):
    child.stdin.write(json.dumps(msg) + "\n")
    child.stdin.flush()
    if "id" not in msg:
        return None
    return json.loads(child.stdout.readline())


def reply(msg_id, result=None, error=None):
    out = {"jsonrpc": "2.0", "id": msg_id}
    if error is not None:
        out["error"] = error
    else:
        out["result"] = result
    sys.stdout.write(json.dumps(out) + "\n")
    sys.stdout.flush()


def tool_text(text, err):
    return {"content": [{"type": "text", "text": text}], "isError": err}


save()
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    method = msg.get("method")
    if method == "tools/list":
        r = forward(msg)
        if kind == "navigate":
            r["result"]["tools"].append(harness.ANSWER_TOOL)
        for tool in r["result"]["tools"]:
            tool.setdefault("_meta", {})[RESULT_SIZE_META] = RESULT_SIZE_CHARS
        sys.stdout.write(json.dumps(r) + "\n")
        sys.stdout.flush()
        continue
    if method == "tools/call":
        name = msg["params"]["name"]
        args = msg["params"].get("arguments", {})
        if m["tool_calls"] >= harness.MAX_TOOL_CALLS:
            m["limit_hit"] = True
            save()
            reply(msg["id"], tool_text("not run: tool call limit reached", True))
            continue
        m["tool_calls"] += 1
        if name == "answer" and kind == "navigate":
            m["answer"] = args.get("defs", [])
            text, err = '{"answered":true}', False
            reply(msg["id"], tool_text(text, err))
        else:
            r = forward(msg)
            res = r.get("result") or {}
            text = (res.get("content") or [{"text": ""}])[0].get("text", "")
            err = res.get("isError", False) or "error" in r
            sys.stdout.write(json.dumps(r) + "\n")
            sys.stdout.flush()
        if name == "tx_commit":
            m["commits"] += 1
        if name in harness.READ_TOOLS:
            m["read_calls"] += 1
            m["read_bytes"] += len(text.encode())
        m["e02"] += len(re.findall(r'"code":"E02\d\d"', text))
        m["calls"].append({"name": name, "args": {k: (v[:2000] if isinstance(v, str) else v) for k, v in args.items()},
                           "error": bool(err), "bytes": len(text.encode())})
        save()
        continue
    r = forward(msg)
    if r is not None:
        sys.stdout.write(json.dumps(r) + "\n")
        sys.stdout.flush()
child.stdin.close()
child.wait()
