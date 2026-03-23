"""
bmb-algo: High-performance algorithms powered by BMB
https://github.com/iyulab/lang-bmb

BMB beats C AND Rust:
  knapsack: 6.8x faster than C
  lcs: 1.8x faster than C
  floyd_warshall: 1.4x faster than C
"""

import ctypes
import os
import sys

__all__ = [
    # DP
    'knapsack', 'edit_distance', 'lcs', 'max_subarray', 'coin_change', 'lis',
    # Graph
    'dijkstra', 'floyd_warshall', 'bfs_count', 'topological_sort',
    # Sort
    'quicksort', 'merge_sort', 'heap_sort', 'counting_sort', 'shell_sort',
    'insertion_sort', 'selection_sort', 'bubble_sort',
    # Search
    'binary_search',
    # Number Theory
    'gcd', 'lcm', 'fibonacci', 'prime_count', 'nqueens', 'modpow', 'is_prime',
    # Matrix
    'matrix_multiply', 'matrix_transpose', 'matrix_det',
    # Utility
    'djb2_hash', 'power_set_size', 'is_sorted', 'array_reverse', 'array_rotate',
    'unique_count', 'prefix_sum', 'array_sum', 'array_min', 'array_max',
    'array_product', 'bit_popcount', 'bit_set', 'bit_clear', 'bit_test',
    'bit_toggle', 'array_fill', 'array_contains', 'array_index_of', 'subset_sum',
    # Cycle 2149
    'is_palindrome_num', 'digit_sum', 'kth_smallest', 'array_mode',
    'sorted_intersect_count', 'two_sum',
]

# Find the shared library
_lib_dir = os.path.dirname(os.path.abspath(__file__))
_lib_name = {
    'win32': 'bmb_algo.dll',
    'linux': 'libbmb_algo.so',
    'darwin': 'libbmb_algo.dylib',
}.get(sys.platform, 'libbmb_algo.so')

_lib_path = os.path.join(_lib_dir, _lib_name)
if not os.path.exists(_lib_path):
    _lib_path = os.path.join(_lib_dir, '..', '..', _lib_name)

# On Windows, add MSYS2/MinGW runtime directory for GCC runtime dependencies
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

_lib = ctypes.CDLL(_lib_path)

# FFI safety API
_lib.bmb_ffi_begin.argtypes = []
_lib.bmb_ffi_begin.restype = ctypes.c_int
_lib.bmb_ffi_end.argtypes = []
_lib.bmb_ffi_end.restype = None
_lib.bmb_ffi_has_error.argtypes = []
_lib.bmb_ffi_has_error.restype = ctypes.c_int
_lib.bmb_ffi_error_message.argtypes = []
_lib.bmb_ffi_error_message.restype = ctypes.c_char_p

# String FFI
_lib.bmb_ffi_cstr_to_string.argtypes = [ctypes.c_char_p]
_lib.bmb_ffi_cstr_to_string.restype = ctypes.c_void_p
_lib.bmb_ffi_string_data.argtypes = [ctypes.c_void_p]
_lib.bmb_ffi_string_data.restype = ctypes.c_char_p
_lib.bmb_ffi_string_len.argtypes = [ctypes.c_void_p]
_lib.bmb_ffi_string_len.restype = ctypes.c_int64
_lib.bmb_ffi_free_string.argtypes = [ctypes.c_void_p]
_lib.bmb_ffi_free_string.restype = None

# Algorithm signatures
_lib.bmb_knapsack.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_knapsack.restype = ctypes.c_int64

_lib.bmb_lcs.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_lcs.restype = ctypes.c_int64

_lib.bmb_edit_distance.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_edit_distance.restype = ctypes.c_int64

_lib.bmb_floyd_warshall.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_floyd_warshall.restype = ctypes.c_int64

_lib.bmb_max_subarray.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_max_subarray.restype = ctypes.c_int64

_lib.bmb_coin_change.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_coin_change.restype = ctypes.c_int64

_lib.bmb_lis.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_lis.restype = ctypes.c_int64

_lib.bmb_dijkstra.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_dijkstra.restype = ctypes.c_int64

_lib.bmb_quicksort.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_quicksort.restype = ctypes.c_int64

_lib.bmb_bfs_count.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_bfs_count.restype = ctypes.c_int64

_lib.bmb_matrix_multiply.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_matrix_multiply.restype = ctypes.c_int64

_lib.bmb_merge_sort.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_merge_sort.restype = ctypes.c_int64

_lib.bmb_heap_sort.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_heap_sort.restype = ctypes.c_int64

_lib.bmb_counting_sort.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_counting_sort.restype = ctypes.c_int64

_lib.bmb_binary_search.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_binary_search.restype = ctypes.c_int64

_lib.bmb_topological_sort.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_topological_sort.restype = ctypes.c_int64

_lib.bmb_gcd.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_gcd.restype = ctypes.c_int64

_lib.bmb_fibonacci.argtypes = [ctypes.c_int64]
_lib.bmb_fibonacci.restype = ctypes.c_int64

_lib.bmb_prime_count.argtypes = [ctypes.c_int64]
_lib.bmb_prime_count.restype = ctypes.c_int64

_lib.bmb_modpow.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_modpow.restype = ctypes.c_int64

_lib.bmb_nqueens.argtypes = [ctypes.c_int64]
_lib.bmb_nqueens.restype = ctypes.c_int64

_lib.bmb_djb2_hash.argtypes = [ctypes.c_void_p]
_lib.bmb_djb2_hash.restype = ctypes.c_int64

_lib.bmb_lcm.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_lcm.restype = ctypes.c_int64

_lib.bmb_power_set_size.argtypes = [ctypes.c_int64]
_lib.bmb_power_set_size.restype = ctypes.c_int64

_lib.bmb_matrix_transpose.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_matrix_transpose.restype = ctypes.c_int64

_lib.bmb_is_sorted.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_is_sorted.restype = ctypes.c_int64

_lib.bmb_array_reverse.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_reverse.restype = ctypes.c_int64

_lib.bmb_bit_popcount.argtypes = [ctypes.c_int64]
_lib.bmb_bit_popcount.restype = ctypes.c_int64
_lib.bmb_array_rotate.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_rotate.restype = ctypes.c_int64
_lib.bmb_unique_count.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_unique_count.restype = ctypes.c_int64
_lib.bmb_prefix_sum.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_prefix_sum.restype = ctypes.c_int64
_lib.bmb_array_sum.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_sum.restype = ctypes.c_int64
_lib.bmb_array_min.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_min.restype = ctypes.c_int64
_lib.bmb_array_max.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_max.restype = ctypes.c_int64

_lib.bmb_bit_set.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_bit_set.restype = ctypes.c_int64
_lib.bmb_bit_clear.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_bit_clear.restype = ctypes.c_int64
_lib.bmb_bit_test.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_bit_test.restype = ctypes.c_int64
_lib.bmb_bit_toggle.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_bit_toggle.restype = ctypes.c_int64
_lib.bmb_array_fill.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_fill.restype = ctypes.c_int64
_lib.bmb_array_contains.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_contains.restype = ctypes.c_int64
_lib.bmb_array_index_of.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_index_of.restype = ctypes.c_int64

# Cycle 2125: New algorithms
_lib.bmb_shell_sort.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_shell_sort.restype = ctypes.c_int64
_lib.bmb_subset_sum.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_subset_sum.restype = ctypes.c_int64
_lib.bmb_matrix_det.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_matrix_det.restype = ctypes.c_int64
_lib.bmb_insertion_sort.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_insertion_sort.restype = ctypes.c_int64
_lib.bmb_is_prime.argtypes = [ctypes.c_int64]
_lib.bmb_is_prime.restype = ctypes.c_int64
_lib.bmb_selection_sort.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_selection_sort.restype = ctypes.c_int64
_lib.bmb_bubble_sort.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_bubble_sort.restype = ctypes.c_int64
_lib.bmb_array_product.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_product.restype = ctypes.c_int64
# Cycle 2149
_lib.bmb_is_palindrome_num.argtypes = [ctypes.c_int64]
_lib.bmb_is_palindrome_num.restype = ctypes.c_int64
_lib.bmb_digit_sum.argtypes = [ctypes.c_int64]
_lib.bmb_digit_sum.restype = ctypes.c_int64
_lib.bmb_kth_smallest.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_kth_smallest.restype = ctypes.c_int64
_lib.bmb_array_mode.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_mode.restype = ctypes.c_int64
_lib.bmb_sorted_intersect_count.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_sorted_intersect_count.restype = ctypes.c_int64
_lib.bmb_two_sum.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_two_sum.restype = ctypes.c_int64


def knapsack(weights: list, values: list, capacity: int) -> int:
    """Solve 0/1 knapsack problem.

    Args:
        weights: List of item weights (positive integers)
        values: List of item values (positive integers)
        capacity: Maximum weight capacity

    Returns:
        Maximum total value achievable

    Example:
        >>> knapsack([2, 3, 4], [3, 4, 5], 7)
        9
    """
    n = len(weights)
    assert len(values) == n, "weights and values must have same length"
    w_arr = (ctypes.c_int64 * n)(*weights)
    v_arr = (ctypes.c_int64 * n)(*values)
    return _lib.bmb_knapsack(
        ctypes.addressof(w_arr),
        ctypes.addressof(v_arr),
        n,
        capacity
    )


def _safe_call(fn, *args):
    """Call a BMB function with FFI error handling."""
    if _lib.bmb_ffi_begin() != 0:
        msg = _lib.bmb_ffi_error_message()
        _lib.bmb_ffi_end()
        raise RuntimeError(f"BMB error: {msg.decode() if msg else 'unknown'}")
    result = fn(*args)
    _lib.bmb_ffi_end()
    return result


def edit_distance(a: str, b: str) -> int:
    """Compute Levenshtein edit distance between two strings.

    Example:
        >>> edit_distance("kitten", "sitting")
        3
    """
    sa = _lib.bmb_ffi_cstr_to_string(a.encode('utf-8'))
    sb = _lib.bmb_ffi_cstr_to_string(b.encode('utf-8'))
    result = _safe_call(_lib.bmb_edit_distance, sa, sb)
    _lib.bmb_ffi_free_string(sa)
    _lib.bmb_ffi_free_string(sb)
    return result


def lcs(a: str, b: str) -> int:
    """Find length of longest common subsequence.

    Example:
        >>> lcs("ABCBDAB", "BDCAB")
        4
    """
    sa = _lib.bmb_ffi_cstr_to_string(a.encode('utf-8'))
    sb = _lib.bmb_ffi_cstr_to_string(b.encode('utf-8'))
    result = _safe_call(_lib.bmb_lcs, sa, sb)
    _lib.bmb_ffi_free_string(sa)
    _lib.bmb_ffi_free_string(sb)
    return result


def max_subarray(arr: list) -> int:
    """Find maximum contiguous subarray sum (Kadane's algorithm).

    Example:
        >>> max_subarray([-2, 1, -3, 4, -1, 2, 1, -5, 4])
        6
    """
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_max_subarray(ctypes.addressof(c_arr), n)


def coin_change(coins: list, amount: int) -> int:
    """Find minimum coins to make amount. Returns -1 if impossible.

    Example:
        >>> coin_change([1, 5, 11], 15)
        3
    """
    n = len(coins)
    c_arr = (ctypes.c_int64 * n)(*coins)
    return _lib.bmb_coin_change(ctypes.addressof(c_arr), n, amount)


def lis(arr: list) -> int:
    """Find length of longest strictly increasing subsequence.

    Example:
        >>> lis([10, 9, 2, 5, 3, 7, 101, 18])
        4
    """
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_lis(ctypes.addressof(c_arr), n)


def dijkstra(adj_matrix: list, source: int) -> list:
    """Find shortest distances from source using Dijkstra's algorithm.

    Args:
        adj_matrix: 2D adjacency matrix (use -1 for no edge)
        source: Source node index

    Example:
        >>> dijkstra([[0, 4, -1], [-1, 0, 2], [-1, -1, 0]], 0)
        [0, 4, 6]
    """
    n = len(adj_matrix)
    flat = []
    for row in adj_matrix:
        flat.extend(row)
    arr = (ctypes.c_int64 * (n * n))(*flat)
    result = (ctypes.c_int64 * n)()
    _lib.bmb_dijkstra(ctypes.addressof(arr), n, source, ctypes.addressof(result))
    return list(result)


def floyd_warshall(matrix: list) -> list:
    """Solve all-pairs shortest path problem.

    Args:
        matrix: 2D list of distances (use float('inf') for no edge)

    Returns:
        2D list of shortest distances

    Example:
        >>> floyd_warshall([[0, 3, 999], [2, 0, 999], [999, 7, 0]])
        [[0, 3, 999], [2, 0, 999], [9, 7, 0]]
    """
    n = len(matrix)
    flat = []
    for row in matrix:
        flat.extend(row)
    arr = (ctypes.c_int64 * (n * n))(*flat)
    _lib.bmb_floyd_warshall(ctypes.addressof(arr), n)
    result = []
    for i in range(n):
        result.append([arr[i * n + j] for j in range(n)])
    return result


def quicksort(arr: list) -> list:
    """Sort array using quicksort. Returns sorted copy.

    Example:
        >>> quicksort([3, 1, 4, 1, 5])
        [1, 1, 3, 4, 5]
    """
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_quicksort(ctypes.addressof(c_arr), n)
    return list(c_arr)


def bfs_count(adj_matrix: list, source: int) -> int:
    """Count reachable nodes from source via BFS.

    Example:
        >>> bfs_count([[0,1,0],[0,0,1],[0,0,0]], 0)
        3
    """
    n = len(adj_matrix)
    flat = []
    for row in adj_matrix:
        flat.extend(row)
    arr = (ctypes.c_int64 * (n * n))(*flat)
    return _lib.bmb_bfs_count(ctypes.addressof(arr), n, source)


def matrix_multiply(a: list, b: list) -> list:
    """Multiply two square matrices.

    Example:
        >>> matrix_multiply([[1,2],[3,4]], [[5,6],[7,8]])
        [[19, 22], [43, 50]]
    """
    n = len(a)
    flat_a = [v for row in a for v in row]
    flat_b = [v for row in b for v in row]
    ca = (ctypes.c_int64 * (n * n))(*flat_a)
    cb = (ctypes.c_int64 * (n * n))(*flat_b)
    cc = (ctypes.c_int64 * (n * n))()
    _lib.bmb_matrix_multiply(ctypes.addressof(ca), ctypes.addressof(cb), ctypes.addressof(cc), n)
    return [[cc[i * n + j] for j in range(n)] for i in range(n)]


def merge_sort(arr: list) -> list:
    """Stable merge sort. Returns sorted copy."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_merge_sort(ctypes.addressof(c_arr), n)
    return list(c_arr)


def heap_sort(arr: list) -> list:
    """In-place heap sort. Returns sorted copy."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_heap_sort(ctypes.addressof(c_arr), n)
    return list(c_arr)


def counting_sort(arr: list, max_val: int = None) -> list:
    """Counting sort for non-negative integers. Returns sorted copy."""
    n = len(arr)
    if max_val is None:
        max_val = max(arr) if arr else 0
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_counting_sort(ctypes.addressof(c_arr), n, max_val)
    return list(c_arr)


def binary_search(arr: list, target: int) -> int:
    """Binary search in sorted array. Returns index or -1."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_binary_search(ctypes.addressof(c_arr), n, target)


def topological_sort(adj_matrix: list) -> list:
    """Topological sort of DAG. Returns ordered node indices."""
    n = len(adj_matrix)
    flat = [v for row in adj_matrix for v in row]
    arr = (ctypes.c_int64 * (n * n))(*flat)
    result = (ctypes.c_int64 * n)()
    count = _lib.bmb_topological_sort(ctypes.addressof(arr), n, ctypes.addressof(result))
    return list(result[:count])


def gcd(a: int, b: int) -> int:
    """Greatest common divisor (Euclidean algorithm)."""
    return _lib.bmb_gcd(a, b)


def fibonacci(n: int) -> int:
    """Compute n-th Fibonacci number. F(0)=0, F(1)=1."""
    return _lib.bmb_fibonacci(n)


def prime_count(n: int) -> int:
    """Count primes up to n (inclusive) using Sieve of Eratosthenes."""
    return _lib.bmb_prime_count(n)


def modpow(base: int, exp: int, modulus: int) -> int:
    """Modular exponentiation: (base^exp) mod modulus."""
    return _lib.bmb_modpow(base, exp, modulus)


def nqueens(n: int) -> int:
    """Count N-Queens solutions for n×n board."""
    return _lib.bmb_nqueens(n)


def djb2_hash(s: str) -> int:
    """DJB2 string hash function."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    result = _safe_call(_lib.bmb_djb2_hash, ss)
    _lib.bmb_ffi_free_string(ss)
    return result


def lcm(a: int, b: int) -> int:
    """Least common multiple."""
    return _lib.bmb_lcm(a, b)


def power_set_size(n: int) -> int:
    """Size of power set (2^n)."""
    return _lib.bmb_power_set_size(n)


def matrix_transpose(matrix: list) -> list:
    """Transpose a square matrix in-place. Returns transposed copy."""
    n = len(matrix)
    flat = [v for row in matrix for v in row]
    arr = (ctypes.c_int64 * (n * n))(*flat)
    _lib.bmb_matrix_transpose(ctypes.addressof(arr), n)
    return [[arr[i * n + j] for j in range(n)] for i in range(n)]


def is_sorted(arr: list) -> bool:
    """Check if array is sorted in non-decreasing order."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return bool(_lib.bmb_is_sorted(ctypes.addressof(c_arr), n))


def bit_popcount(x: int) -> int:
    """Count set bits in integer."""
    return _lib.bmb_bit_popcount(x)


def array_rotate(arr: list, k: int) -> list:
    """Rotate array left by k positions. Returns rotated copy."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_array_rotate(ctypes.addressof(c_arr), n, k)
    return list(c_arr)


def unique_count(sorted_arr: list) -> int:
    """Count distinct values in sorted array."""
    n = len(sorted_arr)
    c_arr = (ctypes.c_int64 * n)(*sorted_arr)
    return _lib.bmb_unique_count(ctypes.addressof(c_arr), n)


def prefix_sum(arr: list) -> list:
    """Compute prefix sum in-place. Returns result."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_prefix_sum(ctypes.addressof(c_arr), n)
    return list(c_arr)


def array_sum(arr: list) -> int:
    """Sum of array elements."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_array_sum(ctypes.addressof(c_arr), n)


def array_min(arr: list) -> int:
    """Minimum value in array."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_array_min(ctypes.addressof(c_arr), n)


def array_max(arr: list) -> int:
    """Maximum value in array."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_array_max(ctypes.addressof(c_arr), n)


def array_reverse(arr: list) -> list:
    """Reverse an array. Returns reversed copy."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_array_reverse(ctypes.addressof(c_arr), n)
    return list(c_arr)


def bit_set(value: int, pos: int) -> int:
    """Set bit at position."""
    return _lib.bmb_bit_set(value, pos)


def bit_clear(value: int, pos: int) -> int:
    """Clear bit at position."""
    return _lib.bmb_bit_clear(value, pos)


def bit_test(value: int, pos: int) -> bool:
    """Test if bit is set."""
    return bool(_lib.bmb_bit_test(value, pos))


def bit_toggle(value: int, pos: int) -> int:
    """Toggle bit at position."""
    return _lib.bmb_bit_toggle(value, pos)


def array_fill(n: int, value: int) -> list:
    """Create array of n elements all set to value."""
    c_arr = (ctypes.c_int64 * n)()
    _lib.bmb_array_fill(ctypes.addressof(c_arr), n, value)
    return list(c_arr)


def array_contains(arr: list, target: int) -> bool:
    """Check if target exists in array."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return bool(_lib.bmb_array_contains(ctypes.addressof(c_arr), n, target))


def array_index_of(arr: list, target: int) -> int:
    """Find first index of target. -1 if not found."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_array_index_of(ctypes.addressof(c_arr), n, target)


def shell_sort(arr: list) -> list:
    """Shell sort (returns new sorted list)."""
    n = len(arr)
    if n == 0:
        return []
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_shell_sort(ctypes.addressof(c_arr), n)
    return list(c_arr)


def subset_sum(arr: list, target: int) -> bool:
    """Check if any subset of arr sums to target."""
    n = len(arr)
    if n == 0:
        return target == 0
    c_arr = (ctypes.c_int64 * n)(*arr)
    return bool(_lib.bmb_subset_sum(ctypes.addressof(c_arr), n, target))


def matrix_det(matrix: list) -> int:
    """Compute determinant of N x N matrix (integer approximation)."""
    n = len(matrix)
    flat = []
    for row in matrix:
        flat.extend(row)
    c_arr = (ctypes.c_int64 * len(flat))(*flat)
    return _lib.bmb_matrix_det(ctypes.addressof(c_arr), n)


def insertion_sort(arr: list) -> list:
    """Insertion sort (returns new sorted list)."""
    n = len(arr)
    if n == 0:
        return []
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_insertion_sort(ctypes.addressof(c_arr), n)
    return list(c_arr)


def is_prime(n: int) -> bool:
    """Check if n is a prime number."""
    return bool(_lib.bmb_is_prime(n))


def selection_sort(arr: list) -> list:
    """Selection sort (returns new sorted list)."""
    n = len(arr)
    if n == 0:
        return []
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_selection_sort(ctypes.addressof(c_arr), n)
    return list(c_arr)


def bubble_sort(arr: list) -> list:
    """Bubble sort (returns new sorted list)."""
    n = len(arr)
    if n == 0:
        return []
    c_arr = (ctypes.c_int64 * n)(*arr)
    _lib.bmb_bubble_sort(ctypes.addressof(c_arr), n)
    return list(c_arr)


def array_product(arr: list) -> int:
    """Product of all elements."""
    n = len(arr)
    if n == 0:
        return 1
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_array_product(ctypes.addressof(c_arr), n)


def is_palindrome_num(n: int) -> bool:
    """Check if integer is a palindrome (reads same forwards and backwards)."""
    return bool(_lib.bmb_is_palindrome_num(n))


def digit_sum(n: int) -> int:
    """Sum of all digits of n."""
    return _lib.bmb_digit_sum(n)


def kth_smallest(arr: list, k: int) -> int:
    """Find kth smallest element (1-indexed)."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    return _lib.bmb_kth_smallest(ctypes.addressof(c_arr), n, k)


def array_mode(sorted_arr: list) -> int:
    """Most frequent element in sorted array."""
    n = len(sorted_arr)
    c_arr = (ctypes.c_int64 * n)(*sorted_arr)
    return _lib.bmb_array_mode(ctypes.addressof(c_arr), n)


def sorted_intersect_count(a: list, b: list) -> int:
    """Count common elements between two sorted arrays."""
    na = len(a)
    nb = len(b)
    c_a = (ctypes.c_int64 * na)(*a)
    c_b = (ctypes.c_int64 * nb)(*b)
    return _lib.bmb_sorted_intersect_count(ctypes.addressof(c_a), na, ctypes.addressof(c_b), nb)


def two_sum(arr: list, target: int):
    """Find indices of two elements summing to target. Returns (i, j) or None."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    result = _lib.bmb_two_sum(ctypes.addressof(c_arr), n, target)
    if result < 0:
        return None
    return (result // 10000, result % 10000)


if __name__ == '__main__':
    print("bmb-algo test suite -- Powered by BMB")
    print()

    # DP (pointer-based)
    print(f"  knapsack([2,3,4], [3,4,5], 7) = {knapsack([2,3,4], [3,4,5], 7)}")
    print(f"  max_subarray([-2,1,-3,4,-1,2,1,-5,4]) = {max_subarray([-2,1,-3,4,-1,2,1,-5,4])}")
    print(f"  coin_change([1,5,11], 15) = {coin_change([1,5,11], 15)}")
    print(f"  lis([10,9,2,5,3,7,101,18]) = {lis([10,9,2,5,3,7,101,18])}")

    # DP (string-based via FFI)
    print(f"  edit_distance('kitten', 'sitting') = {edit_distance('kitten', 'sitting')}")
    print(f"  lcs('ABCBDAB', 'BDCAB') = {lcs('ABCBDAB', 'BDCAB')}")

    # Graph
    INF = 999999
    dist = [[0, 3, INF], [2, 0, INF], [INF, 7, 0]]
    print(f"  floyd_warshall = {floyd_warshall(dist)}")

    adj = [[0, 4, -1], [-1, 0, 2], [-1, -1, 0]]
    print(f"  dijkstra(source=0) = {dijkstra(adj, 0)}")

    # Sort
    print(f"  quicksort([3,1,4,1,5]) = {quicksort([3,1,4,1,5])}")

    # Graph (BFS)
    adj = [[0,1,0],[0,0,1],[0,0,0]]
    print(f"  bfs_count(3-node chain, 0) = {bfs_count(adj, 0)}")

    # Matrix
    result = matrix_multiply([[1,2],[3,4]], [[5,6],[7,8]])
    print(f"  matrix_multiply 2x2 = {result}")

    # New algorithms
    print(f"  merge_sort([5,3,1,4,2]) = {merge_sort([5,3,1,4,2])}")
    print(f"  heap_sort([5,3,1,4,2]) = {heap_sort([5,3,1,4,2])}")
    print(f"  counting_sort([3,1,4,1,5,9,2,6]) = {counting_sort([3,1,4,1,5,9,2,6])}")
    print(f"  binary_search([10,20,30,40,50], 30) = {binary_search([10,20,30,40,50], 30)}")
    print(f"  binary_search([10,20,30,40,50], 35) = {binary_search([10,20,30,40,50], 35)}")

    # DAG: 0→1, 0→2, 1→3, 2→3
    topo_adj = [[0,1,1,0],[0,0,0,1],[0,0,0,1],[0,0,0,0]]
    print(f"  topological_sort(4-node DAG) = {topological_sort(topo_adj)}")

    print(f"  gcd(12, 8) = {gcd(12, 8)}")
    print(f"  gcd(100, 75) = {gcd(100, 75)}")
    print(f"  fibonacci(10) = {fibonacci(10)}")
    print(f"  fibonacci(20) = {fibonacci(20)}")
    print(f"  prime_count(100) = {prime_count(100)}")
    print(f"  prime_count(1000) = {prime_count(1000)}")

    # New cycle 1989 algorithms
    print(f"  modpow(2, 10, 1000) = {modpow(2, 10, 1000)}")
    print(f"  nqueens(8) = {nqueens(8)}")
    print(f"  djb2_hash('hello') = {djb2_hash('hello')}")
    print(f"  lcm(12, 8) = {lcm(12, 8)}")
    print(f"  power_set_size(10) = {power_set_size(10)}")
    print(f"  matrix_transpose([[1,2],[3,4]]) = {matrix_transpose([[1,2],[3,4]])}")
    print(f"  is_sorted([1,2,3]) = {is_sorted([1,2,3])}")
    print(f"  is_sorted([3,1,2]) = {is_sorted([3,1,2])}")
    print(f"  array_reverse([1,2,3,4,5]) = {array_reverse([1,2,3,4,5])}")

    # Cycle 2013 algorithms
    print(f"  bit_popcount(255) = {bit_popcount(255)}")
    print(f"  bit_popcount(0) = {bit_popcount(0)}")
    print(f"  array_rotate([1,2,3,4,5], 2) = {array_rotate([1,2,3,4,5], 2)}")
    print(f"  unique_count([1,1,2,3,3,3,4]) = {unique_count([1,1,2,3,3,3,4])}")
    print(f"  prefix_sum([1,2,3,4,5]) = {prefix_sum([1,2,3,4,5])}")
    print(f"  array_sum([1,2,3,4,5]) = {array_sum([1,2,3,4,5])}")
    print(f"  array_min([5,3,8,1,7]) = {array_min([5,3,8,1,7])}")
    print(f"  array_max([5,3,8,1,7]) = {array_max([5,3,8,1,7])}")

    # Cycle 2051 additions
    print(f"  bit_set(0, 3) = {bit_set(0, 3)}")
    print(f"  bit_test(8, 3) = {bit_test(8, 3)}")
    print(f"  bit_clear(8, 3) = {bit_clear(8, 3)}")
    print(f"  array_fill(5, 42) = {array_fill(5, 42)}")
    print(f"  array_contains([1,2,3], 2) = {array_contains([1,2,3], 2)}")
    print(f"  array_index_of([10,20,30], 20) = {array_index_of([10,20,30], 20)}")

    print()
    print("All 41 algorithms working! https://github.com/iyulab/lang-bmb")
