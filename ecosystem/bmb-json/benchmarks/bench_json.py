#!/usr/bin/env python3
"""Benchmark: bmb-json vs Python json stdlib"""
import sys, os, time, json as pyjson
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_json as bmb

# ── Python baselines ───────────────────────────────────────────────────────

def py_validate(s):
    try:
        pyjson.loads(s)
        return True
    except Exception:
        return False

def py_stringify(s):
    return pyjson.dumps(pyjson.loads(s), separators=(',', ':'))

def py_get(s, key):
    obj = pyjson.loads(s)
    return pyjson.dumps(obj.get(key, ''), separators=(',', ':'))

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
SIMPLE_OBJ = '{"name":"BMB","version":97,"fast":true}'
NESTED_OBJ = '{"project":"bmb","lang":{"name":"BMB","version":97},"tags":["fast","safe","native"]}'
ARRAY_JSON = '[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]'
DEEP_JSON  = '{"a":{"b":{"c":{"d":{"e":42}}}}}'

if __name__ == '__main__':
    print("=" * 68)
    print(f"{'Benchmark: bmb-json vs Python json stdlib':^68}")
    print("=" * 68)
    print(f"{'Function':<28} {'BMB (µs)':>10} {'Python (µs)':>12} {'Speedup':>9}")
    print("-" * 68)

    results = [
        # validate
        run("validate (simple obj)",
            lambda: bmb.validate(SIMPLE_OBJ),
            lambda: py_validate(SIMPLE_OBJ)),
        run("validate (nested obj)",
            lambda: bmb.validate(NESTED_OBJ),
            lambda: py_validate(NESTED_OBJ)),
        run("validate (array 20)",
            lambda: bmb.validate(ARRAY_JSON),
            lambda: py_validate(ARRAY_JSON)),
        # stringify (parse + re-serialize)
        run("stringify (simple obj)",
            lambda: bmb.stringify(SIMPLE_OBJ),
            lambda: py_stringify(SIMPLE_OBJ)),
        run("stringify (nested obj)",
            lambda: bmb.stringify(NESTED_OBJ),
            lambda: py_stringify(NESTED_OBJ)),
        run("stringify (array 20)",
            lambda: bmb.stringify(ARRAY_JSON),
            lambda: py_stringify(ARRAY_JSON)),
        # get — key lookup
        run("get (top-level key)",
            lambda: bmb.get(SIMPLE_OBJ, "name"),
            lambda: py_get(SIMPLE_OBJ, "name")),
        run("get (nested obj key)",
            lambda: bmb.get(NESTED_OBJ, "tags"),
            lambda: py_get(NESTED_OBJ, "tags")),
        # get_number
        run("get_number",
            lambda: bmb.get_number(SIMPLE_OBJ, "version"),
            lambda: pyjson.loads(SIMPLE_OBJ).get("version", 0)),
        # array_len
        run("array_len",
            lambda: bmb.array_len(ARRAY_JSON),
            lambda: len(pyjson.loads(ARRAY_JSON))),
    ]

    for name, bmb_us, py_us, speedup in results:
        marker = "FAST" if speedup >= 2.0 else ("OK" if speedup >= 1.0 else "SLOW")
        print(f"{name:<28} {bmb_us:>10.2f} {py_us:>12.2f} {speedup:>8.2f}x {marker}")

    print("=" * 68)
    wins = sum(1 for _, b, p, _ in results if b < p)
    print(f"BMB faster in {wins}/{len(results)} benchmarks")
    print("Note: Python baseline uses C-backed json module.")
    print("      BMB timings include ctypes FFI + string allocation overhead.")
