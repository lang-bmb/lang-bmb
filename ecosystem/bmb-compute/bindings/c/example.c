/**
 * bmb-compute C bindings — example usage
 *
 * Build:
 *   Windows: gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_compute.dll
 *   Linux:   gcc -O2 -I../../include -o example example.c -L../.. -lbmb_compute -Wl,-rpath,../..
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "bmb_compute.h"

static void check_err(const char *ctx) {
    if (bmb_ffi_has_error()) {
        fprintf(stderr, "BMB FFI error in %s: %s\n", ctx, bmb_ffi_error_message());
        exit(1);
    }
}

int main(void) {
    /* ── Scalar math ─────────────────────────────────────────── */
    bmb_ffi_begin();
    printf("abs(-7)           = %lld\n", (long long)bmb_c_abs(-7));
    printf("min(3, 8)         = %lld\n", (long long)bmb_c_min(3, 8));
    printf("max(3, 8)         = %lld\n", (long long)bmb_c_max(3, 8));
    printf("clamp(15, 0, 10)  = %lld\n", (long long)bmb_c_clamp(15, 0, 10));
    printf("sign(-5)          = %lld\n", (long long)bmb_sign(-5));
    printf("ipow(2, 10)       = %lld\n", (long long)bmb_ipow(2, 10));
    printf("sqrt(144)         = %lld\n", (long long)bmb_sqrt(144));
    printf("factorial(10)     = %lld\n", (long long)bmb_factorial(10));
    printf("is_power_of_two(64) = %lld\n", (long long)bmb_c_is_power_of_two(64));
    printf("next_power_of_two(100) = %lld\n", (long long)bmb_c_next_power_of_two(100));
    printf("lerp_scaled(0,100,500) = %lld\n", (long long)bmb_lerp_scaled(0, 100, 500)); /* 50 (t=0.5*1000) */
    check_err("scalar");
    bmb_ffi_end();

    /* ── Array statistics ────────────────────────────────────── */
    int64_t data[] = {10, 20, 30, 40, 50};
    int64_t n = 5;
    int64_t ptr = (int64_t)data;

    bmb_ffi_begin();
    printf("sum([10..50])          = %lld\n", (long long)bmb_sum(ptr, n));
    printf("mean_scaled([10..50])  = %lld\n", (long long)bmb_mean_scaled(ptr, n)); /* 30000 */
    printf("min_val                = %lld\n", (long long)bmb_c_min_val(ptr, n));
    printf("max_val                = %lld\n", (long long)bmb_c_max_val(ptr, n));
    printf("range_val              = %lld\n", (long long)bmb_range_val(ptr, n));
    printf("variance_scaled        = %lld\n", (long long)bmb_variance_scaled(ptr, n));
    printf("magnitude_squared      = %lld\n", (long long)bmb_magnitude_squared(ptr, n));
    check_err("stats");
    bmb_ffi_end();

    /* median_scaled requires sorted input */
    bmb_ffi_begin();
    printf("median_scaled(sorted)  = %lld\n", (long long)bmb_median_scaled(ptr, n)); /* 30000 */
    check_err("median");
    bmb_ffi_end();

    /* ── Vector operations ───────────────────────────────────── */
    int64_t a[] = {1, 2, 3};
    int64_t b[] = {4, 5, 6};
    int64_t out[3] = {0};

    bmb_ffi_begin();
    printf("dot([1,2,3],[4,5,6])   = %lld\n", (long long)bmb_dot_product((int64_t)a, (int64_t)b, 3));
    printf("dist_sq                = %lld\n", (long long)bmb_dist_squared((int64_t)a, (int64_t)b, 3));
    bmb_vec_add((int64_t)a, (int64_t)b, (int64_t)out, 3);
    printf("vec_add[0]             = %lld\n", (long long)out[0]); /* 5 */
    bmb_vec_scale((int64_t)a, 3, (int64_t)out, 3);
    printf("vec_scale(*3)[0]       = %lld\n", (long long)out[0]); /* 3 */
    check_err("vector");
    bmb_ffi_end();

    /* ── Random number generator ─────────────────────────────── */
    bmb_ffi_begin();
    int64_t state = bmb_rand_seed(42);
    state = bmb_rand_next(state);
    int64_t rval = bmb_rand_range(state, 100);
    printf("rand_range(seed=42, max=100) = %lld\n", (long long)rval);
    check_err("rng");
    bmb_ffi_end();

    /* ── Prefix sums / moving average ───────────────────────── */
    int64_t cs_out[5] = {0};
    bmb_ffi_begin();
    bmb_cumsum(ptr, (int64_t)cs_out, n);
    printf("cumsum[4]              = %lld\n", (long long)cs_out[4]); /* 150 */
    int64_t ma_out[5] = {0};
    bmb_moving_avg_scaled(ptr, (int64_t)ma_out, n, 3);
    printf("moving_avg_scaled[2]   = %lld\n", (long long)ma_out[2]); /* (10+20+30)/3 * 1000 = 20000 */
    check_err("cumsum/mavg");
    bmb_ffi_end();

    printf("All examples passed.\n");
    return 0;
}
