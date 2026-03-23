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

def run(name, bmb_fn, py_fn, iterations=500):
    bmb_us = bench(bmb_fn, iterations) * 1e6
    py_us  = bench(py_fn,  iterations) * 1e6
    speedup = py_us / bmb_us if bmb_us > 0 else float('inf')
    return name, bmb_us, py_us, speedup

# ── Test data ─────────────────────────────────────────────────────────────

WEIGHTS   = [2, 3, 4, 5, 6, 7, 8, 9, 1, 3]
VALUES    = [3, 4, 5, 6, 7, 8, 9, 10, 2, 4]
CAPACITY  = 20
SORT_DATA = [64, 34, 25, 12, 22, 11, 90, 45, 78, 56, 33, 17, 99, 3, 60]
STR_A, STR_B = "intention", "execution"

if __name__ == '__main__':
    print("=" * 65)
    print(f"{'Benchmark: bmb-algo vs pure Python':^65}")
    print("=" * 65)
    print(f"{'Function':<20} {'BMB (µs)':>10} {'Python (µs)':>12} {'Speedup':>9}")
    print("-" * 65)

    results = [
        run("knapsack",
            lambda: bmb.knapsack(WEIGHTS, VALUES, CAPACITY),
            lambda: py_knapsack(WEIGHTS, VALUES, CAPACITY)),
        run("fibonacci(30)",
            lambda: bmb.fibonacci(30),
            lambda: py_fibonacci(30)),
        run("prime_count(10000)",
            lambda: bmb.prime_count(10000),
            lambda: py_prime_count(10000),
            iterations=100),
        run("nqueens(10)",
            lambda: bmb.nqueens(10),
            lambda: py_nqueens(10),
            iterations=100),
        run("quicksort(15)",
            lambda: bmb.quicksort(SORT_DATA),
            lambda: py_quicksort(SORT_DATA)),
        run("merge_sort(15)",
            lambda: bmb.merge_sort(SORT_DATA),
            lambda: py_merge_sort(SORT_DATA)),
        run("edit_distance",
            lambda: bmb.edit_distance(STR_A, STR_B),
            lambda: py_edit_distance(STR_A, STR_B)),
    ]

    for name, bmb_us, py_us, speedup in results:
        marker = "FAST" if speedup >= 2.0 else ("OK" if speedup >= 1.0 else "SLOW")
        print(f"{name:<20} {bmb_us:>10.2f} {py_us:>12.2f} {speedup:>8.2f}x {marker}")

    print("=" * 65)
    wins = sum(1 for _, b, p, _ in results if b < p)
    print(f"BMB faster in {wins}/{len(results)} benchmarks")
    print("Note: BMB includes ctypes FFI overhead in all timings.")
