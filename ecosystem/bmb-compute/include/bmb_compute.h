/**
 * bmb_compute.h — Numeric computation
 *
 * Auto-generated from BMB source. Do not edit manually.
 * Generated: 2026-03-23
 *
 * Usage:
 *   #include "bmb_compute.h"
 *   // Link with bmb_compute.dll / libbmb_compute.so / libbmb_compute.dylib
 */

#ifndef BMB_COMPUTE_H
#define BMB_COMPUTE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* FFI Safety API */
int bmb_ffi_begin(void);
void bmb_ffi_end(void);
int bmb_ffi_has_error(void);
const char* bmb_ffi_error_message(void);

/* String FFI API */
void* bmb_ffi_cstr_to_string(const char* s);
const char* bmb_ffi_string_data(void* s);
int64_t bmb_ffi_string_len(void* s);
void bmb_ffi_free_string(void* s);

/* Numeric computation — 33 functions */

int64_t bmb_c_abs(int64_t x);

int64_t bmb_c_min(int64_t a, int64_t b);

int64_t bmb_c_max(int64_t a, int64_t b);

int64_t bmb_c_clamp(int64_t x, int64_t lo, int64_t hi);

int64_t bmb_sign(int64_t x);

/** Integer power (base^exp) */
int64_t bmb_ipow(int64_t base, int64_t exp);

/** Integer square root (Newton's method) */
int64_t bmb_sqrt(int64_t n);

/** Factorial */
int64_t bmb_factorial(int64_t n);

/** Sum of array elements */
int64_t bmb_sum(int64_t arr, int64_t n);

/** Mean × 1000 (3 decimal places in integer arithmetic) */
int64_t bmb_mean_scaled(int64_t arr, int64_t n);

/** Minimum value in array */
int64_t bmb_c_min_val(int64_t arr, int64_t n);

/** Maximum value in array */
int64_t bmb_c_max_val(int64_t arr, int64_t n);

/** Range (max - min) */
int64_t bmb_range_val(int64_t arr, int64_t n);

/** Variance × 1000000 (6 decimal places) */
int64_t bmb_variance_scaled(int64_t arr, int64_t n);

/** Initialize seed (ensure nonzero) */
int64_t bmb_rand_seed(int64_t seed);

/** Next random state */
int64_t bmb_rand_next(int64_t state);

/** Random positive value */
int64_t bmb_rand_pos(int64_t state);

/** Random in range [0, max) */
int64_t bmb_rand_range(int64_t state, int64_t max_val);

int64_t bmb_dot_product(int64_t a, int64_t b, int64_t n);

int64_t bmb_dist_squared(int64_t a, int64_t b, int64_t n);

int64_t bmb_weighted_sum(int64_t values, int64_t weights, int64_t n);

int64_t bmb_array_copy(int64_t src, int64_t dst, int64_t n);

int64_t bmb_lerp_scaled(int64_t a, int64_t b, int64_t t);

int64_t bmb_c_is_power_of_two(int64_t n);

int64_t bmb_c_next_power_of_two(int64_t n);

/** Caller must pass a sorted array */
int64_t bmb_median_scaled(int64_t arr, int64_t n);

/** Cumulative sum — writes prefix sums to output array */
int64_t bmb_cumsum(int64_t arr, int64_t out, int64_t n);

/** Moving average (scaled x 1000) with window size k */
int64_t bmb_moving_avg_scaled(int64_t arr, int64_t out, int64_t n, int64_t k);

/** Vector magnitude squared (sum of squares) */
int64_t bmb_magnitude_squared(int64_t arr, int64_t n);

/** Element-wise addition: out[i] = a[i] + b[i] */
int64_t bmb_vec_add(int64_t a, int64_t b, int64_t out, int64_t n);

/** Element-wise subtraction: out[i] = a[i] - b[i] */
int64_t bmb_vec_sub(int64_t a, int64_t b, int64_t out, int64_t n);

/** Scalar multiplication: out[i] = arr[i] * scalar */
int64_t bmb_vec_scale(int64_t arr, int64_t scalar, int64_t out, int64_t n);

/** Map (square): out[i] = arr[i]^2 */
int64_t bmb_map_square(int64_t arr, int64_t out, int64_t n);

#ifdef __cplusplus
}
#endif

#endif /* BMB_COMPUTE_H */