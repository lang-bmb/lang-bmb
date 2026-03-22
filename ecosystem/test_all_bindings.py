#!/usr/bin/env python3
"""
Comprehensive test suite for all BMB binding libraries.
Tests bmb-algo, bmb-crypto, and bmb-text Python bindings.

Usage: python test_all_bindings.py
"""

import sys
import os
import time

# Add MSYS2 DLL directory on Windows
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

passed = 0
failed = 0
errors = []


def check(suite, name, got, expected):
    global passed, failed, errors
    if got == expected:
        passed += 1
    else:
        failed += 1
        errors.append(f"  [{suite}] {name}: got {repr(got)}, expected {repr(expected)}")


def test_bmb_algo():
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-algo', 'bindings', 'python'))
    import bmb_algo

    S = "bmb-algo"
    # DP
    check(S, "knapsack", bmb_algo.knapsack([2,3,4], [3,4,5], 7), 9)
    check(S, "max_subarray", bmb_algo.max_subarray([-2,1,-3,4,-1,2,1,-5,4]), 6)
    check(S, "coin_change", bmb_algo.coin_change([1,5,11], 15), 3)
    check(S, "lis", bmb_algo.lis([10,9,2,5,3,7,101,18]), 4)
    check(S, "edit_distance", bmb_algo.edit_distance("kitten", "sitting"), 3)
    check(S, "lcs", bmb_algo.lcs("ABCBDAB", "BDCAB"), 4)
    # Graph
    check(S, "dijkstra", bmb_algo.dijkstra([[0,4,-1],[-1,0,2],[-1,-1,0]], 0), [0, 4, 6])
    check(S, "bfs_count", bmb_algo.bfs_count([[0,1,0],[0,0,1],[0,0,0]], 0), 3)
    # Sort
    check(S, "quicksort", bmb_algo.quicksort([3,1,4,1,5]), [1,1,3,4,5])
    check(S, "merge_sort", bmb_algo.merge_sort([5,3,1,4,2]), [1,2,3,4,5])
    check(S, "heap_sort", bmb_algo.heap_sort([5,3,1,4,2]), [1,2,3,4,5])
    check(S, "counting_sort", bmb_algo.counting_sort([3,1,4,1,5,9,2,6]), [1,1,2,3,4,5,6,9])
    # Search
    check(S, "binary_search_found", bmb_algo.binary_search([10,20,30,40,50], 30), 2)
    check(S, "binary_search_miss", bmb_algo.binary_search([10,20,30,40,50], 35), -1)
    # Graph (topo)
    check(S, "topological_sort", bmb_algo.topological_sort([[0,1,1,0],[0,0,0,1],[0,0,0,1],[0,0,0,0]]), [0,1,2,3])
    # Number theory
    check(S, "gcd", bmb_algo.gcd(12, 8), 4)
    check(S, "fibonacci", bmb_algo.fibonacci(10), 55)
    check(S, "prime_count", bmb_algo.prime_count(100), 25)
    # Matrix
    check(S, "matrix_multiply", bmb_algo.matrix_multiply([[1,2],[3,4]], [[5,6],[7,8]]), [[19,22],[43,50]])


def test_bmb_crypto():
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-crypto', 'bindings', 'python'))
    import bmb_crypto
    import hashlib
    import binascii

    S = "bmb-crypto"
    # SHA-256
    check(S, "sha256('')", bmb_crypto.sha256(""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    check(S, "sha256('hello')", bmb_crypto.sha256("hello"), "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
    # MD5
    for inp in ["", "hello", "abc"]:
        check(S, f"md5({repr(inp)})", bmb_crypto.md5(inp), hashlib.md5(inp.encode()).hexdigest())
    # CRC32
    for inp in ["", "hello", "123456789"]:
        check(S, f"crc32({repr(inp)})", bmb_crypto.crc32(inp), format(binascii.crc32(inp.encode()) & 0xffffffff, '08x'))
    # HMAC-SHA256
    import hmac
    for key, msg in [("key", "The quick brown fox jumps over the lazy dog"), ("", ""), ("abc", "abc")]:
        expected = hmac.new(key.encode(), msg.encode(), hashlib.sha256).hexdigest()
        check(S, f"hmac({repr(key)[:8]})", bmb_crypto.hmac_sha256(key, msg), expected)
    # Base64
    for inp, enc in [("f", "Zg=="), ("foo", "Zm9v"), ("hello", "aGVsbG8=")]:
        check(S, f"b64_encode({repr(inp)})", bmb_crypto.base64_encode(inp), enc)
        check(S, f"b64_decode({repr(enc)})", bmb_crypto.base64_decode(enc), inp)
    # Base32
    for inp, enc in [("f", "MY======"), ("foo", "MZXW6==="), ("fooba", "MZXW6YTB")]:
        check(S, f"b32_encode({repr(inp)})", bmb_crypto.base32_encode(inp), enc)
        check(S, f"b32_decode({repr(enc)})", bmb_crypto.base32_decode(enc), inp)


def test_bmb_text():
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-text', 'bindings', 'python'))
    import bmb_text

    S = "bmb-text"
    # KMP
    check(S, "kmp_found", bmb_text.kmp_search("hello world", "world"), 6)
    check(S, "kmp_miss", bmb_text.kmp_search("hello world", "xyz"), -1)
    # Find/rfind/count
    check(S, "find", bmb_text.str_find("abcabc", "bc"), 1)
    check(S, "rfind", bmb_text.str_rfind("abcabc", "bc"), 4)
    check(S, "count", bmb_text.str_count("abcabcabc", "abc"), 3)
    # Contains/starts/ends
    check(S, "contains_true", bmb_text.str_contains("hello world", "world"), True)
    check(S, "contains_false", bmb_text.str_contains("hello world", "xyz"), False)
    check(S, "starts_with", bmb_text.str_starts_with("hello", "hel"), True)
    check(S, "ends_with", bmb_text.str_ends_with("hello", "llo"), True)
    # Palindrome
    check(S, "palindrome_yes", bmb_text.is_palindrome("racecar"), True)
    check(S, "palindrome_no", bmb_text.is_palindrome("hello"), False)
    # Byte ops
    check(S, "find_byte", bmb_text.find_byte("hello", ord('l')), 2)
    check(S, "count_byte", bmb_text.count_byte("hello", ord('l')), 2)
    # Token
    check(S, "token_count", bmb_text.token_count("a,b,c,d", ","), 4)


if __name__ == '__main__':
    print("=" * 60)
    print("BMB Binding Libraries — Comprehensive Test Suite")
    print("=" * 60)
    print()

    t0 = time.perf_counter()

    print("[bmb-algo: 19 algorithms]")
    test_bmb_algo()
    print(f"  {passed} passed")

    algo_passed = passed
    print()
    print("[bmb-crypto: 8 functions]")
    test_bmb_crypto()
    print(f"  {passed - algo_passed} passed")

    crypto_passed = passed
    print()
    print("[bmb-text: 11 functions]")
    test_bmb_text()
    print(f"  {passed - crypto_passed} passed")

    elapsed = time.perf_counter() - t0
    print()
    print("=" * 60)
    total = passed + failed
    print(f"Total: {passed}/{total} passed ({elapsed:.2f}s)")

    if errors:
        print()
        print("FAILURES:")
        for e in errors:
            print(e)
        sys.exit(1)
    else:
        print()
        print("ALL TESTS PASSED!")
        print()
        print("Libraries:")
        print("  bmb-algo  — 19 algorithms (DP, Graph, Sort, Search, Number Theory)")
        print("  bmb-crypto — 8 functions (SHA-256, MD5, CRC32, HMAC, Base64, Base32)")
        print("  bmb-text  — 11 functions (KMP, find, contains, palindrome, tokenize)")
        print()
        print("Powered by BMB — https://github.com/iyulab/lang-bmb")
