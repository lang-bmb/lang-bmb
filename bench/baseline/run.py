#!/usr/bin/env python3
"""Rust baseline for BMB (backlog 1E-05, genesis §3.4-3.5).

Builds each benchmark three ways from the same algorithm:
  bmb     - `bmb build` (clang -O2); bounds checks removed only by proofs
  safe    - rustc -C opt-level=2, ordinary indexing (bounds-checked)
  unck    - rustc -C opt-level=2 --cfg unchecked (get_unchecked)
Outputs must agree before any timing is reported. Runs are interleaved
round-robin; the reported time is the median wall-clock of RUNS runs.

Usage: run.py [--runs N] [--out results/DATE.md]
"""
import argparse
import datetime
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BENCHES = ["sieve", "isort", "kadane", "gcdsum"]
VARIANTS = ["bmb", "safe", "unck"]
RUSTC = ["rustc", "-C", "opt-level=2", "-A", "warnings"]


def sh(cmd, **kw):
    return subprocess.run(cmd, check=True, capture_output=True, text=True, **kw).stdout


def build(tmp):
    bmb = os.path.join(ROOT, "target", "release", "bmb")
    sh(["cargo", "build", "--release", "--quiet"], cwd=ROOT)
    exes = {}
    for b in BENCHES:
        exes[(b, "bmb")] = os.path.join(tmp, f"{b}.bmb")
        sh([bmb, "build", os.path.join(HERE, f"{b}.bmb"), "-o", exes[(b, "bmb")]])
        exes[(b, "safe")] = os.path.join(tmp, f"{b}.safe")
        sh(RUSTC + ["-o", exes[(b, "safe")], os.path.join(HERE, f"{b}.rs")])
        exes[(b, "unck")] = os.path.join(tmp, f"{b}.unck")
        sh(RUSTC + ["--cfg", "unchecked", "-o", exes[(b, "unck")], os.path.join(HERE, f"{b}.rs")])
    return exes


def environment():
    cpu = ""
    try:
        for line in open("/proc/cpuinfo"):
            if line.startswith("model name"):
                cpu = line.split(":", 1)[1].strip()
                break
    except OSError:
        pass
    return {
        "date": datetime.date.today().isoformat(),
        "cpu": cpu,
        "cores": os.cpu_count(),
        "os": platform.platform(),
        "rustc": sh(["rustc", "-V"]).strip(),
        "clang": sh(["clang", "--version"]).splitlines()[0],
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=9)
    ap.add_argument("--out")
    args = ap.parse_args()
    tmp = tempfile.mkdtemp(prefix="bmb-bench-")
    try:
        exes = build(tmp)
        # Correctness first: every variant prints the same thing.
        outputs = {}
        for b in BENCHES:
            outs = {v: sh([exes[(b, v)]]) for v in VARIANTS}
            if len(set(outs.values())) != 1:
                sys.exit(f"{b}: outputs differ: {outs}")
            outputs[b] = outs["bmb"].split()
        times = {k: [] for k in exes}
        for _ in range(args.runs):
            for b in BENCHES:
                for v in VARIANTS:
                    t0 = time.perf_counter()
                    subprocess.run([exes[(b, v)]], check=True, stdout=subprocess.DEVNULL)
                    times[(b, v)].append(time.perf_counter() - t0)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    env = environment()
    rows = []
    for b in BENCHES:
        med = {v: statistics.median(times[(b, v)]) for v in VARIANTS}
        rows.append({
            "bench": b,
            "output": outputs[b],
            "ms": {v: round(med[v] * 1000, 1) for v in VARIANTS},
            "bmb_vs_safe": round(med["bmb"] / med["safe"], 3),
            "bmb_vs_unck": round(med["bmb"] / med["unck"], 3),
        })
    lines = [
        f"# Rust 기준선 ({env['date']})",
        "",
        f"- 환경: {env['cpu']} ({env['cores']} cores), {env['os']}",
        f"- 도구: {env['rustc']}; {env['clang']}; bmb `build`(clang -O2)",
        f"- 방법: 변형별 {args.runs}회 교차 실행의 중앙값(벽시계). 출력 일치 확인 후 측정.",
        "- 비율 < 1 이면 BMB가 빠르다.",
        "",
        "| 벤치 | BMB ms | Rust safe ms | Rust unchecked ms | BMB/safe | BMB/unchecked |",
        "| --- | ---: | ---: | ---: | ---: | ---: |",
    ]
    for r in rows:
        m = r["ms"]
        lines.append(
            f"| {r['bench']} | {m['bmb']} | {m['safe']} | {m['unck']} | {r['bmb_vs_safe']} | {r['bmb_vs_unck']} |"
        )
    report = "\n".join(lines) + "\n"
    print(report)
    if args.out:
        os.makedirs(os.path.dirname(os.path.abspath(args.out)), exist_ok=True)
        with open(args.out, "w") as f:
            f.write(report)
        with open(os.path.splitext(args.out)[0] + ".jsonl", "w") as f:
            f.write(json.dumps({"env": env, "runs": args.runs}, ensure_ascii=False) + "\n")
            for r in rows:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
