#!/usr/bin/env python3
"""Benchmark: bmb-algo vs pure Python implementations"""
import sys, os, time, ctypes
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_algo as bmb

# ── Pure Python baselines ──────────────────────────────────────────────────

def py_knapsack(weights, values, capacity):
    n = len(weights)
    dp = [0] * (capacity + 1)
    for i in range(n):
        for w in range(capacity, weights[i] - 1, -1):
            dp[w] = max(dp[w], dp[w - weights[i]] + values[i])
    return dp[capacity]

def py_fibonacci(n):
    if n <= 1: return n
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b

def py_prime_count(n):
    if n < 2: return 0
    sieve = bytearray([1]) * (n + 1)
    sieve[0] = sieve[1] = 0
    for i in range(2, int(n**0.5) + 1):
        if sieve[i]:
            for j in range(i*i, n + 1, i):
                sieve[j] = 0
    return sum(sieve)

def py_nqueens(n):
    count = [0]
    def solve(row, cols, d1, d2):
        if row == n:
            count[0] += 1
            return
        avail = ((1 << n) - 1) & ~(cols | d1 | d2)
        while avail:
            bit = avail & (-avail)
            avail ^= bit
            solve(row + 1, cols | bit, (d1 | bit) << 1, (d2 | bit) >> 1)
    solve(0, 0, 0, 0)
    return count[0]

def py_quicksort(arr):
    a = list(arr)
    def qs(lo, hi):
        if lo < hi:
            p = a[hi]; i = lo
            for j in range(lo, hi):
                if a[j] <= p:
                    a[i], a[j] = a[j], a[i]; i += 1
            a[i], a[hi] = a[hi], a[i]
            qs(lo, i - 1); qs(i + 1, hi)
    qs(0, len(a) - 1)
    return a

def py_merge_sort(arr):
    if len(arr) <= 1: return list(arr)
    m = len(arr) // 2
    L, R = py_merge_sort(arr[:m]), py_merge_sort(arr[m:])
    result, i, j = [], 0, 0
    while i < len(L) and j < len(R):
        if L[i] <= R[j]: result.append(L[i]); i += 1
        else: result.append(R[j]); j += 1
    return result + L[i:] + R[j:]

def py_edit_distance(a, b):
    m, n = len(a), len(b)
    dp = list(range(n + 1))
    for i in range(1, m + 1):
        prev = dp[0]; dp[0] = i
        for j in range(1, n + 1):
            tmp = dp[j]
            dp[j] = prev if a[i-1] == b[j-1] else 1 + min(prev, dp[j], dp[j-1])
            prev = tmp
    return dp[n]

# ── Benchmark harness ──────────────────────────────────────────────────────

def bench(fn, iterations=500):
    for _ in range(10): fn()
    t0 = time.perf_counter()
    for _ in range(iterations): fn()
    return (time.perf_counter() - t0) / iterations

def run(name, bmb_fn, py_fn, iterations=500, runs=1):
    """Return (name, median_bmb_us, median_py_us, median_speedup, all_speedups)."""
    samples = []
    for _ in range(runs):
        bmb_us = bench(bmb_fn, iterations) * 1e6
        py_us  = bench(py_fn,  iterations) * 1e6
        speedup = py_us / bmb_us if bmb_us > 0 else float('inf')
        samples.append((bmb_us, py_us, speedup))
    samples.sort(key=lambda x: x[2])
    mid = samples[len(samples) // 2]
    all_speedups = [s[2] for s in samples]
    return name, mid[0], mid[1], mid[2], all_speedups

# ── Test data ─────────────────────────────────────────────────────────────

WEIGHTS   = [2, 3, 4, 5, 6, 7, 8, 9, 1, 3]
VALUES    = [3, 4, 5, 6, 7, 8, 9, 10, 2, 4]
CAPACITY  = 20
SORT_DATA = [64, 34, 25, 12, 22, 11, 90, 45, 78, 56, 33, 17, 99, 3, 60]
STR_A, STR_B = "intention", "execution"

# Larger inputs — demonstrate scaling (BMB advantage amplifies with N)
import random as _r
_r.seed(42)
WEIGHTS_LG  = [_r.randint(1, 50) for _ in range(100)]
VALUES_LG   = [_r.randint(1, 100) for _ in range(100)]
CAPACITY_LG = sum(WEIGHTS_LG) // 2
SORT_DATA_LG = [_r.randint(1, 10000) for _ in range(1000)]

def scaling_sweep(runs=1):
    """Reproduces the scaling table from README (n=10/30/100/300 knapsack, n=15/50/100/500/1000 quicksort)."""
    print("=" * 75)
    print(f"{'Scaling sweep — FFI overhead amortization':^75}")
    print("=" * 75)
    fmt_hdr = f"{'Config':<22} {'BMB (µs)':>10} {'Py (µs)':>10} {'Speedup':>10}"
    if runs > 1:
        fmt_hdr += f"  {'min-max':>14}"
    print(fmt_hdr)
    print("-" * 75)

    def gen_knapsack(n, seed=42):
        r = _r.Random(seed)
        w = [r.randint(1, 50) for _ in range(n)]
        v = [r.randint(1, 100) for _ in range(n)]
        return w, v, sum(w) // 2

    def gen_sort(n, seed=42):
        r = _r.Random(seed)
        return [r.randint(1, 10000) for _ in range(n)]

    sweep = []
    for n, iters in [(10, 500), (30, 200), (100, 100), (300, 20)]:
        w, v, c = gen_knapsack(n)
        sweep.append(run(f"knapsack(n={n})",
            lambda w=w, v=v, c=c: bmb.knapsack(w, v, c),
            lambda w=w, v=v, c=c: py_knapsack(w, v, c),
            iterations=iters, runs=runs))
    for n, iters in [(15, 500), (50, 500), (100, 500), (500, 100), (1000, 50)]:
        data = gen_sort(n)
        sweep.append(run(f"quicksort(n={n})",
            lambda d=data: bmb.quicksort(d),
            lambda d=data: py_quicksort(d),
            iterations=iters, runs=runs))

    for name, bmb_us, py_us, speedup, all_speedups in sweep:
        marker = "FAST" if speedup >= 2.0 else ("OK" if speedup >= 1.0 else "SLOW")
        line = f"{name:<22} {bmb_us:>10.2f} {py_us:>10.2f} {speedup:>9.2f}x  {marker}"
        if runs > 1:
            line += f"  {min(all_speedups):>5.2f}-{max(all_speedups):.2f}x"
        print(line)
    print("=" * 75)


if __name__ == '__main__':
    runs = 1
    for arg in sys.argv:
        if arg.startswith('--runs='):
            runs = int(arg.split('=', 1)[1])

    print("=" * 75)
    title = 'Benchmark: bmb-algo vs pure Python' + (f' (median of {runs} runs)' if runs > 1 else '')
    print(f"{title:^75}")
    print("=" * 75)
    hdr = f"{'Function':<20} {'BMB (µs)':>10} {'Py (µs)':>10} {'Speedup':>10}"
    if runs > 1:
        hdr += f"  {'min-max':>14}"
    print(hdr)
    print("-" * 75)

    results = [
        run("knapsack(10)",
            lambda: bmb.knapsack(WEIGHTS, VALUES, CAPACITY),
            lambda: py_knapsack(WEIGHTS, VALUES, CAPACITY), runs=runs),
        run("knapsack(100)",
            lambda: bmb.knapsack(WEIGHTS_LG, VALUES_LG, CAPACITY_LG),
            lambda: py_knapsack(WEIGHTS_LG, VALUES_LG, CAPACITY_LG),
            iterations=100, runs=runs),
        run("fibonacci(30)",
            lambda: bmb.fibonacci(30),
            lambda: py_fibonacci(30), runs=runs),
        run("prime_count(10000)",
            lambda: bmb.prime_count(10000),
            lambda: py_prime_count(10000),
            iterations=100, runs=runs),
        run("nqueens(10)",
            lambda: bmb.nqueens(10),
            lambda: py_nqueens(10),
            iterations=100, runs=runs),
        run("quicksort(15)",
            lambda: bmb.quicksort(SORT_DATA),
            lambda: py_quicksort(SORT_DATA), runs=runs),
        run("quicksort(1000)",
            lambda: bmb.quicksort(SORT_DATA_LG),
            lambda: py_quicksort(SORT_DATA_LG),
            iterations=50, runs=runs),
        run("merge_sort(15)",
            lambda: bmb.merge_sort(SORT_DATA),
            lambda: py_merge_sort(SORT_DATA), runs=runs),
        run("edit_distance",
            lambda: bmb.edit_distance(STR_A, STR_B),
            lambda: py_edit_distance(STR_A, STR_B), runs=runs),
    ]

    for name, bmb_us, py_us, speedup, all_speedups in results:
        marker = "FAST" if speedup >= 2.0 else ("OK" if speedup >= 1.0 else "SLOW")
        line = f"{name:<20} {bmb_us:>10.2f} {py_us:>10.2f} {speedup:>9.2f}x  {marker}"
        if runs > 1:
            line += f"  {min(all_speedups):>5.2f}-{max(all_speedups):.2f}x"
        print(line)

    print("=" * 75)
    wins = sum(1 for r in results if r[1] < r[2])
    print(f"BMB faster in {wins}/{len(results)} benchmarks (median).")
    print("Note: BMB includes ctypes FFI overhead. 50-500-iter mean per sample, 10-iter warmup.")
    print("Run `python bench_algo.py --runs=5` for median-of-5 with min-max spread.")
    print("Run `python bench_algo.py --scaling` to reproduce the README scaling table.")

    if '--scaling' in sys.argv:
        print()
        scaling_sweep(runs=runs)
