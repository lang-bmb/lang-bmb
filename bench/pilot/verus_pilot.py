#!/usr/bin/env python3
"""Pilot runs of the Verus side of BVB through the Claude Code CLI (backlog T3-03).

  verus_pilot.py --model MODEL [--tasks ID,..] [--jobs N] --out FILE

The BMB side is `cli_pilot.py --set bench/bvb`. This is the same CLI session
(`cli_pilot.run_claude`: frozen system prompt, MCP tools only, no built-in
tools, an empty directory outside the repository, the API adapter's effort)
on the original Verus task: `bench/bvb/verus/prompt.md`, the task's Verus
prompt, the skeleton file, the harness's three tools (`verus_mcp.py`) and the
harness's judge (verified with --no-cheating, signature, spec strength, given
functions intact). Rows say `"pilot": true`; ADR-0058 §2 applies.
"""
import argparse
import concurrent.futures
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
import cli_pilot  # noqa: E402
from verus_mcp import vh  # noqa: E402  (bench/bvb/verus/harness.py)

verifier = vh.VerusVerifier()


def run_task(task, model, keep=None):
    tmp = tempfile.mkdtemp(prefix=f"bvb-verus-pilot-{task['id']}-")
    state, cwd = os.path.join(tmp, "state"), os.path.join(tmp, "cwd")
    os.makedirs(state)
    os.makedirs(cwd)
    with open(os.path.join(state, "main.rs"), "w") as f:
        f.write(vh.skeleton(task))
    prompt = open(os.path.join(vh.HERE, "prompt.md")).read()
    server = [sys.executable, "-I", os.path.join(HERE, "verus_mcp.py"), state]
    session = cli_pilot.run_claude(model, prompt, task["verus"]["prompt"], server, cwd)
    m = json.load(open(os.path.join(state, "metrics.json")))
    verdict = vh.judge(task, open(os.path.join(state, "main.rs")).read(), verifier)
    if keep:
        os.makedirs(os.path.join(keep, task["id"]), exist_ok=True)
        shutil.copy(os.path.join(state, "main.rs"), os.path.join(keep, task["id"], "main.rs"))
    shutil.rmtree(tmp, ignore_errors=True)
    return {"id": task["id"], "kind": "write", "agent": f"claude-cli:{model}", "pilot": True, **verdict,
            **{k: m.get(k) for k in ("tool_calls", "writes", "verify_calls", "limit_hit", "calls")}, **session}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True)
    ap.add_argument("--tasks")
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--out", required=True)
    ap.add_argument("--keep", help="copy each session's final file to KEEP/ID/main.rs")
    args = ap.parse_args()
    if not verifier.available():
        sys.exit("pinned Verus is not installed: bench/bvb/verus/install.sh")
    tasks = vh.load_tasks(args.tasks)
    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        for r in pool.map(lambda t: run_task(t, args.model, args.keep), tasks):
            results.append(r)
            print(json.dumps({k: r.get(k) for k in ("id", "success", "tool_calls", "cost_usd", "wall_s")}),
                  flush=True)
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=vh.ROOT, capture_output=True,
                            text=True).stdout.strip()
    cli_version = subprocess.run(["claude", "--version"], capture_output=True, text=True).stdout.strip()
    summary = {
        "summary": True, "pilot": True, "agent": f"claude-cli:{args.model}", "cli": cli_version,
        "effort": cli_pilot.anthropic_agent.EFFORT, "tasks": len(results), "task_set": "bvb-verus",
        "manifest": vh.bmb_harness.manifest_digest(), "verus_version": verifier.version, "commit": commit,
        "write_success": f"{sum(r['success'] for r in results)}/{len(results)}",
        "cost_usd": round(sum(r.get("cost_usd") or 0 for r in results), 4),
    }
    print(json.dumps(summary, ensure_ascii=False))
    with open(args.out, "w") as f:
        for r in results + [summary]:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
