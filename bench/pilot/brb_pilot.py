#!/usr/bin/env python3
"""Pilot runs of BRB, BMB+proof against Rust+tests, through the Claude Code CLI
(H-ORACLE, G5, G8; backlog T0-07).

  brb_pilot.py --arm bmb|rust --model MODEL [--tasks ID,..] [--jobs N] [--proptest on|off] --out FILE

The frozen harness (`bench/brb/harness.py`) is used as a library: the same
workspace or crate, the same prompts (`prompt-ARM.md` as the whole system
prompt, `judge.task_prompt`), the same tools (BMB: `bmb mcp` through
`mcp_proxy.py`; Rust: `rust_tools.py` through `rust_mcp.py`), the 40-call
limit, the arm's checker once more at the end (`final_check_ok`), the hidden
judge, the delivered sources and the test-code measure. Only the session loop
is the CLI's (`cli_pilot.run_claude`); the harness's per-session output-token
budget (64,000) is not enforced. Rows say `"pilot": true` (ADR-0058 §2), and
`total_tokens` counts cache reads and writes too, since the CLI reports them
apart. `bench/brb/harness.py summarize` reads the result files.
"""
import argparse
import concurrent.futures
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
import cli_pilot  # noqa: E402

_spec = importlib.util.spec_from_file_location("brb_harness", os.path.join(os.path.dirname(HERE), "brb", "harness.py"))
brb = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(brb)


def run_task(task, arm, model, target, proptest):
    tmp = tempfile.mkdtemp(prefix=f"brb-pilot-{arm}-{task.ID}-")
    try:
        cwd = os.path.join(tmp, "cwd")
        os.makedirs(cwd)
        metrics = os.path.join(tmp, "metrics.json")
        if arm == "bmb":
            ws = os.path.join(tmp, "ws")
            os.makedirs(os.path.join(ws, "src"))
            server = [sys.executable, "-I", os.path.join(HERE, "mcp_proxy.py"), brb.bmb_bin(), ws, metrics, "write"]
            check_tool = "verify"
        else:
            ws = os.path.join(tmp, "solution")
            brb.make_crate(ws, proptest)
            server = [sys.executable, "-I", os.path.join(HERE, "rust_mcp.py"), ws, target, metrics]
            check_tool = "cargo_test"
        prompt = brb.read_bytes(os.path.join(brb.HERE, f"prompt-{arm}.md")).decode()
        session = cli_pilot.run_claude(model, prompt, brb.judge.task_prompt(task, arm), server, cwd)
        m = json.load(open(metrics)) if os.path.exists(metrics) else {}
        if arm == "bmb":
            checker = brb.m1.Mcp(ws)
        else:
            checker = brb.RustTools(ws, target)
        try:
            text, err = checker.call(check_tool, {})
        finally:
            if hasattr(checker, "close"):
                checker.close()
        final_check_ok = brb.check_ok(arm, text, err)
        final_sources = brb.delivered_sources(ws, arm)
        test_code = brb.bmb_test_surface(ws) if arm == "bmb" else {"test_lines": brb.rust_test_lines(final_sources)}
        t1 = time.monotonic()
        jdir = os.path.join(tmp, "judge")
        os.makedirs(jdir)
        if arm == "bmb":
            verdict = brb.judge.judge_bmb(task, ws, brb.bmb_bin(), jdir, brb.m1.cache_dir(ws))
        else:
            verdict = brb.judge.judge_rust(task, ws, jdir)
        judge_s = time.monotonic() - t1
        success = verdict["delivered"] and verdict["failed"] == 0 and verdict.get("domain_ok", True) is not False
        usage = session.get("model_usage") or {}
        total = sum(usage.get(k, 0) or 0 for k in ("input_tokens", "cache_creation_input_tokens",
                                                   "cache_read_input_tokens", "output_tokens"))
        calls = m.get("calls") or []
        return {
            "id": task.ID, "arm": arm, "agent": f"claude-cli:{model}", "pilot": True,
            "success": success, "defect": not success, **verdict,
            "tool_calls": m.get("tool_calls", 0), "checks": sum(1 for c in calls if c["name"] == check_tool),
            "last_check_ok": None, "limit_hit": m.get("limit_hit") or (None if m.get("tool_calls", 0) < 40 else "tool_calls"),
            "tool_counts": m.get("tool_counts") or {c["name"]: sum(1 for d in calls if d["name"] == c["name"]) for c in calls},
            "final_check_ok": final_check_ok, "final_sources": final_sources, "test_code": test_code,
            "session_seconds": session["wall_s"], "tool_seconds": round(m.get("tool_seconds") or 0.0, 2),
            "judge_seconds": round(judge_s, 2), "total_tokens": total, **session,
        }
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", choices=["bmb", "rust"], required=True)
    ap.add_argument("--model", required=True)
    ap.add_argument("--tasks")
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--proptest", choices=["on", "off"], default="on")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    brb.check_manifest()
    ts = brb.tasks()
    if args.tasks:
        wanted = set(args.tasks.split(","))
        ts = [t for t in ts if t.ID in wanted]
    proptest = args.proptest == "on"
    target = tempfile.mkdtemp(prefix="brb-pilot-target-")
    results = []
    try:
        if args.arm == "rust":
            brb.warm_rust(target, proptest)
        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
            for r in pool.map(lambda t: run_task(t, args.arm, args.model, target, proptest), ts):
                results.append(r)
                print(json.dumps({k: r.get(k) for k in ("id", "arm", "success", "final_check_ok", "tool_calls",
                                                        "cost_usd", "wall_s")}), flush=True)
    finally:
        shutil.rmtree(target, ignore_errors=True)
    n = len(results)
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=brb.ROOT, capture_output=True, text=True).stdout.strip()
    cli_version = subprocess.run(["claude", "--version"], capture_output=True, text=True).stdout.strip()
    summary = {
        "summary": True, "pilot": True, "task_set": "brb", "arm": args.arm, "agent": f"claude-cli:{args.model}",
        "cli": cli_version, "tasks": n, "proptest": proptest if args.arm == "rust" else None,
        "manifest": brb.manifest_digest(), "commit": commit,
        "limits": {"tool_calls": brb.MAX_TOOL_CALLS, "session_output_tokens": None},
        "success": f"{sum(r['success'] for r in results)}/{n}",
        "defect_rate": round(sum(r["defect"] for r in results) / n, 3) if n else None,
        "total_tokens": sum(r["total_tokens"] for r in results),
        "cost_usd": round(sum(r.get("cost_usd") or 0 for r in results), 4),
    }
    print(json.dumps(summary, ensure_ascii=False))
    with open(args.out, "w") as f:
        for r in results + [summary]:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
