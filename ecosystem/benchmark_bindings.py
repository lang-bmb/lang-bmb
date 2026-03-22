#!/usr/bin/env python3
"""
Competitive benchmark: BMB binding libraries vs Python stdlib / popular packages.

Measures:
  bmb-algo vs pure Python implementations
  bmb-crypto vs hashlib/binascii
  bmb-text vs Python str methods
  bmb-json vs json module

Usage: python benchmark_bindings.py
"""

import sys
import os
import time
import json
import hashlib
import binascii

# Add MSYS2 DLL directory on Windows
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

# Import BMB libraries
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-algo', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-crypto', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-text', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-json', 'bindings', 'python'))

import bmb_algo
import bmb_crypto
import bmb_text
import bmb_json


def bench(name, fn, n=1000):
    """Run function n times, return avg microseconds."""
    # Warmup
    for _ in range(min(n, 10)):
        fn()
    t0 = time.perf_counter()
    for _ in range(n):
        fn()
    elapsed = time.perf_counter() - t0
    us = elapsed / n * 1_000_000
    return us


def fmt_ratio(bmb_us, py_us):
    if bmb_us == 0:
        return "inf"
    ratio = py_us / bmb_us
    if ratio > 1:
        return f"\033[32m{ratio:.1f}x faster\033[0m"  # green
    elif ratio > 0.5:
        return f"\033[33m{ratio:.1f}x\033[0m"  # yellow
    else:
        return f"\033[31m{ratio:.1f}x slower\033[0m"  # red


print("=" * 70)
print("BMB Binding Libraries — Competitive Benchmark")
print("=" * 70)
print()

# ============================================================================
# bmb-algo benchmarks
# ============================================================================
print("[bmb-algo vs Pure Python]")
print()

# Knapsack (large)
N_KS = 100
weights = list(range(1, N_KS + 1))
values = list(range(1, N_KS + 1))
capacity = N_KS * 3

def py_knapsack(w, v, cap):
    n = len(w)
    dp = [0] * (cap + 1)
    for i in range(n):
        for j in range(cap, w[i] - 1, -1):
            dp[j] = max(dp[j], dp[j - w[i]] + v[i])
    return dp[cap]

bmb_ks = bench("bmb_knapsack", lambda: bmb_algo.knapsack(weights, values, capacity), n=100)
py_ks = bench("py_knapsack", lambda: py_knapsack(weights, values, capacity), n=100)
print(f"  knapsack({N_KS} items, cap={capacity})")
print(f"    BMB: {bmb_ks:.0f} us, Python: {py_ks:.0f} us ->{fmt_ratio(bmb_ks, py_ks)}")

# Fibonacci (large)
def py_fib(n):
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b

bmb_fib = bench("bmb_fibonacci", lambda: bmb_algo.fibonacci(50), n=10000)
py_fib_t = bench("py_fibonacci", lambda: py_fib(50), n=10000)
print(f"  fibonacci(50)")
print(f"    BMB: {bmb_fib:.1f} us, Python: {py_fib_t:.1f} us ->{fmt_ratio(bmb_fib, py_fib_t)}")

# Prime count
def py_prime_count(n):
    sieve = [True] * (n + 1)
    sieve[0] = sieve[1] = False
    p = 2
    while p * p <= n:
        if sieve[p]:
            for j in range(p * p, n + 1, p):
                sieve[j] = False
        p += 1
    return sum(sieve)

bmb_pc = bench("bmb_prime_count", lambda: bmb_algo.prime_count(10000), n=100)
py_pc = bench("py_prime_count", lambda: py_prime_count(10000), n=100)
print(f"  prime_count(10000)")
print(f"    BMB: {bmb_pc:.0f} us, Python: {py_pc:.0f} us ->{fmt_ratio(bmb_pc, py_pc)}")

# NQueens
def py_nqueens(n):
    def safe(cols, row, col):
        return all(cols[r] != col and abs(cols[r] - col) != row - r for r in range(row))
    def solve(cols, row):
        if row == n: return 1
        return sum(solve(cols + [c], row + 1) for c in range(n) if safe(cols, row, c))
    return solve([], 0)

bmb_nq = bench("bmb_nqueens", lambda: bmb_algo.nqueens(8), n=10)
py_nq = bench("py_nqueens", lambda: py_nqueens(8), n=10)
print(f"  nqueens(8)")
print(f"    BMB: {bmb_nq:.0f} us, Python: {py_nq:.0f} us ->{fmt_ratio(bmb_nq, py_nq)}")

# Sort
import random
sort_data = [random.randint(0, 10000) for _ in range(1000)]

bmb_ms = bench("bmb_merge_sort", lambda: bmb_algo.merge_sort(sort_data), n=500)
py_ms = bench("py_sorted", lambda: sorted(sort_data), n=500)
print(f"  merge_sort(1000 elements)")
print(f"    BMB: {bmb_ms:.0f} us, Python sorted(): {py_ms:.0f} us ->{fmt_ratio(bmb_ms, py_ms)}")

print()

# ============================================================================
# bmb-crypto benchmarks
# ============================================================================
print("[bmb-crypto vs Python hashlib]")
print()

data_small = "hello world"
data_large = "The quick brown fox jumps over the lazy dog. " * 100

# SHA-256
bmb_sha_s = bench("bmb_sha256_small", lambda: bmb_crypto.sha256(data_small), n=5000)
py_sha_s = bench("py_sha256_small", lambda: hashlib.sha256(data_small.encode()).hexdigest(), n=5000)
print(f"  sha256({len(data_small)}B)")
print(f"    BMB: {bmb_sha_s:.1f} us, hashlib: {py_sha_s:.1f} us ->{fmt_ratio(bmb_sha_s, py_sha_s)}")

bmb_sha_l = bench("bmb_sha256_large", lambda: bmb_crypto.sha256(data_large), n=1000)
py_sha_l = bench("py_sha256_large", lambda: hashlib.sha256(data_large.encode()).hexdigest(), n=1000)
print(f"  sha256({len(data_large)}B)")
print(f"    BMB: {bmb_sha_l:.1f} us, hashlib: {py_sha_l:.1f} us ->{fmt_ratio(bmb_sha_l, py_sha_l)}")

# MD5
bmb_md5 = bench("bmb_md5", lambda: bmb_crypto.md5(data_large), n=1000)
py_md5 = bench("py_md5", lambda: hashlib.md5(data_large.encode()).hexdigest(), n=1000)
print(f"  md5({len(data_large)}B)")
print(f"    BMB: {bmb_md5:.1f} us, hashlib: {py_md5:.1f} us ->{fmt_ratio(bmb_md5, py_md5)}")

# CRC32
bmb_crc = bench("bmb_crc32", lambda: bmb_crypto.crc32(data_large), n=1000)
py_crc = bench("py_crc32", lambda: format(binascii.crc32(data_large.encode()) & 0xffffffff, '08x'), n=1000)
print(f"  crc32({len(data_large)}B)")
print(f"    BMB: {bmb_crc:.1f} us, binascii: {py_crc:.1f} us ->{fmt_ratio(bmb_crc, py_crc)}")

print()

# ============================================================================
# bmb-text benchmarks
# ============================================================================
print("[bmb-text vs Python str methods]")
print()

haystack = "The quick brown fox jumps over the lazy dog. " * 50
needle = "lazy dog"

bmb_find = bench("bmb_str_find", lambda: bmb_text.str_find(haystack, needle), n=2000)
py_find = bench("py_str_find", lambda: haystack.find(needle), n=2000)
print(f"  str_find({len(haystack)}B, '{needle}')")
print(f"    BMB: {bmb_find:.1f} us, Python .find(): {py_find:.1f} us ->{fmt_ratio(bmb_find, py_find)}")

bmb_cnt = bench("bmb_str_count", lambda: bmb_text.str_count(haystack, "the"), n=2000)
py_cnt = bench("py_str_count", lambda: haystack.count("the"), n=2000)
print(f"  str_count({len(haystack)}B, 'the')")
print(f"    BMB: {bmb_cnt:.1f} us, Python .count(): {py_cnt:.1f} us ->{fmt_ratio(bmb_cnt, py_cnt)}")

bmb_rep = bench("bmb_replace_all", lambda: bmb_text.str_replace_all(haystack, "the", "THE"), n=1000)
py_rep = bench("py_replace", lambda: haystack.replace("the", "THE"), n=1000)
print(f"  str_replace_all({len(haystack)}B)")
print(f"    BMB: {bmb_rep:.1f} us, Python .replace(): {py_rep:.1f} us ->{fmt_ratio(bmb_rep, py_rep)}")

print()

# ============================================================================
# bmb-json benchmarks
# ============================================================================
print("[bmb-json vs Python json]")
print()

json_small = '{"name":"BMB","version":97,"tags":["fast","safe"]}'
json_large = json.dumps({"items": [{"id": i, "name": f"item_{i}", "value": i * 10} for i in range(100)]})

bmb_js = bench("bmb_json_small", lambda: bmb_json.stringify(json_small), n=5000)
py_js = bench("py_json_small", lambda: json.dumps(json.loads(json_small), separators=(',', ':')), n=5000)
print(f"  roundtrip({len(json_small)}B)")
print(f"    BMB: {bmb_js:.1f} us, json: {py_js:.1f} us ->{fmt_ratio(bmb_js, py_js)}")

bmb_jl = bench("bmb_json_large", lambda: bmb_json.stringify(json_large), n=500)
py_jl = bench("py_json_large", lambda: json.dumps(json.loads(json_large), separators=(',', ':')), n=500)
print(f"  roundtrip({len(json_large)}B)")
print(f"    BMB: {bmb_jl:.1f} us, json: {py_jl:.1f} us ->{fmt_ratio(bmb_jl, py_jl)}")

bmb_get = bench("bmb_json_get", lambda: bmb_json.get_string(json_small, "name"), n=5000)
py_get = bench("py_json_get", lambda: json.loads(json_small)["name"], n=5000)
print(f"  get_string({len(json_small)}B, 'name')")
print(f"    BMB: {bmb_get:.1f} us, json.loads[]: {py_get:.1f} us ->{fmt_ratio(bmb_get, py_get)}")

print()
print("=" * 70)
print("Note: BMB FFI overhead (ctypes marshalling) is significant for small inputs.")
print("Raw BMB computation is competitive with C; FFI adds ~2-5us per call.")
print("For batch processing or large data, BMB's advantage increases.")
