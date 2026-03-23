#!/usr/bin/env python3
"""Benchmark: bmb-text vs Python str built-ins"""
import sys, os, time
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_text as bmb

# ── Pure Python baselines ──────────────────────────────────────────────────

def py_kmp_search(text, pattern):
    """KMP via Python str.find (C-backed)."""
    return text.find(pattern)

def py_is_palindrome(s):
    return s == s[::-1]

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
HAYSTACK   = "The quick brown fox jumps over the lazy dog " * 20   # ~880 chars
NEEDLE     = "lazy dog"
REPLACE_IN = "aababcabcdabcde" * 50
PALINDROME = "racecar" * 10 + "x" + "racecar" * 10   # long non-palindrome
UPPER_STR  = "hello world, this is bmb-text benchmarking suite!" * 5

if __name__ == '__main__':
    print("=" * 68)
    print(f"{'Benchmark: bmb-text vs Python str built-ins':^68}")
    print("=" * 68)
    print(f"{'Function':<26} {'BMB (µs)':>10} {'Python (µs)':>12} {'Speedup':>9}")
    print("-" * 68)

    results = [
        # KMP vs str.find
        run("kmp_search",
            lambda: bmb.kmp_search(HAYSTACK, NEEDLE),
            lambda: HAYSTACK.find(NEEDLE)),
        # str_find vs str.find
        run("str_find",
            lambda: bmb.str_find(HAYSTACK, NEEDLE),
            lambda: HAYSTACK.find(NEEDLE)),
        # str_replace vs str.replace (first occurrence)
        run("str_replace (first)",
            lambda: bmb.str_replace(REPLACE_IN, "abc", "X"),
            lambda: REPLACE_IN.replace("abc", "X", 1)),
        # str_replace_all vs str.replace
        run("str_replace_all",
            lambda: bmb.str_replace_all(REPLACE_IN, "abc", "X"),
            lambda: REPLACE_IN.replace("abc", "X")),
        # to_upper vs str.upper
        run("to_upper",
            lambda: bmb.to_upper(UPPER_STR),
            lambda: UPPER_STR.upper()),
        # to_lower vs str.lower
        run("to_lower",
            lambda: bmb.to_lower(UPPER_STR.upper()),
            lambda: UPPER_STR.upper().lower()),
        # is_palindrome
        run("is_palindrome (long)",
            lambda: bmb.is_palindrome(PALINDROME),
            lambda: py_is_palindrome(PALINDROME)),
        # str_count vs str.count
        run("str_count",
            lambda: bmb.str_count(HAYSTACK, "the"),
            lambda: HAYSTACK.count("the")),
        # str_reverse vs slicing
        run("str_reverse",
            lambda: bmb.str_reverse(UPPER_STR),
            lambda: UPPER_STR[::-1]),
    ]

    for name, bmb_us, py_us, speedup in results:
        marker = "FAST" if speedup >= 2.0 else ("OK" if speedup >= 1.0 else "SLOW")
        print(f"{name:<26} {bmb_us:>10.2f} {py_us:>12.2f} {speedup:>8.2f}x {marker}")

    print("=" * 68)
    wins = sum(1 for _, b, p, _ in results if b < p)
    print(f"BMB faster in {wins}/{len(results)} benchmarks")
    print("Note: Python baselines use C-backed str methods.")
    print("      BMB timings include ctypes FFI + UTF-8 encode overhead.")
