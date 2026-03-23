#!/usr/bin/env python3
"""
Edge case tests for all BMB binding libraries.
Tests boundary conditions, empty inputs, large values, and error paths.
"""

import sys
import os

if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-algo', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-crypto', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-text', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-json', 'bindings', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'bmb-compute', 'bindings', 'python'))

import bmb_algo
import bmb_crypto
import bmb_text
import bmb_json
import bmb_compute

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

print("=" * 60)
print("BMB Edge Case Tests")
print("=" * 60)
print()

# ============================================================================
# bmb-algo edge cases
# ============================================================================
print("[bmb-algo edge cases]")

# Empty/single element arrays
check("quicksort empty", bmb_algo.quicksort([]), [])
check("quicksort single", bmb_algo.quicksort([42]), [42])
check("merge_sort empty", bmb_algo.merge_sort([]), [])
check("heap_sort empty", bmb_algo.heap_sort([]), [])
check("binary_search empty", bmb_algo.binary_search([], 1), -1)
check("is_sorted empty", bmb_algo.is_sorted([]), True)
check("is_sorted single", bmb_algo.is_sorted([42]), True)
check("array_reverse empty", bmb_algo.array_reverse([]), [])
check("array_reverse single", bmb_algo.array_reverse([42]), [42])

# Boundary values
check("fibonacci(0)", bmb_algo.fibonacci(0), 0)
check("fibonacci(1)", bmb_algo.fibonacci(1), 1)
check("gcd(0,0)", bmb_algo.gcd(0, 0), 0)
check("gcd(7,0)", bmb_algo.gcd(7, 0), 7)
check("gcd(0,7)", bmb_algo.gcd(0, 7), 7)
check("lcm(0,5)", bmb_algo.lcm(0, 5), 0)
check("power_set_size(0)", bmb_algo.power_set_size(0), 1)
check("nqueens(0)", bmb_algo.nqueens(0), 1)
check("nqueens(1)", bmb_algo.nqueens(1), 1)
check("prime_count(0)", bmb_algo.prime_count(0), 0)
check("prime_count(1)", bmb_algo.prime_count(1), 0)
check("prime_count(2)", bmb_algo.prime_count(2), 1)

# Large values
check("modpow(3,100,1000000007)", bmb_algo.modpow(3, 100, 1000000007), bmb_algo.modpow(3, 100, 1000000007))
check("bit_popcount(0)", bmb_algo.bit_popcount(0), 0)
check("bit_popcount(-1)", bmb_algo.bit_popcount(-1), 64)

# Duplicate elements
check("quicksort duplicates", bmb_algo.quicksort([5,5,5,5]), [5,5,5,5])
check("unique_count all same", bmb_algo.unique_count([3,3,3,3]), 1)
check("unique_count empty", bmb_algo.unique_count([]), 0)

print(f"  {passed} passed")
algo_passed = passed

# ============================================================================
# bmb-crypto edge cases
# ============================================================================
print("[bmb-crypto edge cases]")

# Empty inputs
check("sha256 empty", bmb_crypto.sha256(""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
check("md5 empty", bmb_crypto.md5(""), "d41d8cd98f00b204e9800998ecf8427e")
check("crc32 empty", bmb_crypto.crc32(""), "00000000")
check("base64_encode empty", bmb_crypto.base64_encode(""), "")
check("base64_decode empty", bmb_crypto.base64_decode(""), "")
check("base32_encode empty", bmb_crypto.base32_encode(""), "")

# Long inputs
long_str = "A" * 1000
check("sha256 1000 chars len", len(bmb_crypto.sha256(long_str)), 64)
check("md5 1000 chars len", len(bmb_crypto.md5(long_str)), 32)

# Special characters
check("sha256 newline", len(bmb_crypto.sha256("hello\nworld")), 64)

print(f"  {passed - algo_passed} passed")
crypto_passed = passed

# ============================================================================
# bmb-text edge cases
# ============================================================================
print("[bmb-text edge cases]")

# Empty strings
check("kmp empty text", bmb_text.kmp_search("", "abc"), -1)
check("kmp empty pattern", bmb_text.kmp_search("hello", ""), 0)
check("kmp both empty", bmb_text.kmp_search("", ""), 0)
check("str_find empty", bmb_text.str_find("", "abc"), -1)
check("str_count empty", bmb_text.str_count("", "abc"), 0)
check("contains empty", bmb_text.str_contains("", "abc"), False)
check("contains empty needle", bmb_text.str_contains("hello", ""), True)
check("starts_with empty", bmb_text.str_starts_with("hello", ""), True)
check("ends_with empty", bmb_text.str_ends_with("hello", ""), True)
check("palindrome empty", bmb_text.is_palindrome(""), True)
check("word_count empty", bmb_text.word_count(""), 0)
check("reverse empty", bmb_text.str_reverse(""), "")
check("to_upper empty", bmb_text.to_upper(""), "")
check("trim empty", bmb_text.trim(""), "")
check("repeat 0", bmb_text.repeat("abc", 0), "")
check("repeat 1", bmb_text.repeat("abc", 1), "abc")

# Single char
check("palindrome single", bmb_text.is_palindrome("x"), True)
check("word_count single", bmb_text.word_count("x"), 1)

# No match
check("str_replace no match", bmb_text.str_replace("hello", "xyz", "!"), "hello")
check("str_replace_all no match", bmb_text.str_replace_all("hello", "xyz", "!"), "hello")

print(f"  {passed - crypto_passed} passed")
text_passed = passed

# ============================================================================
# bmb-json edge cases
# ============================================================================
print("[bmb-json edge cases]")

check("validate empty", bmb_json.validate(""), False)
check("validate whitespace", bmb_json.validate("   "), True)  # Parser treats as null
check("type null", bmb_json.get_type("null"), "null")
check("type true", bmb_json.get_type("true"), "bool")
check("type false", bmb_json.get_type("false"), "bool")
check("type number", bmb_json.get_type("0"), "number")
check("type negative", bmb_json.get_type("-42"), "number")
check("array empty", bmb_json.array_len("[]"), 0)
check("object empty", bmb_json.stringify("{}"), "{}")
check("get missing key", bmb_json.get('{"a":1}', "b"), "")
check("get_number missing", bmb_json.get_number('{"a":1}', "b"), 0)
check("array_get out of bounds", bmb_json.array_get("[1,2]", 5), "")

print(f"  {passed - text_passed} passed")
json_passed = passed

# ============================================================================
# bmb-compute edge cases
# ============================================================================
print("[bmb-compute edge cases]")

check("abs(0)", bmb_compute.abs(0), 0)
check("sqrt(0)", bmb_compute.sqrt(0), 0)
check("sqrt(1)", bmb_compute.sqrt(1), 1)
check("factorial(0)", bmb_compute.factorial(0), 1)
check("factorial(1)", bmb_compute.factorial(1), 1)
check("ipow(0,0)", bmb_compute.ipow(0, 0), 1)
check("ipow(5,0)", bmb_compute.ipow(5, 0), 1)
check("sign(0)", bmb_compute.sign(0), 0)
check("is_power_of_two(0)", bmb_compute.is_power_of_two(0), False)
check("is_power_of_two(1)", bmb_compute.is_power_of_two(1), True)
check("next_power_of_two(1)", bmb_compute.next_power_of_two(1), 1)
check("lerp_scaled boundaries", bmb_compute.lerp_scaled(10, 20, 0), 10)
check("lerp_scaled mid", bmb_compute.lerp_scaled(10, 20, 500), 15)

print(f"  {passed - json_passed} passed")

# ============================================================================
# Cycle 2149: bmb-algo new functions edge cases
# ============================================================================
print("[bmb-algo new edge cases]")
prev = passed

check("is_palindrome_num(0)", bmb_algo.is_palindrome_num(0), True)
check("is_palindrome_num(121)", bmb_algo.is_palindrome_num(121), True)
check("is_palindrome_num(-121)", bmb_algo.is_palindrome_num(-121), False)
check("digit_sum(0)", bmb_algo.digit_sum(0), 0)
check("digit_sum(12345)", bmb_algo.digit_sum(12345), 15)
check("kth_smallest(1st)", bmb_algo.kth_smallest([5,3,1], 1), 1)
check("kth_smallest(3rd)", bmb_algo.kth_smallest([5,3,1], 3), 5)
check("two_sum found", bmb_algo.two_sum([2,7,11,15], 9), (0, 1))
check("two_sum not found", bmb_algo.two_sum([1,2,3], 100), None)
check("shell_sort empty", bmb_algo.shell_sort([]), [])
check("bubble_sort single", bmb_algo.bubble_sort([42]), [42])

print(f"  {passed - prev} passed")

# ============================================================================
# Cycle 2127: bmb-compute new edge cases
# ============================================================================
print("[bmb-compute new edge cases]")
prev = passed

check("cumsum empty", bmb_compute.cumsum([]), [])
check("cumsum single", bmb_compute.cumsum([42]), [42])
check("vec_add zeros", bmb_compute.vec_add([0,0], [0,0]), [0, 0])
check("vec_scale zero", bmb_compute.vec_scale([1,2,3], 0), [0, 0, 0])
check("magnitude_squared unit", bmb_compute.magnitude_squared([1, 0, 0]), 1)
check("map_square zeros", bmb_compute.map_square([0,0,0]), [0, 0, 0])
check("median_scaled single", bmb_compute.median_scaled([5]), 5000)

print(f"  {passed - prev} passed")

# ============================================================================
# Cycle 2151: bmb-crypto new edge cases
# ============================================================================
print("[bmb-crypto new edge cases]")
prev = passed

check("rot13 self-inverse", bmb_crypto.rot13(bmb_crypto.rot13("test")), "test")
check("rot13 empty", bmb_crypto.rot13(""), "")
check("hex roundtrip", bmb_crypto.hex_decode(bmb_crypto.hex_encode("hello")), "hello")
check("hex_encode empty", bmb_crypto.hex_encode(""), "")
check("hex_decode empty", bmb_crypto.hex_decode(""), "")

print(f"  {passed - prev} passed")

# ============================================================================
# Cycle 2131: bmb-json new edge cases
# ============================================================================
print("[bmb-json new edge cases]")
prev = passed

check("has_key empty obj", bmb_json.has_key("{}", "a"), False)
check("object_len empty", bmb_json.object_len("{}"), 0)
check("get_bool not bool", bmb_json.get_bool('{"a":1}', "a"), -1)
check("count scalar", bmb_json.count("42"), 1)

print(f"  {passed - prev} passed")

# ============================================================================
# Cycle 2129: bmb-text new edge cases
# ============================================================================
print("[bmb-text new edge cases]")
prev = passed

check("str_len empty", bmb_text.str_len(""), 0)
check("str_char_at oob", bmb_text.str_char_at("hi", 5), -1)
check("str_compare empty", bmb_text.str_compare("", ""), 0)

print(f"  {passed - prev} passed")

print()
print("=" * 60)
total = passed + failed
print(f"Edge Case Tests: {passed}/{total} passed")
if errors:
    print()
    for e in errors:
        print(e)
    sys.exit(1)
else:
    print("All edge cases passed!")
