#!/usr/bin/env python3
"""Build time observation (backlog INF-13, ADR-0041 accepted costs).

  build_time.py [--defs N] [--out FILE.jsonl]

Generates the m3 workspace of N definitions (bench/m3/gen.py) and times
`bmb emit-llvm --cache` (proof gate and code generation; the m3 workspace
has no `main` to link) three times: cold (empty cache), warm (nothing changed)
and after a one-definition edit (a comment inside one body, which re-encodes
that definition alone, ADR-0032). CPU is user+sys of the build process
(`wait4`). One JSON line per run, with the per-definition cost of the cold
build. Time is not deterministic, so nothing here is a gate: the nightly job
records the line for the trend (it never fails on a value).
Standard library only.
"""
import argparse
import datetime
import json
import os
import platform
import re
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def timed(argv):
    t0 = time.monotonic()
    p = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    _, status, ru = os.wait4(p.pid, 0)
    wall = time.monotonic() - t0
    err = p.stderr.read().decode()
    if os.waitstatus_to_exitcode(status) != 0:
        sys.exit(f"build_time: {' '.join(argv)} failed\n{err[-2000:]}")
    return round(ru.ru_utime + ru.ru_stime, 3), round(wall, 3)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--defs", type=int, default=1000)
    ap.add_argument("--out")
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "release", "bmb"))
    args = ap.parse_args()
    with tempfile.TemporaryDirectory() as d:
        ws, cache, stats, ll = (os.path.join(d, x) for x in ("ws", "cache", "stats.json", "out.ll"))
        subprocess.run([sys.executable, os.path.join(ROOT, "bench", "m3", "gen.py"), ws, "--defs", str(args.defs)],
                       check=True, stdout=subprocess.DEVNULL)
        build = [args.bmb, "emit-llvm", "--cache", cache, "--stats", stats, os.path.join(ws, "src"), "-o", ll]
        row = {"date": datetime.date.today().isoformat(), "defs": args.defs,
               "commit": subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True,
                                        text=True).stdout.strip(),
               "cpu": next((line.split(":", 1)[1].strip() for line in open("/proc/cpuinfo")
                            if line.startswith("model name")), platform.processor())}
        row["cold_cpu"], row["cold_wall"] = timed(build)
        row["cold_encoded"] = len(json.load(open(stats))["encoded"])
        row["warm_cpu"], row["warm_wall"] = timed(build)
        # One edit: a comment inside the first executable body of the last unit.
        units = sorted(os.listdir(os.path.join(ws, "src")), key=lambda f: int(re.sub(r"\D", "", f) or 0))
        path = os.path.join(ws, "src", units[-1])
        text = open(path).read()
        m = re.search(r"\n    fn [^\n]*\n(?:[^\n]*\n)*?    \{\n", text)
        with open(path, "w") as f:
            f.write(text[:m.end()] + "        // edited\n" + text[m.end():])
        row["edit_cpu"], row["edit_wall"] = timed(build)
        row["edit_encoded"] = len(json.load(open(stats))["encoded"])
        row["cold_ms_per_def"] = round(1000 * row["cold_cpu"] / args.defs, 2)
    line = json.dumps(row)
    print(line)
    if args.out:
        with open(args.out, "a") as f:
            f.write(line + "\n")


if __name__ == "__main__":
    main()
