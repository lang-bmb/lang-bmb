#!/usr/bin/env python3
"""Runs the gate's independent checks concurrently (backlog INF-08).

  gate_jobs.py [--slots N] [--timeout S] [--cargo-json FILE] [--job NAME=CMD]...

--cargo-json reads `cargo test --no-run --message-format=json` output and adds
one job per test executable (integration tests, unit tests of the lib and the
bin), run from the package directory the way `cargo test` runs them.
--job adds a shell command (bash, `set -euo pipefail`) run from the repository
root.

Every job runs to the end; output goes to a log that is printed only when the
job fails. The exit code is non-zero when any job fails or times out, and the
failing jobs are named last. Jobs start longest-first by the durations of the
previous run (target/gate-times.json; only the order depends on it).

Standard library only.
"""
import argparse
import json
import os
import signal
import subprocess
import sys
import tempfile
import threading
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TIMES = os.path.join(ROOT, "target", "gate-times.json")


def cargo_jobs(path):
    jobs = []
    for line in open(path):
        line = line.strip()
        if not line.startswith("{"):
            continue
        m = json.loads(line)
        if m.get("reason") != "compiler-artifact" or not m.get("executable"):
            continue
        if not m["profile"]["test"]:
            continue  # the `bmb` binary itself, not a test harness
        kind = m["target"]["kind"][0]
        name = m["target"]["name"] if kind == "test" else f"{kind}:{m['target']['name']}"
        cwd = os.path.dirname(m["manifest_path"])
        jobs.append((f"test {name}", [m["executable"], "--quiet"], cwd))
    if not jobs:
        sys.exit(f"{path}: no test executables")
    return jobs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--slots", type=int, default=os.cpu_count() or 2)
    ap.add_argument("--timeout", type=float, default=1800)
    ap.add_argument("--cargo-json")
    ap.add_argument("--job", action="append", default=[])
    args = ap.parse_args()

    jobs = cargo_jobs(args.cargo_json) if args.cargo_json else []
    for spec in args.job:
        name, cmd = spec.split("=", 1)
        jobs.append((name, ["bash", "-c", "set -euo pipefail\n" + cmd], ROOT))
    try:
        last = json.load(open(TIMES))
    except (OSError, ValueError):
        last = {}
    # Longest first; unknown jobs keep their given order ahead of known short ones.
    order = sorted(range(len(jobs)), key=lambda i: (-last.get(jobs[i][0], 1e9), i))

    logs = tempfile.mkdtemp(prefix="bmb-gate-")
    lock = threading.Lock()
    results = {}
    queue = list(order)
    running = set()

    def worker():
        while True:
            with lock:
                if not queue:
                    return
                i = queue.pop(0)
            name, cmd, cwd = jobs[i]
            log = os.path.join(logs, f"{i}.log")
            start = time.monotonic()
            with open(log, "w") as out:
                p = subprocess.Popen(cmd, cwd=cwd, stdout=out, stderr=subprocess.STDOUT,
                                     stdin=subprocess.DEVNULL, start_new_session=True)
                # A timed-out job is killed with everything it started (its own session).
                timer = threading.Timer(args.timeout, os.killpg, (p.pid, signal.SIGKILL))
                timer.start()
                with lock:
                    running.add(p.pid)
                # wait4: the job's own CPU time (its waited-for children included).
                _, status, usage = os.wait4(p.pid, 0)
                p.returncode = os.waitstatus_to_exitcode(status)
                timed_out = not timer.is_alive()
                timer.cancel()
                with lock:
                    running.discard(p.pid)
                code = "timeout" if timed_out else p.returncode
            secs = time.monotonic() - start
            cpu = usage.ru_utime + usage.ru_stime
            with lock:
                results[i] = (code, secs, log)
                mark = "ok" if code == 0 else f"FAILED ({code})"
                print(f"   {mark:>8} {secs:6.1f}s wall {cpu:6.1f}s cpu  {name}", flush=True)

    threads = [threading.Thread(target=worker, daemon=True) for _ in range(max(1, args.slots))]
    for t in threads:
        t.start()
    try:
        for t in threads:
            while t.is_alive():
                t.join(0.5)
    except KeyboardInterrupt:
        # Jobs run in their own sessions, so the terminal's signal does not reach them.
        with lock:
            queue.clear()
            for pid in running:
                try:
                    os.killpg(pid, signal.SIGKILL)
                except OSError:
                    pass
        return 130

    failed = [i for i in order if results[i][0] != 0]
    for i in failed:
        code, secs, log = results[i]
        print(f"\n== {jobs[i][0]}: FAILED ({code}) after {secs:.1f}s; output:")
        sys.stdout.write(open(log, errors="replace").read())
    try:
        os.makedirs(os.path.dirname(TIMES), exist_ok=True)
        last.update({jobs[i][0]: round(results[i][1], 1) for i in results})
        with open(TIMES, "w") as f:
            json.dump(last, f, indent=1, sort_keys=True)
    except OSError:
        pass
    if failed:
        print("\nfailed: " + ", ".join(jobs[i][0] for i in failed))
        return 1
    for i in order:
        os.unlink(results[i][2])
    os.rmdir(logs)
    return 0


if __name__ == "__main__":
    sys.exit(main())
