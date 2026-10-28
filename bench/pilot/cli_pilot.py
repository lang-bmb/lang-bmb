#!/usr/bin/env python3
"""Pilot runs through the Claude Code CLI (ADR-0058 T0, human decision 2026-10-28).

  cli_pilot.py [--set DIR] --model MODEL [--tasks ID,..] [--jobs N] --out FILE

A pilot is NOT a measurement under ADR-0021: the agent loop, its prompt
caching, thinking and retries belong to the CLI, not to the harness. Each row
says `"pilot": true` and the summary refuses to be mixed with harness results.
Results guide direction only; no document claim rests on them (C3).

What is kept from the frozen protocol: the task set and its MANIFEST, the
system prompt (`prompt.md`, passed as the whole system prompt), the tools
(`bmb mcp` only, through `mcp_proxy.py`, no built-in tools, no files, no
shell), the 40-call limit, one session per task with no retry, the effort the
API adapter uses, and the harness's own judging (`judge_write`, `spec_kill`,
`judge_navigate`). The CLI runs in an empty directory outside the repository.
"""
import argparse
import concurrent.futures
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(HERE), "m1"))
import anthropic_agent  # noqa: E402
import harness  # noqa: E402

TIMEOUT_S = 3600


def run_task(task, model):
    tmp = tempfile.mkdtemp(prefix=f"bmb-pilot-{task['id']}-")
    ws = os.path.join(tmp, "ws")
    cwd = os.path.join(tmp, "cwd")
    os.makedirs(cwd)
    shutil.copytree(os.path.join(harness.SET, "seed"), ws)
    if os.path.isdir(harness.seed_cache()):
        shutil.copytree(harness.seed_cache(), harness.cache_dir(ws))
    metrics = os.path.join(tmp, "metrics.json")
    cfg = os.path.join(tmp, "mcp.json")
    json.dump({"mcpServers": {"bmb": {"command": sys.executable, "args": [
        "-I", os.path.join(HERE, "mcp_proxy.py"), harness.bmb_bin(), ws, metrics, task["kind"]]}}},
        open(cfg, "w"))
    prompt = open(os.path.join(harness.SET, "prompt.md")).read()
    cmd = ["claude", "-p", task["prompt"], "--model", model, "--effort", anthropic_agent.EFFORT,
           "--system-prompt", prompt, "--tools", "", "--mcp-config", cfg, "--strict-mcp-config",
           "--allowedTools", "mcp__bmb", "--output-format", "json", "--no-session-persistence"]
    t0 = time.time()
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=TIMEOUT_S,
                           stdin=subprocess.DEVNULL)
        out, code = p.stdout, p.returncode
    except subprocess.TimeoutExpired:
        out, code = "", "timeout"
    wall = round(time.time() - t0, 1)
    try:
        cli = json.loads(out)
    except ValueError:
        cli = {"raw": out[-2000:]}
    m = json.load(open(metrics)) if os.path.exists(metrics) else {}
    if task["kind"] == "write":
        verdict = harness.judge_write(task, ws)
        verdict.update(harness.spec_kill(task, ws, verdict))
    else:
        verdict = harness.judge_navigate(task, ws, m.get("answer"))
    shutil.rmtree(tmp, ignore_errors=True)
    reads = m.get("read_bytes", 0)
    tokens = reads // 4 + (1 if reads % 4 else 0)
    return {
        "id": task["id"], "kind": task["kind"], "agent": f"claude-cli:{model}", "pilot": True,
        **verdict,
        **{k: m.get(k) for k in ("tool_calls", "read_calls", "read_bytes", "commits", "e02", "limit_hit")},
        "read_tokens_per_query": round(tokens / m["read_calls"], 1) if m.get("read_calls") else 0,
        "e02_per_commit": round(m["e02"] / m["commits"], 3) if m.get("commits") else 0,
        "calls": m.get("calls"),
        "cli_exit": code, "wall_s": wall,
        "model_usage": cli.get("usage"), "cost_usd": cli.get("total_cost_usd"),
        "num_turns": cli.get("num_turns"), "stop": cli.get("stop_reason") or cli.get("subtype"),
        "cli_models": sorted((cli.get("modelUsage") or {}).keys()),
        "cli_raw": cli.get("raw"),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--set", default=os.path.join(os.path.dirname(HERE), "m1"))
    ap.add_argument("--model", required=True)
    ap.add_argument("--tasks")
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    harness.SET = os.path.abspath(args.set)
    harness.check_manifest()
    harness.bsb_ready()
    tdir = os.path.join(harness.SET, "tasks")
    tasks = [json.load(open(os.path.join(tdir, f))) for f in sorted(os.listdir(tdir))]
    if args.tasks:
        wanted = set(args.tasks.split(","))
        tasks = [t for t in tasks if t["id"] in wanted]
    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        for r in pool.map(lambda t: run_task(t, args.model), tasks):
            results.append(r)
            print(json.dumps({k: r.get(k) for k in ("id", "success", "tool_calls", "cost_usd", "wall_s")}),
                  flush=True)
    write = [r for r in results if r["kind"] == "write"]
    nav = [r for r in results if r["kind"] == "navigate"]
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=harness.ROOT, capture_output=True,
                            text=True).stdout.strip()
    cli_version = subprocess.run(["claude", "--version"], capture_output=True, text=True).stdout.strip()
    summary = {
        "summary": True, "pilot": True, "agent": f"claude-cli:{args.model}", "cli": cli_version,
        "effort": anthropic_agent.EFFORT, "tasks": len(results), "task_set": harness.set_name(),
        "manifest": harness.manifest_digest(), "commit": commit,
        "write_success": f"{sum(r['success'] for r in write)}/{len(write)}",
        "navigate_success": f"{sum(r['success'] for r in nav)}/{len(nav)}",
        "cost_usd": round(sum(r.get("cost_usd") or 0 for r in results), 4),
    }
    print(json.dumps(summary, ensure_ascii=False))
    with open(args.out, "w") as f:
        for r in results + [summary]:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
