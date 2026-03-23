#!/usr/bin/env python3
"""Benchmark: bmb-compute vs Python math / built-ins"""
import sys, os, time, math, ctypes
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_compute as bmb

# ── Pure Python / stdlib baselines ────────────────────────────────────────

def py_dot_product(a, b):
    return sum(x * y for x, y in zip(a, b))

def py_sum(arr):
    return builtins_sum(arr)

import builtins
builtins_sum = builtins.sum    # avoid shadowing by bmb_compute's sum

# ── Benchmark harness ──────────────────────────────────────────────────────

def bench(fn, iterations=1000):
    for _ in range(10): fn()
    t0 = time.perf_counter()
    for _ in range(iterations): fn()
    return (time.perf_counter() - t0) / iterations

def run(name, bmb_fn, py_fn, iterations=1000):
    bmb_us = bench(bmb_fn, iterations) * 1e6
    py_us  = bench(py_fn,  iterations) * 1e6
    speedup = py_us / bmb_us if bmb_us > 0 else float('inf')
    return name, bmb_us, py_us, speedup

# ── Test data ─────────────────────────────────────────────────────────────
DATA_100  = list(range(1, 101))                          # [1..100]
VEC_A     = list(range(1, 51))                           # 50-element vector
VEC_B     = list(range(51, 101))

if __name__ == '__main__':
    print("=" * 68)
    print(f"{'Benchmark: bmb-compute vs Python math / built-ins':^68}")
    print("=" * 68)
    print(f"{'Function':<26} {'BMB (µs)':>10} {'Python (µs)':>12} {'Speedup':>9}")
    print("-" * 68)

    results = [
        # factorial
        run("factorial(20)",
            lambda: bmb.factorial(20),
            lambda: math.factorial(20)),
        run("factorial(12)",
            lambda: bmb.factorial(12),
            lambda: math.factorial(12)),
        # sum
        run("sum (100 ints)",
            lambda: bmb.sum(DATA_100),
            lambda: py_sum(DATA_100)),
        # dot_product (50-element vectors)
        run("dot_product (50)",
            lambda: bmb.dot_product(VEC_A, VEC_B),
            lambda: py_dot_product(VEC_A, VEC_B)),
        # sqrt (integer isqrt)
        run("sqrt(1000000)",
            lambda: bmb.sqrt(1000000),
            lambda: math.isqrt(1000000)),
        run("sqrt(144)",
            lambda: bmb.sqrt(144),
            lambda: math.isqrt(144)),
        # ipow vs pow
        run("ipow(2, 30)",
            lambda: bmb.ipow(2, 30),
            lambda: pow(2, 30)),
        # min / max scalar
        run("min(a, b)",
            lambda: bmb.min(12345, 67890),
            lambda: builtins.min(12345, 67890)),
        run("max(a, b)",
            lambda: bmb.max(12345, 67890),
            lambda: builtins.max(12345, 67890)),
        # clamp
        run("clamp(x, lo, hi)",
            lambda: bmb.clamp(500, 0, 1000),
            lambda: max(0, min(1000, 500))),
        # min_val / max_val over array
        run("min_val (100 ints)",
            lambda: bmb.min_val(DATA_100),
            lambda: builtins.min(DATA_100)),
        run("max_val (100 ints)",
            lambda: bmb.max_val(DATA_100),
            lambda: builtins.max(DATA_100)),
    ]

    for name, bmb_us, py_us, speedup in results:
        marker = "FAST" if speedup >= 2.0 else ("OK" if speedup >= 1.0 else "SLOW")
        print(f"{name:<26} {bmb_us:>10.2f} {py_us:>12.2f} {speedup:>8.2f}x {marker}")

    print("=" * 68)
    wins = sum(1 for _, b, p, _ in results if b < p)
    print(f"BMB faster in {wins}/{len(results)} benchmarks")
    print("Note: BMB timings include ctypes FFI overhead.")
    print("      Array benchmarks also include ctypes array allocation.")
