#!/usr/bin/env python3
"""
Build all BMB binding libraries (.dll/.so/.dylib).

Usage:
    python build_all.py              # Build all 5 libraries
    python build_all.py bmb-algo     # Build one library
    python build_all.py --release    # Build with optimization (default)
    python build_all.py --debug      # Build without optimization
    python build_all.py --test       # Build + run tests
"""

import subprocess
import sys
import os
import shutil
import time
import argparse

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.dirname(SCRIPT_DIR)

# BMB compiler path
BMB_COMPILER = os.path.join(ROOT_DIR, "target", "release", "bmb.exe")
if not os.path.exists(BMB_COMPILER):
    BMB_COMPILER = os.path.join(ROOT_DIR, "target", "release", "bmb")

LIBRARIES = {
    "bmb-algo": {
        "src": "bmb-algo/src/lib.bmb",
        "output": "bmb_algo",
        "description": "41 algorithms (DP, Graph, Sort, Search, NumberTheory, Bit, Array)",
    },
    "bmb-compute": {
        "src": "bmb-compute/src/lib.bmb",
        "output": "bmb_compute",
        "description": "25 functions (Math, Statistics, Random, Vector, Utility)",
    },
    "bmb-crypto": {
        "src": "bmb-crypto/src/lib.bmb",
        "output": "bmb_crypto",
        "description": "11 functions (SHA-256, MD5, CRC32, HMAC, Base64/32, Adler32)",
    },
    "bmb-text": {
        "src": "bmb-text/src/lib.bmb",
        "output": "bmb_text",
        "description": "20 functions (KMP, find, replace, case, trim, repeat)",
    },
    "bmb-json": {
        "src": "bmb-json/src/lib.bmb",
        "output": "bmb_json",
        "description": "8 functions (validate, stringify, get, array)",
    },
}

# Platform-specific shared library extension
if sys.platform == "win32":
    LIB_EXT = ".dll"
elif sys.platform == "darwin":
    LIB_EXT = ".dylib"
else:
    LIB_EXT = ".so"
    LIB_PREFIX = "lib"


def build_library(name, config, release=True, verbose=False):
    """Build a single BMB library as a shared library."""
    src_path = os.path.join(SCRIPT_DIR, config["src"])
    lib_name = config["output"]

    if sys.platform == "win32":
        output_name = f"{lib_name}.dll"
    elif sys.platform == "darwin":
        output_name = f"lib{lib_name}.dylib"
    else:
        output_name = f"lib{lib_name}.so"

    # Build into the library's root directory
    lib_dir = os.path.join(SCRIPT_DIR, name)
    output_path = os.path.join(lib_dir, output_name)

    cmd = [BMB_COMPILER, "build", src_path, "--shared", "-o", output_path]
    if release:
        cmd.append("--release")

    if verbose:
        print(f"  CMD: {' '.join(cmd)}")

    t0 = time.perf_counter()
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT_DIR)
    elapsed = time.perf_counter() - t0

    if result.returncode != 0:
        print(f"  FAIL ({elapsed:.1f}s)")
        if result.stderr:
            print(f"  stderr: {result.stderr[:500]}")
        if result.stdout:
            print(f"  stdout: {result.stdout[:500]}")
        return False

    # Copy to bindings/python/ directory
    binding_dir = os.path.join(lib_dir, "bindings", "python")
    if os.path.isdir(binding_dir):
        shutil.copy2(output_path, os.path.join(binding_dir, output_name))

    size_kb = os.path.getsize(output_path) / 1024
    print(f"  OK ({elapsed:.1f}s, {size_kb:.0f} KB)")
    return True


def run_tests(name, verbose=False):
    """Run pytest for a library."""
    test_dir = os.path.join(SCRIPT_DIR, name, "tests")
    if not os.path.isdir(test_dir):
        print(f"  No tests found")
        return True

    cmd = [sys.executable, "-m", "pytest", test_dir, "-q"]
    if verbose:
        cmd.append("-v")

    result = subprocess.run(cmd, capture_output=True, text=True)
    # Extract summary line
    lines = result.stdout.strip().split("\n")
    summary = lines[-1] if lines else "no output"
    if result.returncode == 0:
        print(f"  Tests: {summary}")
        return True
    else:
        print(f"  Tests FAILED: {summary}")
        if verbose:
            print(result.stdout)
        return False


def main():
    parser = argparse.ArgumentParser(description="Build BMB binding libraries")
    parser.add_argument("libraries", nargs="*", help="Libraries to build (default: all)")
    parser.add_argument("--debug", action="store_true", help="Build without optimization")
    parser.add_argument("--release", action="store_true", default=True, help="Build with optimization (default)")
    parser.add_argument("--test", action="store_true", help="Run tests after building")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    args = parser.parse_args()

    if not os.path.exists(BMB_COMPILER):
        print(f"ERROR: BMB compiler not found at {BMB_COMPILER}")
        print("Run: cargo build --release --features llvm --target x86_64-pc-windows-gnu")
        sys.exit(1)

    targets = args.libraries if args.libraries else list(LIBRARIES.keys())
    release = not args.debug

    print("=" * 60)
    print("BMB Binding Libraries — Build")
    print(f"Compiler: {BMB_COMPILER}")
    print(f"Mode: {'release' if release else 'debug'}")
    print(f"Platform: {sys.platform}")
    print("=" * 60)

    results = {}
    t_total = time.perf_counter()

    for name in targets:
        if name not in LIBRARIES:
            print(f"\nUnknown library: {name}")
            print(f"Available: {', '.join(LIBRARIES.keys())}")
            sys.exit(1)

        config = LIBRARIES[name]
        print(f"\n[{name}] {config['description']}")
        ok = build_library(name, config, release=release, verbose=args.verbose)
        results[name] = ok

        if ok and args.test:
            test_ok = run_tests(name, verbose=args.verbose)
            results[name] = test_ok

    elapsed = time.perf_counter() - t_total
    print("\n" + "=" * 60)
    passed = sum(1 for v in results.values() if v)
    total = len(results)
    print(f"Built: {passed}/{total} libraries ({elapsed:.1f}s)")

    if passed < total:
        failed = [k for k, v in results.items() if not v]
        print(f"Failed: {', '.join(failed)}")
        sys.exit(1)
    else:
        print("All libraries built successfully!")


if __name__ == "__main__":
    main()
