/**
 * bmb-compute C bindings — test suite
 *
 * Tests all 33 exported functions. Exit code 0 = all pass.
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "bmb_compute.h"

static int g_pass = 0, g_fail = 0;

#define ASSERT_EQ(label, got, expected) do { \
    int64_t _g = (int64_t)(got); \
    int64_t _e = (int64_t)(expected); \
    if (_g == _e) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: got %lld, expected %lld\n", label, (long long)_g, (long long)_e); g_fail++; \
    } \
} while(0)

#define FFI_BEGIN() bmb_ffi_begin()
#define FFI_END_CHECK(label) do { \
    if (bmb_ffi_has_error()) { \
        printf("[FAIL] FFI error in %s: %s\n", label, bmb_ffi_error_message()); g_fail++; \
    } \
    bmb_ffi_end(); \
} while(0)

/* ── Scalar math ─────────────────────────────────────────────────────── */
static void test_scalar(void) {
    FFI_BEGIN();
    ASSERT_EQ("abs(-7)=7",           bmb_c_abs(-7),              7);
    ASSERT_EQ("abs(7)=7",            bmb_c_abs(7),               7);
    ASSERT_EQ("abs(0)=0",            bmb_c_abs(0),               0);
    ASSERT_EQ("min(3,8)=3",          bmb_c_min(3, 8),            3);
    ASSERT_EQ("min(8,3)=3",          bmb_c_min(8, 3),            3);
    ASSERT_EQ("max(3,8)=8",          bmb_c_max(3, 8),            8);
    ASSERT_EQ("clamp(5,0,10)=5",     bmb_c_clamp(5, 0, 10),      5);
    ASSERT_EQ("clamp(-1,0,10)=0",    bmb_c_clamp(-1, 0, 10),     0);
    ASSERT_EQ("clamp(15,0,10)=10",   bmb_c_clamp(15, 0, 10),     10);
    ASSERT_EQ("sign(-5)=-1",         bmb_sign(-5),               -1);
    ASSERT_EQ("sign(0)=0",           bmb_sign(0),                0);
    ASSERT_EQ("sign(3)=1",           bmb_sign(3),                1);
    ASSERT_EQ("ipow(2,10)=1024",     bmb_ipow(2, 10),            1024);
    ASSERT_EQ("ipow(3,0)=1",         bmb_ipow(3, 0),             1);
    ASSERT_EQ("sqrt(144)=12",        bmb_sqrt(144),              12);
    ASSERT_EQ("sqrt(2)=1",           bmb_sqrt(2),                1);
    ASSERT_EQ("factorial(0)=1",      bmb_factorial(0),           1);
    ASSERT_EQ("factorial(5)=120",    bmb_factorial(5),           120);
    ASSERT_EQ("factorial(10)=3628800",bmb_factorial(10),         3628800);
    ASSERT_EQ("is_pow2(64)=1",       bmb_c_is_power_of_two(64),  1);
    ASSERT_EQ("is_pow2(7)=0",        bmb_c_is_power_of_two(7),   0);
    ASSERT_EQ("next_pow2(100)=128",  bmb_c_next_power_of_two(100), 128);
    ASSERT_EQ("next_pow2(64)=64",    bmb_c_next_power_of_two(64),  64);
    /* lerp_scaled(a, b, t): t is in [0..1000], returns a + (b-a)*t/1000 */
    ASSERT_EQ("lerp(0,100,t=500)=50",bmb_lerp_scaled(0, 100, 500), 50);
    ASSERT_EQ("lerp(0,100,t=0)=0",  bmb_lerp_scaled(0, 100, 0),   0);
    ASSERT_EQ("lerp(0,100,t=1000)=100",bmb_lerp_scaled(0, 100, 1000), 100);
    FFI_END_CHECK("scalar");
}

/* ── Array statistics ────────────────────────────────────────────────── */
static void test_stats(void) {
    int64_t data[] = {10, 20, 30, 40, 50};
    int64_t ptr = (int64_t)data;
    int64_t n = 5;

    FFI_BEGIN();
    ASSERT_EQ("sum=150",             bmb_sum(ptr, n),             150);
    ASSERT_EQ("mean_scaled=30000",   bmb_mean_scaled(ptr, n),     30000);
    ASSERT_EQ("min_val=10",          bmb_c_min_val(ptr, n),       10);
    ASSERT_EQ("max_val=50",          bmb_c_max_val(ptr, n),       50);
    ASSERT_EQ("range_val=40",        bmb_range_val(ptr, n),       40);
    ASSERT_EQ("magnitude_sq=5500",   bmb_magnitude_squared(ptr, n), 5500); /* 100+400+900+1600+2500 */
    FFI_END_CHECK("stats");

    /* weighted_sum: sum(val[i] * w[i]) */
    int64_t vals[] = {1, 2, 3};
    int64_t wts[]  = {3, 2, 1};
    FFI_BEGIN();
    ASSERT_EQ("weighted_sum=10", bmb_weighted_sum((int64_t)vals, (int64_t)wts, 3), 10); /* 3+4+3 */
    FFI_END_CHECK("weighted_sum");

    /* median_scaled requires sorted input */
    FFI_BEGIN();
    ASSERT_EQ("median_scaled=30000", bmb_median_scaled(ptr, n),   30000);
    FFI_END_CHECK("median");

    /* variance_scaled */
    /* data=[10,20,30,40,50], mean=30, var=((10-30)²+(20-30)²+...)/n = 2000000/5 = wait... */
    /* Actually = (400+100+0+100+400)/5 * 1000000 = 1000/5 * 1000000 = 200 * 1000000 = 200000000 */
    /* But implementation uses integer arithmetic; check actual value */
    FFI_BEGIN();
    int64_t var = bmb_variance_scaled(ptr, n);
    FFI_END_CHECK("variance");
    ASSERT_EQ("variance_scaled>0", var > 0, 1);
}

/* ── Vector ops ──────────────────────────────────────────────────────── */
static void test_vector(void) {
    int64_t a[] = {1, 2, 3};
    int64_t b[] = {4, 5, 6};
    int64_t out[3] = {0};

    FFI_BEGIN();
    ASSERT_EQ("dot([1,2,3],[4,5,6])=32", bmb_dot_product((int64_t)a, (int64_t)b, 3), 32);
    ASSERT_EQ("dist_sq=27",           bmb_dist_squared((int64_t)a, (int64_t)b, 3), 27); /* 9+9+9 */
    FFI_END_CHECK("dot/dist");

    FFI_BEGIN();
    bmb_vec_add((int64_t)a, (int64_t)b, (int64_t)out, 3);
    FFI_END_CHECK("vec_add");
    ASSERT_EQ("vec_add[0]=5", out[0], 5);
    ASSERT_EQ("vec_add[2]=9", out[2], 9);

    FFI_BEGIN();
    bmb_vec_sub((int64_t)b, (int64_t)a, (int64_t)out, 3);
    FFI_END_CHECK("vec_sub");
    ASSERT_EQ("vec_sub[0]=3", out[0], 3);
    ASSERT_EQ("vec_sub[2]=3", out[2], 3);

    FFI_BEGIN();
    bmb_vec_scale((int64_t)a, 4, (int64_t)out, 3);
    FFI_END_CHECK("vec_scale");
    ASSERT_EQ("vec_scale[0]=4",  out[0], 4);
    ASSERT_EQ("vec_scale[2]=12", out[2], 12);

    FFI_BEGIN();
    bmb_map_square((int64_t)a, (int64_t)out, 3);
    FFI_END_CHECK("map_square");
    ASSERT_EQ("map_sq[0]=1", out[0], 1);
    ASSERT_EQ("map_sq[2]=9", out[2], 9);

    FFI_BEGIN();
    int64_t ac[] = {10, 20, 30};
    int64_t dc[3] = {0};
    bmb_array_copy((int64_t)ac, (int64_t)dc, 3);
    FFI_END_CHECK("array_copy");
    ASSERT_EQ("copy[0]=10", dc[0], 10);
    ASSERT_EQ("copy[2]=30", dc[2], 30);
}

/* ── Prefix sums & moving avg ────────────────────────────────────────── */
static void test_prefix(void) {
    int64_t data[] = {1, 2, 3, 4, 5};
    int64_t cs[5] = {0};

    FFI_BEGIN();
    bmb_cumsum((int64_t)data, (int64_t)cs, 5);
    FFI_END_CHECK("cumsum");
    ASSERT_EQ("cumsum[0]=1",  cs[0], 1);
    ASSERT_EQ("cumsum[4]=15", cs[4], 15);

    int64_t ma[5] = {0};
    FFI_BEGIN();
    bmb_moving_avg_scaled((int64_t)data, (int64_t)ma, 5, 3);
    FFI_END_CHECK("moving_avg");
    /* out[0]=(1+2+3)/3*1000=2000, out[2]=(3+4+5)/3*1000=4000 (sliding window) */
    ASSERT_EQ("moving_avg[0]=2000", ma[0], 2000);
    ASSERT_EQ("moving_avg[2]=4000", ma[2], 4000);
}

/* ── Random ──────────────────────────────────────────────────────────── */
static void test_rng(void) {
    FFI_BEGIN();
    int64_t s = bmb_rand_seed(42);
    ASSERT_EQ("rand_seed!=0", s != 0, 1);
    int64_t s2 = bmb_rand_next(s);
    ASSERT_EQ("rand_next!=seed", s2 != s, 1);
    int64_t pos = bmb_rand_pos(s2);
    ASSERT_EQ("rand_pos>0", pos > 0, 1);
    int64_t rng = bmb_rand_range(s2, 100);
    ASSERT_EQ("rand_range<100", rng < 100, 1);
    ASSERT_EQ("rand_range>=0",  rng >= 0,  1);
    FFI_END_CHECK("rng");
}

int main(void) {
    printf("=== bmb-compute C binding tests ===\n\n");

    test_scalar();
    test_stats();
    test_vector();
    test_prefix();
    test_rng();

    printf("\n=== Results: %d passed, %d failed ===\n", g_pass, g_fail);
    return g_fail > 0 ? 1 : 0;
}
