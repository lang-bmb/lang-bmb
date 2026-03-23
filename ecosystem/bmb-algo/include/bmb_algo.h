/**
 * bmb_algo.h — High-performance algorithms
 *
 * Auto-generated from BMB source. Do not edit manually.
 * Generated: 2026-03-23
 *
 * Usage:
 *   #include "bmb_algo.h"
 *   // Link with bmb_algo.dll / libbmb_algo.so / libbmb_algo.dylib
 */

#ifndef BMB_ALGO_H
#define BMB_ALGO_H

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

/* High-performance algorithms — 55 functions */

/** Returns: maximum total value */
int64_t bmb_knapsack(int64_t weights, int64_t values, int64_t n, int64_t capacity);

/** Returns: LCS length */
int64_t bmb_lcs(void* a, void* b);

/** Modifies dist in-place with shortest distances */
int64_t bmb_floyd_warshall(int64_t dist, int64_t n);

int64_t bmb_edit_distance(void* a, void* b);

/** Returns: maximum subarray sum */
int64_t bmb_max_subarray(int64_t arr, int64_t n);

/** Returns: minimum coins, or -1 if impossible */
int64_t bmb_coin_change(int64_t coins, int64_t n_coins, int64_t amount);

/** Returns: LIS length */
int64_t bmb_lis(int64_t arr, int64_t n);

/** result: output array (n i64s, filled with shortest distances) */
int64_t bmb_dijkstra(int64_t adj, int64_t n, int64_t source, int64_t result);

/** arr: pointer to i64 array, n: length */
int64_t bmb_quicksort(int64_t arr, int64_t n);

/** source: starting node */
int64_t bmb_bfs_count(int64_t adj, int64_t n, int64_t source);

/** a, b: input matrices (flattened n*n), c: output matrix (caller-allocated n*n) */
int64_t bmb_matrix_multiply(int64_t a, int64_t b, int64_t c, int64_t n);

/** arr: pointer to i64 array, n: length */
int64_t bmb_merge_sort(int64_t arr, int64_t n);

/** arr: pointer to i64 array, n: length */
int64_t bmb_heap_sort(int64_t arr, int64_t n);

/** arr: pointer to i64 array, n: length */
int64_t bmb_counting_sort(int64_t arr, int64_t n, int64_t max_val);

/** arr: pointer to sorted i64 array, n: length, target: search value */
int64_t bmb_binary_search(int64_t arr, int64_t n, int64_t target);

/** result: output array of n i64s (filled with topological order) */
int64_t bmb_topological_sort(int64_t adj, int64_t n, int64_t result);

/** Greatest common divisor */
int64_t bmb_gcd(int64_t a, int64_t b);

/** Compute n-th Fibonacci number (F(0)=0, F(1)=1) */
int64_t bmb_fibonacci(int64_t n);

/** Count primes up to n (inclusive) */
int64_t bmb_prime_count(int64_t n);

/** Compute (base^exp) mod modulus via fast exponentiation */
int64_t bmb_modpow(int64_t base, int64_t exp, int64_t modulus);

/** Count N-Queens solutions for n×n board */
int64_t bmb_nqueens(int64_t n);

/** DJB2 hash function for strings */
int64_t bmb_djb2_hash(void* s);

int64_t bmb_lcm(int64_t a, int64_t b);

int64_t bmb_power_set_size(int64_t n);

/** Transpose n×n matrix in-place */
int64_t bmb_matrix_transpose(int64_t mat, int64_t n);

int64_t bmb_is_sorted(int64_t arr, int64_t n);

int64_t bmb_array_reverse(int64_t arr, int64_t n);

int64_t bmb_bit_popcount(int64_t x);

int64_t bmb_array_rotate(int64_t arr, int64_t n, int64_t k);

int64_t bmb_unique_count(int64_t arr, int64_t n);

int64_t bmb_prefix_sum(int64_t arr, int64_t n);

int64_t bmb_array_sum(int64_t arr, int64_t n);

int64_t bmb_array_min(int64_t arr, int64_t n);

int64_t bmb_array_max(int64_t arr, int64_t n);

int64_t bmb_bit_set(int64_t value, int64_t pos);

int64_t bmb_bit_clear(int64_t value, int64_t pos);

int64_t bmb_bit_test(int64_t value, int64_t pos);

int64_t bmb_bit_toggle(int64_t value, int64_t pos);

int64_t bmb_array_fill(int64_t arr, int64_t n, int64_t value);

int64_t bmb_array_contains(int64_t arr, int64_t n, int64_t target);

int64_t bmb_array_index_of(int64_t arr, int64_t n, int64_t target);

int64_t bmb_shell_sort(int64_t arr, int64_t n);

int64_t bmb_subset_sum(int64_t arr, int64_t n, int64_t target);

int64_t bmb_matrix_det(int64_t mat, int64_t n);

int64_t bmb_insertion_sort(int64_t arr, int64_t n);

int64_t bmb_is_prime(int64_t n);

int64_t bmb_selection_sort(int64_t arr, int64_t n);

int64_t bmb_bubble_sort(int64_t arr, int64_t n);

int64_t bmb_array_product(int64_t arr, int64_t n);

/** Is Palindrome Number — Check if integer reads same forwards and backwards */
int64_t bmb_is_palindrome_num(int64_t n);

/** Digit Sum — Sum of all digits */
int64_t bmb_digit_sum(int64_t n);

/** Returns the kth smallest element in unsorted array */
int64_t bmb_kth_smallest(int64_t arr, int64_t n, int64_t k);

/** Requires sorted input */
int64_t bmb_array_mode(int64_t arr, int64_t n);

/** Array Intersection Count — Count common elements between two sorted arrays */
int64_t bmb_sorted_intersect_count(int64_t a, int64_t na, int64_t b, int64_t nb);

/** Returns encoded pair: i * 10000 + j (-1 if not found) */
int64_t bmb_two_sum(int64_t arr, int64_t n, int64_t target);

#ifdef __cplusplus
}
#endif

#endif /* BMB_ALGO_H */