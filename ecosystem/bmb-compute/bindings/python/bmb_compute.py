"""
bmb-compute: Numeric computation powered by BMB
https://github.com/iyulab/lang-bmb

Functions: abs, min, max, clamp, sign, ipow, sqrt, factorial,
           sum, mean, min_val, max_val, range_val, variance,
           rand_seed, rand_next, rand_pos, rand_range,
           dot_product, dist_squared
"""

import ctypes
import os
import sys

_lib_dir = os.path.dirname(os.path.abspath(__file__))
_lib_name = {'win32': 'bmb_compute.dll', 'linux': 'libbmb_compute.so', 'darwin': 'libbmb_compute.dylib'}.get(sys.platform, 'libbmb_compute.so')
_lib_path = os.path.join(_lib_dir, _lib_name)
if not os.path.exists(_lib_path):
    _lib_path = os.path.join(_lib_dir, '..', '..', _lib_name)

if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

__all__ = [
    # Math
    'abs', 'min', 'max', 'clamp', 'sign', 'ipow', 'sqrt', 'factorial',
    # Statistics
    'sum', 'mean_scaled', 'min_val', 'max_val', 'range_val', 'variance_scaled',
    'median_scaled', 'cumsum', 'moving_avg_scaled',
    # Random
    'rand_seed', 'rand_next', 'rand_pos', 'rand_range',
    # Vector
    'dot_product', 'dist_squared', 'weighted_sum', 'lerp_scaled',
    'magnitude_squared', 'vec_add', 'vec_sub', 'vec_scale', 'map_square',
    # Utility
    'is_power_of_two', 'next_power_of_two',
]

_lib = ctypes.CDLL(_lib_path)

# FFI
_lib.bmb_ffi_begin.restype = ctypes.c_int
_lib.bmb_ffi_end.restype = None

# Math (i64 -> i64)
for fn in ['bmb_c_abs', 'bmb_sign', 'bmb_sqrt', 'bmb_factorial',
           'bmb_rand_seed', 'bmb_rand_next', 'bmb_rand_pos']:
    getattr(_lib, fn).argtypes = [ctypes.c_int64]
    getattr(_lib, fn).restype = ctypes.c_int64

for fn in ['bmb_c_min', 'bmb_c_max', 'bmb_ipow', 'bmb_rand_range']:
    getattr(_lib, fn).argtypes = [ctypes.c_int64, ctypes.c_int64]
    getattr(_lib, fn).restype = ctypes.c_int64

_lib.bmb_c_clamp.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_c_clamp.restype = ctypes.c_int64

# Stats (pointer + n)
for fn in ['bmb_sum', 'bmb_mean_scaled', 'bmb_c_min_val', 'bmb_c_max_val',
           'bmb_range_val', 'bmb_variance_scaled']:
    getattr(_lib, fn).argtypes = [ctypes.c_int64, ctypes.c_int64]
    getattr(_lib, fn).restype = ctypes.c_int64

# Vector ops
_lib.bmb_dot_product.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_dot_product.restype = ctypes.c_int64
_lib.bmb_dist_squared.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_dist_squared.restype = ctypes.c_int64
_lib.bmb_weighted_sum.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_weighted_sum.restype = ctypes.c_int64
_lib.bmb_array_copy.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_array_copy.restype = ctypes.c_int64
_lib.bmb_lerp_scaled.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_lerp_scaled.restype = ctypes.c_int64
_lib.bmb_is_power_of_two.argtypes = [ctypes.c_int64]
_lib.bmb_is_power_of_two.restype = ctypes.c_int64
_lib.bmb_next_power_of_two.argtypes = [ctypes.c_int64]
_lib.bmb_next_power_of_two.restype = ctypes.c_int64

# Cycle 2127: New functions
_lib.bmb_median_scaled.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_median_scaled.restype = ctypes.c_int64
_lib.bmb_cumsum.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_cumsum.restype = ctypes.c_int64
_lib.bmb_moving_avg_scaled.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_moving_avg_scaled.restype = ctypes.c_int64
_lib.bmb_magnitude_squared.argtypes = [ctypes.c_int64, ctypes.c_int64]
_lib.bmb_magnitude_squared.restype = ctypes.c_int64
_lib.bmb_vec_add.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_vec_add.restype = ctypes.c_int64
_lib.bmb_vec_sub.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_vec_sub.restype = ctypes.c_int64
_lib.bmb_vec_scale.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_vec_scale.restype = ctypes.c_int64
_lib.bmb_map_square.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.c_int64]
_lib.bmb_map_square.restype = ctypes.c_int64


def _arr(lst):
    n = len(lst)
    c = (ctypes.c_int64 * n)(*lst)
    return ctypes.addressof(c), n, c  # keep c alive

# Math functions
def abs(x): return _lib.bmb_c_abs(x)
def min(a, b): return _lib.bmb_c_min(a, b)
def max(a, b): return _lib.bmb_c_max(a, b)
def clamp(x, lo, hi): return _lib.bmb_c_clamp(x, lo, hi)
def sign(x): return _lib.bmb_sign(x)
def ipow(base, exp): return _lib.bmb_ipow(base, exp)
def sqrt(n): return _lib.bmb_sqrt(n)
def factorial(n): return _lib.bmb_factorial(n)

# Statistics
def sum(arr):
    p, n, _c = _arr(arr)
    return _lib.bmb_sum(p, n)

def mean_scaled(arr):
    """Mean × 1000 (3 decimal places)."""
    p, n, _c = _arr(arr)
    return _lib.bmb_mean_scaled(p, n)

def min_val(arr):
    p, n, _c = _arr(arr)
    return _lib.bmb_c_min_val(p, n)

def max_val(arr):
    p, n, _c = _arr(arr)
    return _lib.bmb_c_max_val(p, n)

def range_val(arr):
    p, n, _c = _arr(arr)
    return _lib.bmb_range_val(p, n)

def variance_scaled(arr):
    """Variance × 1000000 (6 decimal places)."""
    p, n, _c = _arr(arr)
    return _lib.bmb_variance_scaled(p, n)

# Random
def rand_seed(seed): return _lib.bmb_rand_seed(seed)
def rand_next(state): return _lib.bmb_rand_next(state)
def rand_pos(state): return _lib.bmb_rand_pos(state)
def rand_range(state, max_val): return _lib.bmb_rand_range(state, max_val)

# Vector operations
def dot_product(a, b):
    assert len(a) == len(b)
    pa, na, _ca = _arr(a)
    pb, nb, _cb = _arr(b)
    return _lib.bmb_dot_product(pa, pb, na)

def dist_squared(a, b):
    assert len(a) == len(b)
    pa, na, _ca = _arr(a)
    pb, nb, _cb = _arr(b)
    return _lib.bmb_dist_squared(pa, pb, na)

def weighted_sum(values, weights):
    assert len(values) == len(weights)
    pv, nv, _cv = _arr(values)
    pw, nw, _cw = _arr(weights)
    return _lib.bmb_weighted_sum(pv, pw, nv)

def lerp_scaled(a, b, t):
    """Linear interpolation: a + (b-a)*t/1000 where t is 0-1000."""
    return _lib.bmb_lerp_scaled(a, b, t)

def is_power_of_two(n):
    return bool(_lib.bmb_is_power_of_two(n))

def next_power_of_two(n):
    return _lib.bmb_next_power_of_two(n)


def median_scaled(arr):
    """Median of sorted array, scaled x1000. Caller must pass sorted array."""
    ptr, n, c = _arr(arr)
    return _lib.bmb_median_scaled(ptr, n)


def cumsum(arr):
    """Cumulative sum (prefix sums)."""
    n = len(arr)
    if n == 0:
        return []
    c_in = (ctypes.c_int64 * n)(*arr)
    c_out = (ctypes.c_int64 * n)()
    _lib.bmb_cumsum(ctypes.addressof(c_in), ctypes.addressof(c_out), n)
    return list(c_out)


def moving_avg_scaled(arr, k):
    """Moving average with window k, scaled x1000."""
    n = len(arr)
    if n == 0 or k <= 0:
        return []
    c_in = (ctypes.c_int64 * n)(*arr)
    out_n = n - k + 1
    if out_n <= 0:
        return []
    c_out = (ctypes.c_int64 * out_n)()
    _lib.bmb_moving_avg_scaled(ctypes.addressof(c_in), ctypes.addressof(c_out), n, k)
    return list(c_out)


def magnitude_squared(arr):
    """Sum of squares of elements."""
    ptr, n, c = _arr(arr)
    return _lib.bmb_magnitude_squared(ptr, n)


def vec_add(a, b):
    """Element-wise addition."""
    n = len(a)
    c_a = (ctypes.c_int64 * n)(*a)
    c_b = (ctypes.c_int64 * n)(*b)
    c_out = (ctypes.c_int64 * n)()
    _lib.bmb_vec_add(ctypes.addressof(c_a), ctypes.addressof(c_b), ctypes.addressof(c_out), n)
    return list(c_out)


def vec_sub(a, b):
    """Element-wise subtraction."""
    n = len(a)
    c_a = (ctypes.c_int64 * n)(*a)
    c_b = (ctypes.c_int64 * n)(*b)
    c_out = (ctypes.c_int64 * n)()
    _lib.bmb_vec_sub(ctypes.addressof(c_a), ctypes.addressof(c_b), ctypes.addressof(c_out), n)
    return list(c_out)


def vec_scale(arr, scalar):
    """Scalar multiplication."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    c_out = (ctypes.c_int64 * n)()
    _lib.bmb_vec_scale(ctypes.addressof(c_arr), scalar, ctypes.addressof(c_out), n)
    return list(c_out)


def map_square(arr):
    """Square each element."""
    n = len(arr)
    c_arr = (ctypes.c_int64 * n)(*arr)
    c_out = (ctypes.c_int64 * n)()
    _lib.bmb_map_square(ctypes.addressof(c_arr), ctypes.addressof(c_out), n)
    return list(c_out)


if __name__ == '__main__':
    passed = 0
    failed = 0

    def check(name, got, expected):
        global passed, failed
        if got == expected:
            print(f"  PASS: {name}")
            passed += 1
        else:
            print(f"  FAIL: {name} (got {got}, expected {expected})")
            failed += 1

    print("bmb-compute test suite -- Powered by BMB")
    print()

    print("[Math]")
    check("abs(-42)", abs(-42), 42)
    check("abs(0)", abs(0), 0)
    check("min(3,7)", min(3, 7), 3)
    check("max(3,7)", max(3, 7), 7)
    check("clamp(5,1,10)", clamp(5, 1, 10), 5)
    check("clamp(-5,1,10)", clamp(-5, 1, 10), 1)
    check("clamp(15,1,10)", clamp(15, 1, 10), 10)
    check("sign(-5)", sign(-5), -1)
    check("sign(0)", sign(0), 0)
    check("sign(5)", sign(5), 1)
    check("ipow(2,10)", ipow(2, 10), 1024)
    check("ipow(3,5)", ipow(3, 5), 243)
    check("sqrt(144)", sqrt(144), 12)
    check("sqrt(0)", sqrt(0), 0)
    check("sqrt(1)", sqrt(1), 1)
    check("factorial(10)", factorial(10), 3628800)
    check("factorial(0)", factorial(0), 1)

    print("[Statistics]")
    data = [10, 20, 30, 40, 50]
    check("sum", sum(data), 150)
    check("mean_scaled", mean_scaled(data), 30000)
    check("min_val", min_val(data), 10)
    check("max_val", max_val(data), 50)
    check("range_val", range_val(data), 40)
    v = variance_scaled(data)
    check("variance_scaled", v, 200000000)  # variance = 200, × 1000000 = 200000000

    print("[Random]")
    s = rand_seed(42)
    check("seed nonzero", s != 0, True)
    r1 = rand_pos(s)
    check("rand_pos >= 0", r1 >= 0, True)
    r2 = rand_range(s, 100)
    check("rand_range [0,100)", r2 >= 0 and r2 < 100, True)

    print("[Vector]")
    check("dot_product", dot_product([1,2,3], [4,5,6]), 32)
    check("dist_squared", dist_squared([0,0], [3,4]), 25)

    print("[New Functions]")
    check("weighted_sum", weighted_sum([1,2,3], [4,5,6]), 32)
    check("lerp_scaled(0,100,500)", lerp_scaled(0, 100, 500), 50)
    check("lerp_scaled(0,100,0)", lerp_scaled(0, 100, 0), 0)
    check("lerp_scaled(0,100,1000)", lerp_scaled(0, 100, 1000), 100)
    check("is_power_of_two(8)", is_power_of_two(8), True)
    check("is_power_of_two(7)", is_power_of_two(7), False)
    check("next_power_of_two(5)", next_power_of_two(5), 8)
    check("next_power_of_two(8)", next_power_of_two(8), 8)

    print()
    total = passed + failed
    print(f"Results: {passed}/{total} passed")
    if failed:
        sys.exit(1)
    else:
        print("All tests passed!")
