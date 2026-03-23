#!/usr/bin/env python3
"""Benchmark: bmb-crypto vs Python hashlib/base64/binascii (C-backed stdlib)"""
import sys, os, time
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_crypto as bmb
import hashlib, base64 as b64lib, binascii

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
# Two payload sizes: short and medium
SHORT = "hello world"
MEDIUM = "The quick brown fox jumps over the lazy dog" * 10   # 440 bytes
B64_ENCODED = b64lib.b64encode(MEDIUM.encode()).decode()

if __name__ == '__main__':
    print("=" * 70)
    print(f"{'Benchmark: bmb-crypto vs Python C-backed stdlib':^70}")
    print("=" * 70)
    print(f"{'Function':<26} {'BMB (µs)':>10} {'Python (µs)':>12} {'Speedup':>9}")
    print("-" * 70)

    results = [
        # SHA-256
        run("sha256 (short)",
            lambda: bmb.sha256(SHORT),
            lambda: hashlib.sha256(SHORT.encode()).hexdigest()),
        run("sha256 (440B)",
            lambda: bmb.sha256(MEDIUM),
            lambda: hashlib.sha256(MEDIUM.encode()).hexdigest()),
        # MD5
        run("md5 (short)",
            lambda: bmb.md5(SHORT),
            lambda: hashlib.md5(SHORT.encode()).hexdigest()),
        run("md5 (440B)",
            lambda: bmb.md5(MEDIUM),
            lambda: hashlib.md5(MEDIUM.encode()).hexdigest()),
        # Base64 encode
        run("base64_encode (short)",
            lambda: bmb.base64_encode(SHORT),
            lambda: b64lib.b64encode(SHORT.encode()).decode()),
        run("base64_encode (440B)",
            lambda: bmb.base64_encode(MEDIUM),
            lambda: b64lib.b64encode(MEDIUM.encode()).decode()),
        # Base64 decode
        run("base64_decode",
            lambda: bmb.base64_decode(B64_ENCODED),
            lambda: b64lib.b64decode(B64_ENCODED).decode()),
        # CRC32
        run("crc32 (short)",
            lambda: bmb.crc32(SHORT),
            lambda: format(binascii.crc32(SHORT.encode()) & 0xffffffff, '08x')),
        run("crc32 (440B)",
            lambda: bmb.crc32(MEDIUM),
            lambda: format(binascii.crc32(MEDIUM.encode()) & 0xffffffff, '08x')),
    ]

    for name, bmb_us, py_us, speedup in results:
        marker = "FAST" if speedup >= 2.0 else ("OK" if speedup >= 1.0 else "SLOW")
        print(f"{name:<26} {bmb_us:>10.2f} {py_us:>12.2f} {speedup:>8.2f}x {marker}")

    print("=" * 70)
    wins = sum(1 for _, b, p, _ in results if b < p)
    print(f"BMB faster in {wins}/{len(results)} benchmarks")
    print("Note: Python baseline uses C-backed hashlib/base64/binascii.")
    print("      BMB timings include ctypes FFI + UTF-8 encode overhead.")
