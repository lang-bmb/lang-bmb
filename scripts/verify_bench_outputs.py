#!/usr/bin/env python3
"""
Verify BMB <-> C benchmark output equivalence.

Cycle 2769 (2026-05-12): catches correctness/fairness regressions like the
lexer 0-token bug found in Cycle 2765 — when BMB output diverges from C,
performance ratios become meaningless.

Usage:
    python3 scripts/verify_bench_outputs.py [--tier {1,3,all}] [--rebuild]
        [--verbose] [--json OUT.json]

Exit:
    0 = all matched (or no benches found)
    1 = at least one mismatch
    2 = build/run failure (separate from mismatch)
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

REPO = Path(__file__).resolve().parents[1]
BENCHES = REPO / "ecosystem" / "benchmark-bmb" / "benches"
BMB_EXE = REPO / "target" / "release" / "bmb.exe"
BMB_RUNTIME = REPO / "bmb" / "runtime"

# Tier 1: synthetic compute benches with both BMB+C
TIER1 = [
    "compute/binary_trees", "compute/fannkuch", "compute/fasta",
    "compute/fibonacci", "compute/hash_table", "compute/knapsack",
    "compute/mandelbrot", "compute/n_body", "compute/nqueen",
    "compute/spectral_norm",
]
TIER3 = [
    "real_world/brainfuck", "real_world/csv_parse", "real_world/http_parse",
    "real_world/json_parse", "real_world/json_serialize", "real_world/lexer",
    "real_world/sorting",
]


@dataclass
class Result:
    bench: str
    tier: int
    bmb_ok: bool
    c_ok: bool
    output_match: bool
    bmb_stdout: str
    c_stdout: str
    diff: str
    note: str = ""


def normalize(s: str) -> str:
    """Strip trailing whitespace per line, collapse final blanks."""
    return "\n".join(line.rstrip() for line in s.splitlines()).strip()


def run_exe(exe: Path, timeout: float = 60.0) -> tuple[bool, str]:
    if not exe.exists():
        return False, f"(exe missing: {exe.name})"
    try:
        r = subprocess.run(
            [str(exe)], capture_output=True, text=True, timeout=timeout)
        if r.returncode != 0:
            return False, f"(exit={r.returncode})\n{r.stdout}"
        return True, r.stdout
    except subprocess.TimeoutExpired:
        return False, "(timeout)"
    except Exception as e:
        return False, f"(error: {e})"


def build_c(bench_dir: Path) -> tuple[bool, str]:
    src = bench_dir / "c" / "main.c"
    exe = bench_dir / "c" / "main_verify.exe"
    if not src.exists():
        return False, "no C source"
    r = subprocess.run(
        ["gcc", "-O2", "-march=native", str(src), "-o", str(exe)],
        capture_output=True, text=True)
    if r.returncode != 0:
        return False, f"gcc failed: {r.stderr[:200]}"
    return True, str(exe)


def build_bmb(bench_dir: Path) -> tuple[bool, str]:
    src = bench_dir / "bmb" / "main.bmb"
    exe = bench_dir / "bmb" / "main_verify.exe"
    if not src.exists():
        return False, "no BMB source"
    env = os.environ.copy()
    env["BMB_RUNTIME_PATH"] = str(BMB_RUNTIME)
    r = subprocess.run(
        [str(BMB_EXE), "build", str(src), "-o", str(exe)],
        capture_output=True, text=True, env=env, cwd=str(bench_dir / "bmb"))
    if r.returncode != 0:
        return False, f"bmb build failed: {r.stderr[:200]}"
    return True, str(exe)


def diff_lines(a: str, b: str, max_lines: int = 10) -> str:
    a_lines = a.splitlines()
    b_lines = b.splitlines()
    out = []
    for i in range(max(len(a_lines), len(b_lines))):
        la = a_lines[i] if i < len(a_lines) else "<EOF>"
        lb = b_lines[i] if i < len(b_lines) else "<EOF>"
        if la != lb:
            out.append(f"  line {i+1}: BMB={la!r}  C={lb!r}")
            if len(out) >= max_lines:
                out.append(f"  ... ({max(len(a_lines), len(b_lines)) - i - 1} more)")
                break
    return "\n".join(out) if out else "(identical after normalization)"


def verify_bench(rel: str, rebuild: bool, verbose: bool) -> Result:
    bench_dir = BENCHES / rel
    tier = 1 if rel.startswith("compute/") else 3

    bmb_exe = bench_dir / "bmb" / "main_verify.exe"
    c_exe = bench_dir / "c" / "main_verify.exe"

    if rebuild or not bmb_exe.exists():
        ok, msg = build_bmb(bench_dir)
        if not ok:
            return Result(rel, tier, False, False, False, "", "", "", f"BMB build: {msg}")
    if rebuild or not c_exe.exists():
        ok, msg = build_c(bench_dir)
        if not ok:
            return Result(rel, tier, False, False, False, "", "", "", f"C build: {msg}")

    bmb_ok, bmb_out = run_exe(bmb_exe)
    c_ok, c_out = run_exe(c_exe)

    bmb_norm = normalize(bmb_out) if bmb_ok else ""
    c_norm = normalize(c_out) if c_ok else ""

    match = bmb_ok and c_ok and bmb_norm == c_norm
    diff = "" if match else diff_lines(bmb_norm, c_norm)

    return Result(rel, tier, bmb_ok, c_ok, match, bmb_norm, c_norm, diff)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tier", choices=["1", "3", "all"], default="all")
    ap.add_argument("--rebuild", action="store_true")
    ap.add_argument("--verbose", action="store_true")
    ap.add_argument("--json", help="write machine-readable JSON report")
    args = ap.parse_args()

    if args.tier == "1":
        benches = TIER1
    elif args.tier == "3":
        benches = TIER3
    else:
        benches = TIER1 + TIER3

    if not BMB_EXE.exists():
        print(f"ERROR: BMB compiler not found at {BMB_EXE}", file=sys.stderr)
        print("Run: cargo build --release --features llvm --target x86_64-pc-windows-gnu", file=sys.stderr)
        sys.exit(2)

    results = []
    print(f"Verifying {len(benches)} benches (tier={args.tier})...")
    t0 = time.time()
    for rel in benches:
        print(f"  {rel:<40}", end="", flush=True)
        r = verify_bench(rel, args.rebuild, args.verbose)
        results.append(r)
        if r.note:
            print(f" SKIP — {r.note}")
        elif not r.bmb_ok:
            print(" FAIL (BMB run)")
        elif not r.c_ok:
            print(" FAIL (C run)")
        elif r.output_match:
            print(" PASS")
        else:
            print(" MISMATCH")
            if args.verbose:
                print(r.diff)

    elapsed = time.time() - t0
    matched = sum(1 for r in results if r.output_match)
    mismatched = sum(1 for r in results if r.bmb_ok and r.c_ok and not r.output_match)
    failed = sum(1 for r in results if r.note or not r.bmb_ok or not r.c_ok)

    print(f"\n=== Summary ===")
    print(f"  Matched:    {matched}/{len(results)}")
    print(f"  Mismatched: {mismatched}")
    print(f"  Failed:     {failed}")
    print(f"  Time:       {elapsed:.1f}s")

    if args.json:
        with open(args.json, "w") as f:
            json.dump({
                "tier": args.tier,
                "total": len(results),
                "matched": matched,
                "mismatched": mismatched,
                "failed": failed,
                "elapsed_sec": round(elapsed, 1),
                "results": [asdict(r) for r in results],
            }, f, indent=2)
        print(f"  JSON:       {args.json}")

    if mismatched > 0:
        sys.exit(1)
    if failed > 0 and matched == 0:
        sys.exit(2)
    sys.exit(0)


if __name__ == "__main__":
    main()
