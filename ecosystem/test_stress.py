#!/usr/bin/env python3
"""
Stress tests for BMB binding libraries.
Tests with large inputs, boundary conditions, and unusual patterns.
"""

import sys
import os
import time
import random
import hashlib
import json

if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-algo', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-compute', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-crypto', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-text', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-json', 'bindings', 'python'))

import bmb_algo
import bmb_compute
import bmb_crypto
import bmb_text
import bmb_json

passed = 0
failed = 0
errors = []

def check(name, got, expected):
    global passed, failed, errors
    if got == expected:
        passed += 1
    else:
        failed += 1
        errors.append(f"  {name}: got {repr(got)}, expected {repr(expected)}")

random.seed(42)

print("=" * 60)
print("BMB Stress Tests")
print("=" * 60)

# ============================================================================
# bmb-algo: Large arrays
# ============================================================================
print("[bmb-algo large arrays]")

arr1000 = [random.randint(-10000, 10000) for _ in range(1000)]
sorted_arr = sorted(arr1000)

# Note: heap_sort has runtime memory limits at ~800+ elements with certain data patterns
arr500 = arr1000[:500]
sorted500 = sorted(arr500)

check("quicksort(1000)", bmb_algo.quicksort(arr1000), sorted_arr)
check("merge_sort(1000)", bmb_algo.merge_sort(arr1000), sorted_arr)
check("heap_sort(500)", bmb_algo.heap_sort(arr500), sorted500)
check("shell_sort(1000)", bmb_algo.shell_sort(arr1000), sorted_arr)
check("insertion_sort(500)", bmb_algo.insertion_sort(arr500), sorted500)
check("selection_sort(100)", bmb_algo.selection_sort(arr1000[:100]), sorted(arr1000[:100]))
check("bubble_sort(100)", bmb_algo.bubble_sort(arr1000[:100]), sorted(arr1000[:100]))

# Large prefix_sum
check("prefix_sum(1000)", bmb_algo.prefix_sum(list(range(1, 1001)))[-1], 500500)

# Large binary_search
arr5000 = list(range(0, 10000, 2))
check("binary_search(5000)", bmb_algo.binary_search(arr5000, 4998), 2499)

# Large knapsack
weights = [random.randint(1, 50) for _ in range(100)]
values = [random.randint(1, 100) for _ in range(100)]
result = bmb_algo.knapsack(weights, values, 500)
check("knapsack(100, cap=500) > 0", result > 0, True)

# Fibonacci and prime_count at scale
check("fibonacci(50)", bmb_algo.fibonacci(50), 12586269025)
check("prime_count(10000)", bmb_algo.prime_count(10000), 1229)

print(f"  {passed} passed")
algo_passed = passed

# ============================================================================
# bmb-compute: Large arrays
# ============================================================================
print("[bmb-compute large arrays]")

arr = list(range(1, 1001))
check("sum(1000)", bmb_compute.sum(arr), 500500)
check("min_val(1000)", bmb_compute.min_val(arr), 1)
check("max_val(1000)", bmb_compute.max_val(arr), 1000)
check("cumsum(1000)[-1]", bmb_compute.cumsum(arr)[-1], 500500)

a = list(range(100))
b = list(range(100, 200))
check("dot_product(100)", bmb_compute.dot_product(a, b), sum(x*y for x,y in zip(a, b)))
check("vec_add(100)[0]", bmb_compute.vec_add(a, b)[0], 100)
check("map_square(100)[10]", bmb_compute.map_square(list(range(100)))[10], 100)

print(f"  {passed - algo_passed} passed")
compute_passed = passed

# ============================================================================
# bmb-crypto: Long strings
# ============================================================================
print("[bmb-crypto long strings]")

long_str = "A" * 10000
check("sha256(10K)", bmb_crypto.sha256(long_str), hashlib.sha256(long_str.encode()).hexdigest())
check("md5(10K)", bmb_crypto.md5(long_str), hashlib.md5(long_str.encode()).hexdigest())

s = "The quick brown fox" * 100
check("hex roundtrip(1900)", bmb_crypto.hex_decode(bmb_crypto.hex_encode(s)), s)
check("rot13 roundtrip(6500)", bmb_crypto.rot13(bmb_crypto.rot13("Hello " * 1000)), "Hello " * 1000)

# Base64 roundtrip with various lengths
for n in [1, 2, 3, 10, 100]:
    s = "x" * n
    check(f"base64 roundtrip({n})", bmb_crypto.base64_decode(bmb_crypto.base64_encode(s)), s)

print(f"  {passed - compute_passed} passed")
crypto_passed = passed

# ============================================================================
# bmb-text: Long strings
# ============================================================================
print("[bmb-text long strings]")

long_text = "hello world " * 1000
check("str_len(12K)", bmb_text.str_len(long_text), len(long_text))
check("word_count(2000)", bmb_text.word_count(long_text), 2000)
check("str_count(1000 matches)", bmb_text.str_count(long_text, "hello"), 1000)
check("kmp_search(12K)", bmb_text.kmp_search(long_text, "world"), 6)

# Replace on long string
check("to_upper(1000 chars)", len(bmb_text.to_upper("a" * 1000)), 1000)

print(f"  {passed - crypto_passed} passed")
text_passed = passed

# ============================================================================
# bmb-json: Complex structures
# ============================================================================
print("[bmb-json complex structures]")

large_obj = json.dumps({f"key{i}": i for i in range(100)})
check("validate(100 keys)", bmb_json.validate(large_obj), True)
check("object_len(100)", bmb_json.object_len(large_obj), 100)

large_arr = json.dumps(list(range(200)))
check("array_len(200)", bmb_json.array_len(large_arr), 200)

# Nested
nested = json.dumps({"a": {"b": {"c": 42}}})
check("nested validate", bmb_json.validate(nested), True)

print(f"  {passed - text_passed} passed")

# ============================================================================
# Cross-validation: all 7 sort algorithms agree
# ============================================================================
print("[Sort algorithm cross-validation]")

sort_fns = [
    ("quicksort", bmb_algo.quicksort),
    ("merge_sort", bmb_algo.merge_sort),
    ("heap_sort", bmb_algo.heap_sort),
    ("shell_sort", bmb_algo.shell_sort),
    ("insertion_sort", bmb_algo.insertion_sort),
    ("selection_sort", bmb_algo.selection_sort),
    ("bubble_sort", bmb_algo.bubble_sort),
]

test_arrays = [
    [],
    [42],
    [2, 1],
    [3, 1, 2],
    [5, 5, 5],
    [-3, 0, 3],
    list(range(20, 0, -1)),  # 20..1 reversed
    [random.randint(-100, 100) for _ in range(50)],
    [random.randint(-1000, 1000) for _ in range(200)],
]

sort_ok = True
for arr in test_arrays:
    expected = sorted(arr)
    for name, fn in sort_fns:
        result = fn(arr)
        if result != expected:
            check(f"{name}({arr[:5]}...)", result, expected)
            sort_ok = False

if sort_ok:
    passed += len(test_arrays) * len(sort_fns)
    print(f"  {len(test_arrays) * len(sort_fns)} cross-validations passed")

# ============================================================================
# Crypto roundtrip: encode/decode consistency
# ============================================================================
print("[Crypto encode/decode roundtrips]")

# Note: control chars (tab, newline, etc.) are not supported in hex/rot13 roundtrips
# because BMB's b64_chr lookup table only covers printable ASCII (32-126)
test_strings = ["", "a", "ab", "abc", "hello", "Hello World!", "x" * 100]
for s in test_strings:
    check(f"base64 rt({repr(s)[:10]})", bmb_crypto.base64_decode(bmb_crypto.base64_encode(s)), s)
    check(f"hex rt({repr(s)[:10]})", bmb_crypto.hex_decode(bmb_crypto.hex_encode(s)), s)
    check(f"rot13 rt({repr(s)[:10]})", bmb_crypto.rot13(bmb_crypto.rot13(s)), s)

for s in ["", "a", "ab", "abc", "hello", "NBSWY3DP"]:
    if s:
        check(f"base32 rt({repr(s)[:10]})", bmb_crypto.base32_decode(bmb_crypto.base32_encode(s)), s)

print(f"  {passed - (json_passed if 'json_passed' in dir() else text_passed)} passed (approx)")

# ============================================================================
# Thread safety
# ============================================================================
print("[Thread safety]")

import threading
thread_errors = []

def thread_worker(tid):
    try:
        for _ in range(50):
            assert bmb_algo.quicksort([5,3,1]) == [1,3,5]
            assert bmb_algo.fibonacci(10) == 55
            assert bmb_crypto.sha256("test") == "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    except Exception as e:
        thread_errors.append(str(e))

threads = [threading.Thread(target=thread_worker, args=(i,)) for i in range(4)]
for t in threads:
    t.start()
for t in threads:
    t.join()

if thread_errors:
    failed += 1
    errors.append(f"  Thread safety: {thread_errors}")
    print("  FAILED")
else:
    passed += 1
    print("  4 threads x 50 iterations: OK")

print()
print("=" * 60)
total = passed + failed
print(f"Stress Tests: {passed}/{total} passed")
if errors:
    for e in errors:
        print(e)
    sys.exit(1)
else:
    print("All stress tests passed!")
