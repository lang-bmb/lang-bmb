#!/usr/bin/env python3
"""MCP stdio server for the Verus side of BVB in pilot runs (backlog T3-03).

  verus_mcp.py STATE_DIR

Serves the three tools of `bench/bvb/verus/harness.py` (`read_file`,
`write_file`, `verify`) with the same texts and the same 40-call limit, so the
CLI agent sees what the harness's own loop gives an API agent. The file lives
in STATE_DIR/main.rs (the runner writes the skeleton and judges the final
text); metrics go to STATE_DIR/metrics.json after every call.
"""
import importlib.util
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "verus_harness", os.path.join(os.path.dirname(HERE), "bvb", "verus", "harness.py"))
vh = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(vh)

code_path = metrics_path = verifier = None
m = {"tool_calls": 0, "writes": 0, "verify_calls": 0, "limit_hit": False, "calls": []}


def save():
    with open(metrics_path + ".tmp", "w") as f:
        json.dump(m, f)
    os.replace(metrics_path + ".tmp", metrics_path)


def send(msg_id, result=None, error=None):
    out = {"jsonrpc": "2.0", "id": msg_id}
    if error is not None:
        out["error"] = error
    else:
        out["result"] = result
    sys.stdout.write(json.dumps(out) + "\n")
    sys.stdout.flush()


def call(name, args):
    if m["tool_calls"] >= vh.MAX_TOOL_CALLS:
        m["limit_hit"] = True
        return "not run: tool call limit reached", True
    m["tool_calls"] += 1
    code = open(code_path).read()
    if name == "read_file":
        return code, False
    if name == "write_file" and isinstance(args.get("code"), str):
        with open(code_path, "w") as f:
            f.write(args["code"])
        m["writes"] += 1
        return '{"written":true}', False
    if name == "verify":
        m["verify_calls"] += 1
        ok, out = verifier.verify(code)
        return ("verified\n" if ok else "verification failed\n") + out, not ok
    return f"unknown tool or bad arguments: {name}", True



def main():
    global code_path, metrics_path, verifier
    state = sys.argv[1]
    code_path = os.path.join(state, "main.rs")
    metrics_path = os.path.join(state, "metrics.json")
    verifier = vh.VerusVerifier()
    save()
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        msg = json.loads(line)
        method, msg_id = msg.get("method"), msg.get("id")
        if msg_id is None:
            continue  # notifications
        if method == "initialize":
            send(msg_id, {"protocolVersion": msg.get("params", {}).get("protocolVersion", "2025-06-18"),
                          "capabilities": {"tools": {}}, "serverInfo": {"name": "bvb-verus", "version": "1"}})
        elif method == "tools/list":
            send(msg_id, {"tools": [{**tool, "_meta": {"anthropic/maxResultSizeChars": 2_000_000}} for tool in vh.TOOLS]})
        elif method == "tools/call":
            name = msg["params"]["name"]
            text, err = call(name, msg["params"].get("arguments") or {})
            m["calls"].append({"name": name, "error": bool(err), "bytes": len(text.encode())})
            save()
            send(msg_id, {"content": [{"type": "text", "text": text}], "isError": err})
        elif method == "ping":
            send(msg_id, {})
        else:
            send(msg_id, error={"code": -32601, "message": f"method not found: {method}"})


if __name__ == "__main__":
    main()
