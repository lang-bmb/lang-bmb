#!/usr/bin/env python3
"""Speed and size of the proved libraries against Rust (backlog T4-01, T4-02).

Each bench is a program pair: `NAME_speed.bmb` with `lib/NAME.bmb`, built by
`bmb build` (clang -O2; bounds checks removed only by proofs), and
`NAME_speed.rs` built by `rustc -C opt-level=2`:

  utf8     Rust's `std::str::from_utf8` (the standard library's validator)
  sha256   the same algorithm in safe Rust, standard library only (the code of
           compiler/src/sha256.rs), and the `sha2` crate (`sha2_crate/`, its
           version and checksum pinned by Cargo.lock, built by `cargo build
           --release --locked` at opt-level 2; approved 2026-11-01 as a bench
           arm, not a compiler dependency). Without SHA-NI the crate runs its
           software backend; the report records whether the CPU has SHA-NI.

The method is bench/baseline/run.py's: the outputs must agree before any
timing; runs are interleaved round-robin; the reported time is the median
wall-clock of RUNS runs. Size is the stripped executable.

Usage: run.py [--runs N] [--out results/DATE.md]
"""
import argparse
import datetime
import json
import os
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(ROOT, "bench", "baseline"))
from run import RUSTC, environment, sh  # noqa: E402

BENCHES = ["utf8", "sha256"]
# The compared programs of each bench; "rust" is the standard-library twin.
VARIANTS = {"utf8": ["bmb", "rust"], "sha256": ["bmb", "rust", "sha2"]}
LABELS = {"rust": None, "sha2": "`sha2` 크레이트"}
SHA2_CRATE = os.path.join(HERE, "sha2_crate")


def build(tmp):
    bmb = os.path.join(ROOT, "target", "release", "bmb")
    sh(["cargo", "build", "--release", "--quiet"], cwd=ROOT)
    exes = {}
    for b in BENCHES:
        exes[(b, "bmb")] = os.path.join(tmp, f"{b}.bmb")
        sh([bmb, "build", os.path.join(HERE, f"{b}_speed.bmb"), os.path.join(ROOT, "lib", f"{b}.bmb"),
            "-o", exes[(b, "bmb")]])
        exes[(b, "rust")] = os.path.join(tmp, f"{b}.rust")
        sh(RUSTC + ["-o", exes[(b, "rust")], os.path.join(HERE, f"{b}_speed.rs")])
    target = os.path.join(tmp, "sha2-target")
    sh(["cargo", "build", "--release", "--locked", "--quiet", "--target-dir", target], cwd=SHA2_CRATE)
    exes[("sha256", "sha2")] = os.path.join(tmp, "sha256.sha2")
    shutil.copy(os.path.join(target, "release", "sha256-speed-sha2"), exes[("sha256", "sha2")])
    sizes = {}
    for k, exe in exes.items():
        stripped = exe + ".stripped"
        shutil.copy(exe, stripped)
        sh(["strip", stripped])
        sizes[k] = os.path.getsize(stripped)
    return exes, sizes


def sha2_version():
    lock = open(os.path.join(SHA2_CRATE, "Cargo.lock")).read()
    return lock.split('name = "sha2"\nversion = "', 1)[1].split('"', 1)[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=9)
    ap.add_argument("--out")
    args = ap.parse_args()
    tmp = tempfile.mkdtemp(prefix="bmb-lib-bench-")
    try:
        exes, sizes = build(tmp)
        outputs = {}
        for b in BENCHES:
            outs = {v: sh([exes[(b, v)]]) for v in VARIANTS[b]}
            if len(set(outs.values())) != 1:
                sys.exit(f"{b}: outputs differ: {outs}")
            outputs[b] = outs["bmb"].split()
        times = {k: [] for k in exes}
        for _ in range(args.runs):
            for b in BENCHES:
                for v in VARIANTS[b]:
                    t0 = time.perf_counter()
                    subprocess.run([exes[(b, v)]], check=True, stdout=subprocess.DEVNULL)
                    times[(b, v)].append(time.perf_counter() - t0)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    env = environment()
    env["sha_ni"] = "sha_ni" in open("/proc/cpuinfo").read().split()
    rows = []
    for b in BENCHES:
        med = {v: statistics.median(times[(b, v)]) for v in VARIANTS[b]}
        rows.append({
            "bench": b, "output": outputs[b],
            "ms": {v: round(med[v] * 1000, 1) for v in VARIANTS[b]},
            "ms_all": {v: [round(t * 1000, 1) for t in times[(b, v)]] for v in VARIANTS[b]},
            "bytes": {v: sizes[(b, v)] for v in VARIANTS[b]},
            **{f"bmb_vs_{v}": round(med["bmb"] / med[v], 3) for v in VARIANTS[b] if v != "bmb"},
        })
    lines = [
        f"# 라이브러리 속도·크기 ({env['date']})",
        "",
        f"- 환경: {env['cpu']} ({env['cores']} cores), {env['os']}",
        f"- 도구: {env['rustc']}; {env['clang']}; bmb `build`(clang -O2)",
        f"- 방법: 변형별 {args.runs}회 교차 실행의 중앙값(벽시계). 출력 일치 확인 후 측정. 크기는 strip한 실행 파일.",
        "- 비교 대상: utf8은 Rust 표준 라이브러리 `std::str::from_utf8`, sha256은 같은 알고리즘의 안전한 Rust(표준 라이브러리만)와 "
        f"`sha2` 크레이트 {sha2_version()}(`cargo build --release --locked`, opt-level 2). "
        f"이 CPU의 SHA-NI: {'있음(크레이트가 하드웨어 명령을 쓴다)' if env.get('sha_ni') else '없음(크레이트의 소프트웨어 구현)'}.",
        "- 비율 < 1 이면 BMB가 빠르다.",
        "",
        "| 벤치 | 비교 대상 | BMB ms | 대상 ms | BMB/대상 | BMB 바이트 | 대상 바이트 |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: |",
    ]
    for r in rows:
        m, s = r["ms"], r["bytes"]
        for v in VARIANTS[r["bench"]][1:]:
            what = LABELS[v] or ("`std::str::from_utf8`" if r["bench"] == "utf8" else "같은 알고리즘의 Rust")
            lines.append(f"| {r['bench']} | {what} | {m['bmb']} | {m[v]} | {r[f'bmb_vs_{v}']} | {s['bmb']} | {s[v]} |")
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
