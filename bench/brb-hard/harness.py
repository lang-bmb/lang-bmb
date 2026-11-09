#!/usr/bin/env python3
"""BRB-HARD harness: tasks where tests tend to miss defects (H-ORACLE,
backlog T0-08, ADR-0103 decision 1.3 and 4).

  harness.py manifest [--write]          check (or rewrite) the frozen-input hashes
  harness.py run --arm bmb|rust --agent AGENT [--tasks ID,..] [--jobs N]
                 [--out FILE] [--sessions DIR] [--proptest on|off]
                 [--expect all-pass|weak-fail]
  harness.py adequacy [--jobs N]          every Rust mutant is caught by the hidden cases
  harness.py summarize RESULTS.jsonl..   BRB's metric table, G5 and H-ORACLE lines

A BRB set (`bench/brb`, frozen, loaded here as a library): the same judge,
prompts, tools, limits, Rust sandbox and final self-check, with tasks of its
own. The BMB arm always runs on the official write surface (ADR-0101: the
`bmb mcp` flags of `bench/brb-write`) with this set's prompt, the write set's
with one rule changed: integer `as` conversions are allowed (several tasks
index with element values, which the milestone-1 rules could not convert).

The tasks come from QuixBugs (Lin et al., 2017; MIT licence, see README):
each is one of its programs, restated over BRB's types, and its documented
defect is one of the task's Rust mutants and, in BMB form, its weak control.

Agents: `oracle` (reference: 0 defects in both arms, CI), `weak` (a plausible
wrong solution that passes its own check; the hidden judge finds the defect,
CI), `openai:MODEL`, `anthropic:MODEL` (ADR-0066).

Standard library only.
"""
import argparse
import concurrent.futures
import hashlib
import importlib.util
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BRB = os.path.join(ROOT, "bench", "brb")
WRITE = os.path.join(ROOT, "bench", "brb-write")
TASK_SET = "brb-hard"


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


brb = _load("brb_hard_brb_harness", os.path.join(BRB, "harness.py"))
judge = brb.judge
m1 = brb.m1
ARMS = brb.ARMS


def read_bytes(path):
    with open(path, "rb") as f:
        return f.read()


def frozen_files():
    """This set's harness, README (the registered verdict rules) and tasks,
    and the MANIFESTs of BRB and of its write-surface set, which hold
    everything else a session runs on."""
    files = ["harness.py", "README.md", "prompt-bmb.md", "../brb/MANIFEST", "../brb-write/MANIFEST"]
    return files + sorted("tasks/" + f for f in os.listdir(os.path.join(HERE, "tasks")) if f.endswith(".py"))


def manifest_text():
    return "".join(f"{hashlib.sha256(read_bytes(os.path.join(HERE, f))).hexdigest()}  {f}\n"
                   for f in frozen_files())


def manifest_digest():
    return hashlib.sha256(read_bytes(os.path.join(HERE, "MANIFEST"))).hexdigest()


def check_manifest():
    path = os.path.join(HERE, "MANIFEST")
    if not os.path.exists(path) or read_bytes(path).decode() != manifest_text():
        sys.exit("frozen inputs differ from MANIFEST: refusing to run (ADR-0021 §1)")


def tasks():
    return judge.load_tasks(os.path.join(HERE, "tasks"))


def adequacy(task):
    tmp = tempfile.mkdtemp(prefix=f"brb-hard-mut-{task.ID}-")
    try:
        return {"id": task.ID, "mutants": judge.judge_mutants(task, tmp)}
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    mf = sub.add_parser("manifest")
    mf.add_argument("--write", action="store_true")
    rn = sub.add_parser("run")
    rn.add_argument("--arm", choices=ARMS, required=True)
    rn.add_argument("--agent", required=True)
    rn.add_argument("--tasks")
    rn.add_argument("--out")
    rn.add_argument("--expect", choices=["all-pass", "weak-fail"])
    rn.add_argument("--jobs", type=int, default=1)
    rn.add_argument("--sessions", help="directory for session records (backlog INF-17)")
    rn.add_argument("--proptest", choices=["on", "off"], default="on")
    ad = sub.add_parser("adequacy")
    ad.add_argument("--jobs", type=int, default=1)
    ad.add_argument("--tasks")
    sm = sub.add_parser("summarize")
    sm.add_argument("results", nargs="+")
    args = ap.parse_args()

    if args.cmd == "summarize":
        sys.stdout.write(brb.summarize(args.results))
        return
    if args.cmd == "manifest":
        if args.write:
            with open(os.path.join(HERE, "MANIFEST"), "w") as f:
                f.write(manifest_text())
        else:
            check_manifest()
            print("MANIFEST ok")
        return

    check_manifest()
    brb.check_manifest()
    ts = tasks()
    if args.tasks:
        wanted = set(args.tasks.split(","))
        ts = [t for t in ts if t.ID in wanted]
    pool = concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs))

    if args.cmd == "adequacy":
        res = list(pool.map(adequacy, ts))
        bad = 0
        for r in res:
            print(json.dumps(r, ensure_ascii=False))
            bad += sum(1 for m in r["mutants"] if not m["killed"])
        total = sum(len(r["mutants"]) for r in res)
        print(json.dumps({"summary": True, "tasks": len(res), "mutants": total, "survived": bad}))
        if bad or not total:
            sys.exit("a mutant survived the hidden cases (or did not build)")
        return

    if args.arm == "bmb":
        # The write surface's `bmb mcp` flags (its MANIFEST checked), then
        # this set's prompt: the write set's with integer `as` allowed.
        brb.use_variant(WRITE)
        brb.VARIANT = HERE
    proptest = args.proptest == "on"
    commit = m1.run_commit()
    head = {"agent": args.agent, "task_set": TASK_SET, "arm": args.arm, "manifest": manifest_digest(),
            "commit": commit, "tasks": [t.ID for t in ts], **m1.condition_fields()}
    log = m1.resultlog.ResultLog(args.out, head)

    def session(t):
        r = brb.run_session(t, args.arm, args.agent, tmp_target, proptest, args.sessions)
        log.add(r)
        return r

    tmp_target = tempfile.mkdtemp(prefix="brb-hard-target-")
    try:
        if args.arm == "rust":
            brb.warm_rust(tmp_target, proptest)
        list(pool.map(session, [t for t in ts if not log.done(t.ID)]))
    finally:
        shutil.rmtree(tmp_target, ignore_errors=True)
    results = [log.rows[t.ID] for t in ts]
    for r in results:
        print(json.dumps(r, ensure_ascii=False))
    n = len(results)
    summary = {
        "summary": True, "task_set": TASK_SET, "arm": args.arm, "agent": args.agent, "tasks": n,
        "proptest": proptest if args.arm == "rust" else None,
        "manifest": manifest_digest(), "commit": commit, **m1.condition_fields(),
        "limits": {"tool_calls": brb.MAX_TOOL_CALLS, "session_output_tokens": brb.SESSION_OUTPUT_TOKENS,
                   "session_output_counts": "visible (ADR-0076)"},
        "success": f"{sum(r['success'] for r in results)}/{n}",
        "total_tokens": sum(r["total_tokens"] for r in results),
    }
    log.finish(results, summary)
    print(json.dumps(summary, ensure_ascii=False))
    if args.expect == "all-pass" and not all(r["success"] and r["final_check_ok"] for r in results):
        sys.exit("expected every task to pass the hidden judge and its own checker")
    if args.expect == "weak-fail":
        if not results or any(r["success"] for r in results):
            sys.exit("expected the hidden judge to find a defect in every weak solution")
        if any(r["last_check_ok"] is not True or r["final_check_ok"] is not True for r in results):
            sys.exit("expected every weak solution to pass its own checker (verify / cargo test)")


if __name__ == "__main__":
    main()
