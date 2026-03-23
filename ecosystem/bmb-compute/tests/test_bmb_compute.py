"""
pytest test suite for bmb-compute.

Covers the following core functions:
  abs, sqrt, factorial, ipow,
  sum, mean_scaled, min_val, max_val, range_val,
  dot_product, dist_squared, clamp, sign,
  weighted_sum, lerp_scaled,
  is_power_of_two, next_power_of_two

All numeric results are cross-validated against plain Python arithmetic where
applicable so that the tests remain self-documenting.
"""

import math
import pytest
import bmb_compute


# ---------------------------------------------------------------------------
# abs
# ---------------------------------------------------------------------------

class TestAbs:
    """bmb_compute.abs(n) -> int"""

    def test_positive(self):
        assert bmb_compute.abs(42) == 42

    def test_negative(self):
        assert bmb_compute.abs(-42) == 42

    def test_zero(self):
        assert bmb_compute.abs(0) == 0

    def test_one(self):
        assert bmb_compute.abs(1) == 1

    def test_minus_one(self):
        assert bmb_compute.abs(-1) == 1

    def test_large_positive(self):
        assert bmb_compute.abs(1_000_000) == 1_000_000

    def test_large_negative(self):
        assert bmb_compute.abs(-1_000_000) == 1_000_000

    @pytest.mark.parametrize("n", [-100, -50, -1, 0, 1, 50, 100])
    def test_cross_validate(self, n):
        assert bmb_compute.abs(n) == builtins_abs(n)


def builtins_abs(n):
    return n if n >= 0 else -n


# ---------------------------------------------------------------------------
# sqrt  (integer square root)
# ---------------------------------------------------------------------------

class TestSqrt:
    """bmb_compute.sqrt(n) -> int  (floor of real sqrt)"""

    def test_zero(self):
        assert bmb_compute.sqrt(0) == 0

    def test_one(self):
        assert bmb_compute.sqrt(1) == 1

    def test_four(self):
        assert bmb_compute.sqrt(4) == 2

    def test_nine(self):
        assert bmb_compute.sqrt(9) == 3

    def test_perfect_square_144(self):
        assert bmb_compute.sqrt(144) == 12

    def test_perfect_square_10000(self):
        assert bmb_compute.sqrt(10000) == 100

    def test_non_perfect_square_2(self):
        assert bmb_compute.sqrt(2) == 1   # floor(1.414...)

    def test_non_perfect_square_3(self):
        assert bmb_compute.sqrt(3) == 1   # floor(1.732...)

    def test_non_perfect_square_8(self):
        assert bmb_compute.sqrt(8) == 2   # floor(2.828...)

    def test_non_perfect_square_15(self):
        assert bmb_compute.sqrt(15) == 3  # floor(3.872...)

    def test_non_perfect_square_99(self):
        assert bmb_compute.sqrt(99) == 9  # floor(9.949...)

    @pytest.mark.parametrize("n", [0, 1, 2, 3, 4, 9, 16, 25, 36, 49, 100, 255, 256])
    def test_cross_validate_floor(self, n):
        expected = int(math.isqrt(n))
        assert bmb_compute.sqrt(n) == expected


# ---------------------------------------------------------------------------
# factorial
# ---------------------------------------------------------------------------

class TestFactorial:
    """bmb_compute.factorial(n) -> int"""

    def test_zero(self):
        assert bmb_compute.factorial(0) == 1

    def test_one(self):
        assert bmb_compute.factorial(1) == 1

    def test_two(self):
        assert bmb_compute.factorial(2) == 2

    def test_three(self):
        assert bmb_compute.factorial(3) == 6

    def test_four(self):
        assert bmb_compute.factorial(4) == 24

    def test_five(self):
        assert bmb_compute.factorial(5) == 120

    def test_ten(self):
        assert bmb_compute.factorial(10) == 3_628_800

    def test_twelve(self):
        assert bmb_compute.factorial(12) == 479_001_600

    @pytest.mark.parametrize("n", range(0, 13))
    def test_cross_validate(self, n):
        assert bmb_compute.factorial(n) == math.factorial(n)


# ---------------------------------------------------------------------------
# ipow
# ---------------------------------------------------------------------------

class TestIpow:
    """bmb_compute.ipow(base, exp) -> int"""

    def test_power_of_two(self):
        assert bmb_compute.ipow(2, 10) == 1024

    def test_power_of_three(self):
        assert bmb_compute.ipow(3, 5) == 243

    def test_any_to_zero(self):
        assert bmb_compute.ipow(7, 0) == 1

    def test_any_to_one(self):
        assert bmb_compute.ipow(7, 1) == 7

    def test_zero_to_positive(self):
        assert bmb_compute.ipow(0, 5) == 0

    def test_one_to_large(self):
        assert bmb_compute.ipow(1, 1000) == 1

    def test_negative_base_even_exp(self):
        assert bmb_compute.ipow(-2, 4) == 16

    def test_negative_base_odd_exp(self):
        assert bmb_compute.ipow(-2, 3) == -8

    def test_large_result(self):
        assert bmb_compute.ipow(10, 6) == 1_000_000

    @pytest.mark.parametrize("base,exp", [
        (2, 0), (2, 1), (2, 8), (2, 16),
        (3, 0), (3, 3), (3, 6),
        (5, 4), (10, 5),
    ])
    def test_cross_validate(self, base, exp):
        assert bmb_compute.ipow(base, exp) == base ** exp


# ---------------------------------------------------------------------------
# sum
# ---------------------------------------------------------------------------

class TestSum:
    """bmb_compute.sum(arr) -> int"""

    def test_basic(self):
        assert bmb_compute.sum([10, 20, 30, 40, 50]) == 150

    def test_single_element(self):
        assert bmb_compute.sum([42]) == 42

    def test_all_zeros(self):
        assert bmb_compute.sum([0, 0, 0]) == 0

    def test_negatives(self):
        assert bmb_compute.sum([-1, -2, -3]) == -6

    def test_mixed_sign(self):
        assert bmb_compute.sum([-5, 5, -3, 3]) == 0

    def test_large_values(self):
        assert bmb_compute.sum([100_000, 200_000, 300_000]) == 600_000

    @pytest.mark.parametrize("arr", [
        [1, 2, 3, 4, 5],
        [0],
        [-1, 0, 1],
        [10, -10, 20, -20],
        list(range(1, 11)),
    ])
    def test_cross_validate(self, arr):
        assert bmb_compute.sum(arr) == sum(arr)


# ---------------------------------------------------------------------------
# mean_scaled
# ---------------------------------------------------------------------------

class TestMeanScaled:
    """bmb_compute.mean_scaled(arr) -> int  (mean * 1000, truncated)"""

    def test_uniform(self):
        # mean([10,20,30,40,50]) = 30 -> 30000
        assert bmb_compute.mean_scaled([10, 20, 30, 40, 50]) == 30_000

    def test_single_element(self):
        assert bmb_compute.mean_scaled([7]) == 7_000

    def test_two_elements_exact(self):
        # mean([0,10]) = 5 -> 5000
        assert bmb_compute.mean_scaled([0, 10]) == 5_000

    def test_zero_mean(self):
        assert bmb_compute.mean_scaled([-5, 5]) == 0

    def test_all_same(self):
        assert bmb_compute.mean_scaled([4, 4, 4, 4]) == 4_000

    def test_known_fractional(self):
        # mean([1,2]) = 1.5 -> 1500
        assert bmb_compute.mean_scaled([1, 2]) == 1_500

    @pytest.mark.parametrize("arr", [
        [10, 20, 30, 40, 50],
        [1, 2],
        [0, 100],
        [7, 7, 7],
        [1, 3, 5, 7, 9],
    ])
    def test_cross_validate_approximate(self, arr):
        expected = int(sum(arr) / len(arr) * 1000)
        got = bmb_compute.mean_scaled(arr)
        # Allow off-by-one for integer truncation vs rounding
        assert abs(got - expected) <= 1


# ---------------------------------------------------------------------------
# min_val / max_val / range_val
# ---------------------------------------------------------------------------

class TestMinMaxRange:
    """min_val, max_val, range_val on arrays"""

    DATA = [10, 20, 30, 40, 50]

    def test_min_val(self):
        assert bmb_compute.min_val(self.DATA) == 10

    def test_max_val(self):
        assert bmb_compute.max_val(self.DATA) == 50

    def test_range_val(self):
        assert bmb_compute.range_val(self.DATA) == 40

    def test_single_element_min(self):
        assert bmb_compute.min_val([7]) == 7

    def test_single_element_max(self):
        assert bmb_compute.max_val([7]) == 7

    def test_single_element_range(self):
        assert bmb_compute.range_val([7]) == 0

    def test_negative_values_min(self):
        assert bmb_compute.min_val([-5, -3, -1, 0, 2]) == -5

    def test_negative_values_max(self):
        assert bmb_compute.max_val([-5, -3, -1, 0, 2]) == 2

    def test_all_same_range(self):
        assert bmb_compute.range_val([3, 3, 3, 3]) == 0

    def test_two_elements(self):
        assert bmb_compute.min_val([100, 1]) == 1
        assert bmb_compute.max_val([100, 1]) == 100
        assert bmb_compute.range_val([100, 1]) == 99

    @pytest.mark.parametrize("arr", [
        [10, 20, 30, 40, 50],
        [-5, 0, 5],
        [1],
        [100, 1, 50, 25],
    ])
    def test_cross_validate_min(self, arr):
        assert bmb_compute.min_val(arr) == min(arr)

    @pytest.mark.parametrize("arr", [
        [10, 20, 30, 40, 50],
        [-5, 0, 5],
        [1],
        [100, 1, 50, 25],
    ])
    def test_cross_validate_max(self, arr):
        assert bmb_compute.max_val(arr) == max(arr)

    @pytest.mark.parametrize("arr", [
        [10, 20, 30, 40, 50],
        [-5, 0, 5],
        [1],
        [100, 1, 50, 25],
    ])
    def test_cross_validate_range(self, arr):
        assert bmb_compute.range_val(arr) == max(arr) - min(arr)


# ---------------------------------------------------------------------------
# dot_product
# ---------------------------------------------------------------------------

class TestDotProduct:
    """bmb_compute.dot_product(a, b) -> int"""

    def test_basic(self):
        assert bmb_compute.dot_product([1, 2, 3], [4, 5, 6]) == 32

    def test_zero_vector(self):
        assert bmb_compute.dot_product([0, 0, 0], [1, 2, 3]) == 0

    def test_unit_vectors_parallel(self):
        assert bmb_compute.dot_product([1, 0], [1, 0]) == 1

    def test_unit_vectors_orthogonal(self):
        assert bmb_compute.dot_product([1, 0], [0, 1]) == 0

    def test_single_element(self):
        assert bmb_compute.dot_product([7], [3]) == 21

    def test_negative_values(self):
        assert bmb_compute.dot_product([-1, -2], [3, 4]) == -11

    def test_mixed_signs(self):
        assert bmb_compute.dot_product([1, -1], [1, 1]) == 0

    @pytest.mark.parametrize("a,b,expected", [
        ([1, 2, 3], [4, 5, 6], 32),
        ([0, 0], [5, 5], 0),
        ([3, 4], [4, 3], 24),
        ([1, 1, 1, 1], [1, 1, 1, 1], 4),
    ])
    def test_cross_validate(self, a, b, expected):
        py_result = sum(x * y for x, y in zip(a, b))
        assert py_result == expected
        assert bmb_compute.dot_product(a, b) == expected


# ---------------------------------------------------------------------------
# dist_squared
# ---------------------------------------------------------------------------

class TestDistSquared:
    """bmb_compute.dist_squared(a, b) -> int  (sum of (a_i - b_i)^2)"""

    def test_3_4_5_triangle(self):
        # dist([0,0], [3,4]) = 5; dist^2 = 25
        assert bmb_compute.dist_squared([0, 0], [3, 4]) == 25

    def test_same_point(self):
        assert bmb_compute.dist_squared([1, 2, 3], [1, 2, 3]) == 0

    def test_unit_step_1d(self):
        assert bmb_compute.dist_squared([0], [1]) == 1

    def test_single_dimension(self):
        assert bmb_compute.dist_squared([0], [5]) == 25

    def test_negative_coordinates(self):
        assert bmb_compute.dist_squared([-1, -1], [2, 3]) == 25  # (3^2 + 4^2)

    def test_symmetric(self):
        a = [1, 2, 3]
        b = [4, 5, 6]
        assert bmb_compute.dist_squared(a, b) == bmb_compute.dist_squared(b, a)

    @pytest.mark.parametrize("a,b", [
        ([0, 0], [3, 4]),
        ([1, 1], [4, 5]),
        ([0], [10]),
        ([1, 2, 3], [4, 6, 3]),
    ])
    def test_cross_validate(self, a, b):
        expected = sum((x - y) ** 2 for x, y in zip(a, b))
        assert bmb_compute.dist_squared(a, b) == expected


# ---------------------------------------------------------------------------
# clamp
# ---------------------------------------------------------------------------

class TestClamp:
    """bmb_compute.clamp(val, lo, hi) -> int"""

    def test_within_range(self):
        assert bmb_compute.clamp(5, 1, 10) == 5

    def test_below_lo(self):
        assert bmb_compute.clamp(-5, 1, 10) == 1

    def test_above_hi(self):
        assert bmb_compute.clamp(15, 1, 10) == 10

    def test_at_lo(self):
        assert bmb_compute.clamp(1, 1, 10) == 1

    def test_at_hi(self):
        assert bmb_compute.clamp(10, 1, 10) == 10

    def test_zero_range(self):
        assert bmb_compute.clamp(5, 5, 5) == 5

    def test_negative_range(self):
        assert bmb_compute.clamp(-3, -10, -1) == -3

    def test_below_negative_range(self):
        assert bmb_compute.clamp(-15, -10, -1) == -10

    def test_above_negative_range(self):
        assert bmb_compute.clamp(0, -10, -1) == -1

    def test_zero_value_positive_range(self):
        assert bmb_compute.clamp(0, 1, 10) == 1

    @pytest.mark.parametrize("val,lo,hi,expected", [
        (5,  1, 10, 5),
        (-5, 1, 10, 1),
        (15, 1, 10, 10),
        (0,  0, 0,  0),
        (100, -100, 50, 50),
    ])
    def test_cross_validate(self, val, lo, hi, expected):
        py_result = max(lo, min(hi, val))
        assert py_result == expected
        assert bmb_compute.clamp(val, lo, hi) == expected


# ---------------------------------------------------------------------------
# sign
# ---------------------------------------------------------------------------

class TestSign:
    """bmb_compute.sign(n) -> int  (-1, 0, or 1)"""

    def test_positive(self):
        assert bmb_compute.sign(42) == 1

    def test_negative(self):
        assert bmb_compute.sign(-42) == -1

    def test_zero(self):
        assert bmb_compute.sign(0) == 0

    def test_one(self):
        assert bmb_compute.sign(1) == 1

    def test_minus_one(self):
        assert bmb_compute.sign(-1) == -1

    def test_large_positive(self):
        assert bmb_compute.sign(1_000_000) == 1

    def test_large_negative(self):
        assert bmb_compute.sign(-1_000_000) == -1

    @pytest.mark.parametrize("n,expected", [
        (-100, -1), (-1, -1), (0, 0), (1, 1), (100, 1),
    ])
    def test_cross_validate(self, n, expected):
        assert bmb_compute.sign(n) == expected

    def test_return_values_are_only_minus1_0_1(self):
        samples = [-999, -1, 0, 1, 999]
        for s in samples:
            result = bmb_compute.sign(s)
            assert result in (-1, 0, 1)


# ---------------------------------------------------------------------------
# weighted_sum
# ---------------------------------------------------------------------------

class TestWeightedSum:
    """bmb_compute.weighted_sum(vals, weights) -> int"""

    def test_basic(self):
        # [1,2,3] dot [4,5,6] = 4+10+18 = 32
        assert bmb_compute.weighted_sum([1, 2, 3], [4, 5, 6]) == 32

    def test_unit_weights(self):
        assert bmb_compute.weighted_sum([10, 20, 30], [1, 1, 1]) == 60

    def test_zero_weights(self):
        assert bmb_compute.weighted_sum([10, 20, 30], [0, 0, 0]) == 0

    def test_single_element(self):
        assert bmb_compute.weighted_sum([5], [4]) == 20

    def test_negative_weights(self):
        assert bmb_compute.weighted_sum([1, 2], [-1, 1]) == 1

    def test_negative_values(self):
        assert bmb_compute.weighted_sum([-1, -2, -3], [1, 1, 1]) == -6

    @pytest.mark.parametrize("vals,weights", [
        ([1, 2, 3], [4, 5, 6]),
        ([10, 20], [1, 2]),
        ([0, 0, 0], [1, 2, 3]),
        ([5], [7]),
    ])
    def test_cross_validate(self, vals, weights):
        expected = sum(v * w for v, w in zip(vals, weights))
        assert bmb_compute.weighted_sum(vals, weights) == expected


# ---------------------------------------------------------------------------
# lerp_scaled
# ---------------------------------------------------------------------------

class TestLerpScaled:
    """bmb_compute.lerp_scaled(a, b, t) -> int  (a + (b-a)*t/1000; t in [0,1000])"""

    def test_t_zero(self):
        assert bmb_compute.lerp_scaled(0, 100, 0) == 0

    def test_t_full(self):
        assert bmb_compute.lerp_scaled(0, 100, 1000) == 100

    def test_t_half(self):
        assert bmb_compute.lerp_scaled(0, 100, 500) == 50

    def test_t_quarter(self):
        assert bmb_compute.lerp_scaled(0, 100, 250) == 25

    def test_same_endpoints(self):
        assert bmb_compute.lerp_scaled(7, 7, 500) == 7

    def test_negative_a(self):
        assert bmb_compute.lerp_scaled(-100, 100, 500) == 0

    def test_negative_b(self):
        assert bmb_compute.lerp_scaled(0, -100, 500) == -50

    def test_large_range(self):
        assert bmb_compute.lerp_scaled(0, 1000, 500) == 500

    def test_t_zero_returns_a_regardless(self):
        assert bmb_compute.lerp_scaled(42, 99, 0) == 42

    def test_t_1000_returns_b_regardless(self):
        assert bmb_compute.lerp_scaled(42, 99, 1000) == 99

    @pytest.mark.parametrize("a,b,t", [
        (0,   100,  0),
        (0,   100,  500),
        (0,   100,  1000),
        (0,   200,  250),
        (-50, 50,   500),
    ])
    def test_cross_validate(self, a, b, t):
        expected = int(a + (b - a) * t / 1000)
        got = bmb_compute.lerp_scaled(a, b, t)
        # Allow off-by-one for integer truncation
        assert abs(got - expected) <= 1


# ---------------------------------------------------------------------------
# is_power_of_two
# ---------------------------------------------------------------------------

class TestIsPowerOfTwo:
    """bmb_compute.is_power_of_two(n) -> bool"""

    @pytest.mark.parametrize("n", [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024])
    def test_powers_of_two(self, n):
        assert bmb_compute.is_power_of_two(n) is True

    @pytest.mark.parametrize("n", [3, 5, 6, 7, 9, 10, 12, 15, 100, 1000])
    def test_non_powers_of_two(self, n):
        assert bmb_compute.is_power_of_two(n) is False

    def test_zero(self):
        # 0 is not a power of two by mathematical convention
        result = bmb_compute.is_power_of_two(0)
        assert result is False

    def test_large_power(self):
        assert bmb_compute.is_power_of_two(1 << 20) is True

    def test_returns_bool(self):
        result = bmb_compute.is_power_of_two(4)
        assert isinstance(result, bool)

    @pytest.mark.parametrize("n", range(1, 33))
    def test_cross_validate(self, n):
        expected = (n & (n - 1)) == 0 if n > 0 else False
        assert bmb_compute.is_power_of_two(n) == expected


# ---------------------------------------------------------------------------
# next_power_of_two
# ---------------------------------------------------------------------------

class TestNextPowerOfTwo:
    """bmb_compute.next_power_of_two(n) -> int  (smallest power of two >= n)"""

    def test_already_power_of_two_1(self):
        assert bmb_compute.next_power_of_two(1) == 1

    def test_already_power_of_two_2(self):
        assert bmb_compute.next_power_of_two(2) == 2

    def test_already_power_of_two_8(self):
        assert bmb_compute.next_power_of_two(8) == 8

    def test_between_powers_5(self):
        assert bmb_compute.next_power_of_two(5) == 8

    def test_between_powers_3(self):
        assert bmb_compute.next_power_of_two(3) == 4

    def test_between_powers_100(self):
        assert bmb_compute.next_power_of_two(100) == 128

    def test_between_powers_1000(self):
        assert bmb_compute.next_power_of_two(1000) == 1024

    def test_between_powers_65(self):
        assert bmb_compute.next_power_of_two(65) == 128

    def test_result_is_power_of_two(self):
        """The result must always itself be a power of two."""
        for n in [3, 5, 7, 9, 13, 17, 33, 65, 127, 129, 500, 1023]:
            result = bmb_compute.next_power_of_two(n)
            assert result & (result - 1) == 0, f"next_power_of_two({n})={result} is not a power of two"

    def test_result_is_gte_input(self):
        """The result must always be >= the input."""
        for n in range(1, 200):
            result = bmb_compute.next_power_of_two(n)
            assert result >= n, f"next_power_of_two({n})={result} < {n}"

    def test_result_is_smallest_power(self):
        """No smaller power of two can satisfy >= n."""
        for n in range(1, 200):
            result = bmb_compute.next_power_of_two(n)
            if result > 1:
                assert result // 2 < n, (
                    f"next_power_of_two({n})={result} but {result//2} >= {n} also works"
                )


# ---------------------------------------------------------------------------
# Integration: compound calculations
# ---------------------------------------------------------------------------

class TestIntegration:
    """Multi-function scenarios that mirror real computation pipelines."""

    def test_euclidean_distance_via_sqrt_dist_squared(self):
        # dist([0,0], [3,4]) = 5
        dsq = bmb_compute.dist_squared([0, 0], [3, 4])
        assert dsq == 25
        d = bmb_compute.sqrt(dsq)
        assert d == 5

    def test_clamped_stats_pipeline(self):
        raw = [-100, 5, 10, 15, 200]
        clamped = [bmb_compute.clamp(v, 0, 20) for v in raw]
        assert clamped == [0, 5, 10, 15, 20]
        assert bmb_compute.sum(clamped) == 50
        assert bmb_compute.min_val(clamped) == 0
        assert bmb_compute.max_val(clamped) == 20

    def test_ipow_then_is_power_of_two(self):
        for exp in range(0, 10):
            val = bmb_compute.ipow(2, exp)
            assert bmb_compute.is_power_of_two(val) is True

    def test_factorial_via_ipow_small(self):
        # 2! = 2 = 2^1, 4! = 24 (not a power of 2, just checking values)
        assert bmb_compute.factorial(5) == 120
        assert bmb_compute.ipow(2, 3) * bmb_compute.factorial(3) == 48  # 8 * 6

    def test_sign_abs_identity(self):
        """sign(n) * abs(n) == n for all n != 0."""
        for n in [-50, -1, 1, 50, 100]:
            assert bmb_compute.sign(n) * bmb_compute.abs(n) == n

    def test_lerp_endpoints_via_clamp(self):
        # lerp at t=0 gives a; lerp at t=1000 gives b.
        # Both must pass clamp(result, a, b) == result when a <= b.
        a, b = 10, 90
        assert bmb_compute.lerp_scaled(a, b, 0) == a
        assert bmb_compute.lerp_scaled(a, b, 1000) == b
        mid = bmb_compute.lerp_scaled(a, b, 500)
        assert bmb_compute.clamp(mid, a, b) == mid

    def test_dot_product_equals_weighted_sum(self):
        """dot_product and weighted_sum are semantically identical."""
        a = [1, 2, 3, 4]
        b = [5, 6, 7, 8]
        assert bmb_compute.dot_product(a, b) == bmb_compute.weighted_sum(a, b)

    def test_next_power_of_two_after_sqrt(self):
        # sqrt(1000) = 31; next_power_of_two(31) = 32
        s = bmb_compute.sqrt(1000)
        assert s == 31
        assert bmb_compute.next_power_of_two(s) == 32


class TestNewComputeFunctions:
    """Tests for Cycle 2127-2128 functions."""

    def test_median_scaled_odd(self):
        assert bmb_compute.median_scaled([1, 2, 3, 4, 5]) == 3000

    def test_median_scaled_even(self):
        assert bmb_compute.median_scaled([1, 2, 3, 4]) == 2500

    def test_median_scaled_single(self):
        assert bmb_compute.median_scaled([42]) == 42000

    def test_cumsum_basic(self):
        assert bmb_compute.cumsum([1, 2, 3, 4, 5]) == [1, 3, 6, 10, 15]

    def test_cumsum_single(self):
        assert bmb_compute.cumsum([10]) == [10]

    def test_cumsum_empty(self):
        assert bmb_compute.cumsum([]) == []

    def test_magnitude_squared_basic(self):
        assert bmb_compute.magnitude_squared([3, 4]) == 25

    def test_magnitude_squared_unit(self):
        assert bmb_compute.magnitude_squared([1, 0, 0]) == 1

    def test_vec_add_basic(self):
        assert bmb_compute.vec_add([1, 2, 3], [4, 5, 6]) == [5, 7, 9]

    def test_vec_sub_basic(self):
        assert bmb_compute.vec_sub([10, 20], [3, 7]) == [7, 13]

    def test_vec_scale_basic(self):
        assert bmb_compute.vec_scale([1, 2, 3], 5) == [5, 10, 15]

    def test_vec_scale_zero(self):
        assert bmb_compute.vec_scale([1, 2, 3], 0) == [0, 0, 0]

    def test_map_square_basic(self):
        assert bmb_compute.map_square([1, 2, 3, 4]) == [1, 4, 9, 16]

    def test_moving_avg_scaled_basic(self):
        assert bmb_compute.moving_avg_scaled([10, 20, 30, 40, 50], 3) == [20000, 30000, 40000]

    def test_moving_avg_scaled_k1(self):
        assert bmb_compute.moving_avg_scaled([10, 20, 30], 1) == [10000, 20000, 30000]
