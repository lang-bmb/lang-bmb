#!/usr/bin/env python3
"""The surface before and after ADR-0095 on the same frozen tasks (backlog 2R-05).

ADR-0095 changed the writing surface (input aliases, shadowing, `x as T`,
inline contract clauses, the default proof context); the frozen tasks and
prompts did not change. This runs one task set with one agent twice, once
per compiler build, and puts the two summaries and the habit evidence side
by side:

  before   the compiler at REV_BEFORE (default 99138a9d, the last commit
           before ADR-0095), built in a worktree of its own
  after    the compiler at HEAD (`--after BIN`, default target/debug/bmb,
           built by the gate)

Each run goes through the frozen harness (`bench/m1/harness.py`, the
official path of ADR-0066) with `BMB` set to that build, its results in
OUT/before.jsonl and OUT/after.jsonl, and the summary table of each.
`--agent oracle` (no model) checks the plumbing: the reference solutions
pass under both. A model run needs the measuring machine's gateway.

Usage: bench/syntax/ab_surface.py OUT [--set bench/m1b] [--agent AGENT]
                                  [--before REV] [--after BIN] [--jobs N]
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def arg(name, default):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


def build(rev, out):
    """The `bmb` of commit `rev`, built in a worktree under `out`."""
    wt = os.path.join(out, "before-src")
    if not os.path.isdir(wt):
        subprocess.run(["git", "worktree", "add", "--detach", wt, rev], cwd=ROOT, check=True)
    subprocess.run(["cargo", "build", "--quiet", "--manifest-path", os.path.join(wt, "compiler", "Cargo.toml")],
                   cwd=wt, check=True, env=dict(os.environ, CARGO_TARGET_DIR=os.path.join(out, "before-target")))
    return os.path.join(out, "before-target", "debug", "bmb")


def run(label, bmb, out, task_set, agent, jobs):
    res = os.path.join(out, f"{label}.jsonl")
    if os.path.exists(res):
        os.remove(res)
    h = os.path.join(ROOT, "bench", "m1", "harness.py")
    env = dict(os.environ, BMB=bmb)
    subprocess.run([sys.executable, h, "--set", task_set, "run", "--agent", agent, "--jobs", jobs, "--out", res],
                   cwd=ROOT, env=env, check=False, capture_output=True)
    table = subprocess.run([sys.executable, h, "--set", task_set, "summarize", res],
                           cwd=ROOT, env=env, capture_output=True, text=True).stdout
    return table


def main():
    if len(sys.argv) < 2 or sys.argv[1].startswith("--"):
        sys.exit(__doc__)
    out = os.path.abspath(sys.argv[1])
    os.makedirs(out, exist_ok=True)
    task_set = arg("--set", "bench/m1b")
    agent = arg("--agent", "oracle")
    jobs = arg("--jobs", "2")
    before = build(arg("--before", "99138a9d"), out)
    after = os.path.abspath(arg("--after", os.path.join(ROOT, "target", "debug", "bmb")))
    for label, bmb in (("before", before), ("after", after)):
        print(f"## {label} ({bmb})\n")
        print(run(label, bmb, out, task_set, agent, jobs))


if __name__ == "__main__":
    main()
