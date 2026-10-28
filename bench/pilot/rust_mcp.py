#!/usr/bin/env python3
"""MCP stdio server for the Rust arm of BRB in pilot runs.

  rust_mcp.py CRATE TARGET_DIR METRICS.json

Serves `bench/brb/rust_tools.py` (`list_files`, `read_file`, `write_file`,
`cargo_build`, `cargo_test`) with its own texts, sandbox and limits, and the
40-call limit of the harness (`bench/brb/harness.py` run_session). Metrics go
to METRICS.json after every call.
"""
import json
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(HERE), "brb"))
from rust_tools import RustTools  # noqa: E402

MAX_TOOL_CALLS = 40
RESULT_SIZE_META = "anthropic/maxResultSizeChars"  # see mcp_proxy.py


def main():
    crate, target, metrics_path = sys.argv[1:4]
    tools = RustTools(crate, target)
    m = {"tool_calls": 0, "tool_seconds": 0.0, "limit_hit": None, "tool_counts": {}, "calls": []}

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

    save()
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        msg = json.loads(line)
        method, msg_id = msg.get("method"), msg.get("id")
        if msg_id is None:
            continue
        if method == "initialize":
            send(msg_id, {"protocolVersion": msg.get("params", {}).get("protocolVersion", "2025-06-18"),
                          "capabilities": {"tools": {}}, "serverInfo": {"name": "brb-rust", "version": "1"}})
        elif method == "tools/list":
            send(msg_id, {"tools": [{**t, "_meta": {RESULT_SIZE_META: 2_000_000}} for t in tools.tools]})
        elif method == "tools/call":
            name = msg["params"]["name"]
            args = msg["params"].get("arguments") or {}
            if m["tool_calls"] >= MAX_TOOL_CALLS:
                m["limit_hit"] = "tool_calls"
                text, err = "not run: tool call limit reached", True
            else:
                m["tool_calls"] += 1
                m["tool_counts"][name] = m["tool_counts"].get(name, 0) + 1
                t0 = time.monotonic()
                text, err = tools.call(name, args)
                m["tool_seconds"] += time.monotonic() - t0
            m["calls"].append({"name": name, "error": bool(err), "bytes": len(text.encode())})
            save()
            send(msg_id, {"content": [{"type": "text", "text": text}], "isError": err})
        elif method == "ping":
            send(msg_id, {})
        else:
            send(msg_id, error={"code": -32601, "message": f"method not found: {method}"})


if __name__ == "__main__":
    main()
