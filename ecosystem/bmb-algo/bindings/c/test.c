/**
 * bmb-algo C bindings — test suite
 *
 * Tests all 55 exported functions. Exit code 0 = all pass.
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "bmb_algo.h"

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

/* ── Math tests ──────────────────────────────────────────────────────── */
static void test_math(void) {
    FFI_BEGIN();
    ASSERT_EQ("gcd(48,18)=6",        bmb_gcd(48, 18),       6);
    ASSERT_EQ("gcd(100,75)=25",      bmb_gcd(100, 75),      25);
    ASSERT_EQ("lcm(4,6)=12",         bmb_lcm(4, 6),         12);
    ASSERT_EQ("fibonacci(0)=0",      bmb_fibonacci(0),       0);
    ASSERT_EQ("fibonacci(1)=1",      bmb_fibonacci(1),       1);
    ASSERT_EQ("fibonacci(10)=55",    bmb_fibonacci(10),      55);
    ASSERT_EQ("prime_count(10)=4",   bmb_prime_count(10),    4);
    ASSERT_EQ("prime_count(100)=25", bmb_prime_count(100),   25);
    ASSERT_EQ("modpow(2,10,1000)=24",bmb_modpow(2, 10, 1000), 24);
    ASSERT_EQ("modpow(3,0,7)=1",     bmb_modpow(3, 0, 7),    1);
    ASSERT_EQ("nqueens(4)=2",        bmb_nqueens(4),         2);
    ASSERT_EQ("nqueens(8)=92",       bmb_nqueens(8),         92);
    ASSERT_EQ("power_set_size(4)=16",bmb_power_set_size(4),  16);
    ASSERT_EQ("is_prime(2)=1",       bmb_algo_is_prime(2),   1);
    ASSERT_EQ("is_prime(97)=1",      bmb_algo_is_prime(97),  1);
    ASSERT_EQ("is_prime(4)=0",       bmb_algo_is_prime(4),   0);
    ASSERT_EQ("digit_sum(12345)=15", bmb_digit_sum(12345),   15);
    ASSERT_EQ("palindrome(121)=1",   bmb_is_palindrome_num(121), 1);
    ASSERT_EQ("palindrome(123)=0",   bmb_is_palindrome_num(123), 0);
    FFI_END_CHECK("math");
}

/* ── Bit ops ─────────────────────────────────────────────────────────── */
static void test_bit_ops(void) {
    FFI_BEGIN();
    ASSERT_EQ("popcount(255)=8",     bmb_bit_popcount(255),  8);
    ASSERT_EQ("popcount(0)=0",       bmb_bit_popcount(0),    0);
    ASSERT_EQ("bit_set(0,3)=8",      bmb_bit_set(0, 3),      8);
    ASSERT_EQ("bit_clear(8,3)=0",    bmb_bit_clear(8, 3),    0);
    ASSERT_EQ("bit_test(8,3)=1",     bmb_bit_test(8, 3),     1);
    ASSERT_EQ("bit_test(8,2)=0",     bmb_bit_test(8, 2),     0);
    ASSERT_EQ("bit_toggle(8,3)=0",   bmb_bit_toggle(8, 3),   0);
    ASSERT_EQ("bit_toggle(0,3)=8",   bmb_bit_toggle(0, 3),   8);
    FFI_END_CHECK("bit_ops");
}

/* ── Array utilities ─────────────────────────────────────────────────── */
static void test_array_utils(void) {
    int64_t a[] = {3, 1, 4, 1, 5, 9, 2, 6};
    int64_t n = 8;
    int64_t ptr = (int64_t)a;

    FFI_BEGIN();
    ASSERT_EQ("array_sum",           bmb_array_sum(ptr, n),   31);
    ASSERT_EQ("array_min",           bmb_array_min(ptr, n),   1);
    ASSERT_EQ("array_max",           bmb_array_max(ptr, n),   9);
    ASSERT_EQ("array_contains(9)",   bmb_array_contains(ptr, n, 9), 1);
    ASSERT_EQ("array_contains(7)",   bmb_array_contains(ptr, n, 7), 0);
    ASSERT_EQ("array_index_of(4)",   bmb_array_index_of(ptr, n, 4), 2);
    ASSERT_EQ("array_index_of(7)",   bmb_array_index_of(ptr, n, 7), -1);
    FFI_END_CHECK("array_utils");

    /* unique_count requires sorted input (adjacent comparison) */
    int64_t sorted_dup[] = {1, 1, 2, 3, 4, 5, 6, 9};
    FFI_BEGIN();
    ASSERT_EQ("unique_count",        bmb_unique_count((int64_t)sorted_dup, 8), 7);

    /* array_fill */
    int64_t fill_arr[5];
    FFI_BEGIN();
    bmb_array_fill((int64_t)fill_arr, 5, 42);
    FFI_END_CHECK("array_fill");
    int ok = 1;
    for (int i = 0; i < 5; i++) ok &= (fill_arr[i] == 42);
    ASSERT_EQ("array_fill(42)", ok, 1);

    /* array_reverse */
    int64_t rev[] = {1, 2, 3, 4, 5};
    FFI_BEGIN();
    bmb_array_reverse((int64_t)rev, 5);
    FFI_END_CHECK("array_reverse");
    ASSERT_EQ("array_reverse[0]=5", rev[0], 5);
    ASSERT_EQ("array_reverse[4]=1", rev[4], 1);

    /* is_sorted */
    int64_t sorted[] = {1, 2, 3, 4, 5};
    int64_t unsorted[] = {5, 4, 3};
    FFI_BEGIN();
    ASSERT_EQ("is_sorted(asc)",  bmb_is_sorted((int64_t)sorted, 5),   1);
    ASSERT_EQ("is_sorted(desc)", bmb_is_sorted((int64_t)unsorted, 3), 0);
    FFI_END_CHECK("is_sorted");

    /* prefix_sum */
    int64_t ps[] = {1, 2, 3, 4, 5};
    FFI_BEGIN();
    bmb_prefix_sum((int64_t)ps, 5);
    FFI_END_CHECK("prefix_sum");
    ASSERT_EQ("prefix_sum[4]=15", ps[4], 15);

    /* array_product */
    int64_t pr[] = {2, 3, 4};
    FFI_BEGIN();
    ASSERT_EQ("array_product", bmb_array_product((int64_t)pr, 3), 24);
    FFI_END_CHECK("array_product");

    /* array_rotate */
    int64_t rot[] = {1, 2, 3, 4, 5};
    FFI_BEGIN();
    bmb_array_rotate((int64_t)rot, 5, 2);
    FFI_END_CHECK("array_rotate");
    ASSERT_EQ("rotate[0]=3", rot[0], 3);
    ASSERT_EQ("rotate[4]=2", rot[4], 2);
}

/* ── Sorting ─────────────────────────────────────────────────────────── */
static void test_sorting(void) {
    /* quicksort */
    int64_t q[] = {5, 3, 8, 1, 9, 2};
    FFI_BEGIN();
    bmb_quicksort((int64_t)q, 6);
    FFI_END_CHECK("quicksort");
    ASSERT_EQ("quicksort[0]=1", q[0], 1);
    ASSERT_EQ("quicksort[5]=9", q[5], 9);

    /* merge_sort */
    int64_t m[] = {5, 3, 8, 1, 9, 2};
    FFI_BEGIN();
    bmb_merge_sort((int64_t)m, 6);
    FFI_END_CHECK("merge_sort");
    ASSERT_EQ("merge_sort[0]=1", m[0], 1);
    ASSERT_EQ("merge_sort[5]=9", m[5], 9);

    /* heap_sort */
    int64_t h[] = {5, 3, 8, 1, 9, 2};
    FFI_BEGIN();
    bmb_heap_sort((int64_t)h, 6);
    FFI_END_CHECK("heap_sort");
    ASSERT_EQ("heap_sort[0]=1", h[0], 1);
    ASSERT_EQ("heap_sort[5]=9", h[5], 9);

    /* counting_sort */
    int64_t c[] = {3, 1, 4, 1, 5, 9, 2, 6};
    FFI_BEGIN();
    bmb_counting_sort((int64_t)c, 8, 9);
    FFI_END_CHECK("counting_sort");
    ASSERT_EQ("counting_sort[0]=1", c[0], 1);
    ASSERT_EQ("counting_sort[7]=9", c[7], 9);

    /* insertion_sort */
    int64_t ins[] = {5, 3, 8, 1, 9, 2};
    FFI_BEGIN();
    bmb_insertion_sort((int64_t)ins, 6);
    FFI_END_CHECK("insertion_sort");
    ASSERT_EQ("insertion_sort[0]=1", ins[0], 1);

    /* selection_sort */
    int64_t sel[] = {5, 3, 8, 1, 9, 2};
    FFI_BEGIN();
    bmb_selection_sort((int64_t)sel, 6);
    FFI_END_CHECK("selection_sort");
    ASSERT_EQ("selection_sort[0]=1", sel[0], 1);

    /* bubble_sort */
    int64_t bub[] = {5, 3, 8, 1, 9, 2};
    FFI_BEGIN();
    bmb_bubble_sort((int64_t)bub, 6);
    FFI_END_CHECK("bubble_sort");
    ASSERT_EQ("bubble_sort[0]=1", bub[0], 1);

    /* shell_sort */
    int64_t sh[] = {5, 3, 8, 1, 9, 2};
    FFI_BEGIN();
    bmb_shell_sort((int64_t)sh, 6);
    FFI_END_CHECK("shell_sort");
    ASSERT_EQ("shell_sort[0]=1", sh[0], 1);
}

/* ── Search & DP ─────────────────────────────────────────────────────── */
static void test_search_dp(void) {
    int64_t sorted[] = {1, 2, 3, 4, 5, 6, 7, 8, 9};
    FFI_BEGIN();
    ASSERT_EQ("binary_search(7)=6",  bmb_binary_search((int64_t)sorted, 9, 7), 6);
    ASSERT_EQ("binary_search(10)=-1",bmb_binary_search((int64_t)sorted, 9, 10), -1);
    FFI_END_CHECK("binary_search");

    /* kth_smallest */
    int64_t arr[] = {7, 2, 1, 6, 5, 3, 4, 8};
    FFI_BEGIN();
    ASSERT_EQ("kth_smallest(k=3)=3", bmb_kth_smallest((int64_t)arr, 8, 3), 3);
    FFI_END_CHECK("kth_smallest");

    /* two_sum: returns i*10000+j */
    int64_t ts[] = {2, 7, 11, 15};
    FFI_BEGIN();
    int64_t r = bmb_two_sum((int64_t)ts, 4, 9);
    FFI_END_CHECK("two_sum");
    ASSERT_EQ("two_sum(9) idx0=0", r / 10000, 0);
    ASSERT_EQ("two_sum(9) idx1=1", r % 10000, 1);

    /* subset_sum */
    int64_t sub[] = {3, 1, 4, 1, 5};
    FFI_BEGIN();
    ASSERT_EQ("subset_sum(9)=1",  bmb_subset_sum((int64_t)sub, 5, 9),  1);
    ASSERT_EQ("subset_sum(20)=0", bmb_subset_sum((int64_t)sub, 5, 20), 0);
    FFI_END_CHECK("subset_sum");

    /* coin_change */
    int64_t coins[] = {1, 5, 10, 25};
    FFI_BEGIN();
    ASSERT_EQ("coin_change(36)=3",  bmb_coin_change((int64_t)coins, 4, 36), 3); /* 25+10+1 */
    ASSERT_EQ("coin_change(11)=2",  bmb_coin_change((int64_t)coins, 4, 11), 2); /* 10+1 */
    FFI_END_CHECK("coin_change");

    /* LIS */
    int64_t lis_arr[] = {3, 1, 4, 1, 5, 9, 2, 6};
    FFI_BEGIN();
    ASSERT_EQ("lis=4", bmb_lis((int64_t)lis_arr, 8), 4); /* 1,4,5,9 or 1,4,5,6 */
    FFI_END_CHECK("lis");

    /* max_subarray */
    int64_t ms_arr[] = {-2, 1, -3, 4, -1, 2, 1, -5, 4};
    FFI_BEGIN();
    ASSERT_EQ("max_subarray=6", bmb_max_subarray((int64_t)ms_arr, 9), 6); /* [4,-1,2,1] */
    FFI_END_CHECK("max_subarray");

    /* sorted_intersect_count */
    int64_t a1[] = {1, 2, 3, 4, 5};
    int64_t a2[] = {3, 4, 5, 6, 7};
    FFI_BEGIN();
    ASSERT_EQ("intersect_count=3", bmb_sorted_intersect_count((int64_t)a1, 5, (int64_t)a2, 5), 3);
    FFI_END_CHECK("sorted_intersect_count");

    /* array_mode (sorted input required) */
    int64_t mode_arr[] = {1, 1, 2, 3, 3, 3, 4};
    FFI_BEGIN();
    ASSERT_EQ("array_mode=3", bmb_array_mode((int64_t)mode_arr, 7), 3);
    FFI_END_CHECK("array_mode");
}

/* ── Matrix ops ──────────────────────────────────────────────────────── */
static void test_matrix(void) {
    /* matrix_transpose 2×2 */
    int64_t mat[] = {1, 2, 3, 4};
    FFI_BEGIN();
    bmb_matrix_transpose((int64_t)mat, 2);
    FFI_END_CHECK("matrix_transpose");
    ASSERT_EQ("transpose[1]=3", mat[1], 3);
    ASSERT_EQ("transpose[2]=2", mat[2], 2);

    /* matrix_multiply 2×2: [[1,2],[3,4]] × [[5,6],[7,8]] */
    int64_t ma[] = {1, 2, 3, 4};
    int64_t mb[] = {5, 6, 7, 8};
    int64_t mc[4] = {0};
    FFI_BEGIN();
    bmb_matrix_multiply((int64_t)ma, (int64_t)mb, (int64_t)mc, 2);
    FFI_END_CHECK("matrix_multiply");
    ASSERT_EQ("mat_mul[0]=19", mc[0], 19);
    ASSERT_EQ("mat_mul[3]=50", mc[3], 50);
}

/* ── String ops ──────────────────────────────────────────────────────── */
static void test_string_ops(void) {
    FFI_BEGIN();
    void *s1 = bmb_ffi_cstr_to_string("ABCBDAB");
    void *s2 = bmb_ffi_cstr_to_string("BDCAB");
    ASSERT_EQ("lcs=4", bmb_lcs(s1, s2), 4);
    bmb_ffi_free_string(s1);
    bmb_ffi_free_string(s2);

    void *s3 = bmb_ffi_cstr_to_string("kitten");
    void *s4 = bmb_ffi_cstr_to_string("sitting");
    ASSERT_EQ("edit_dist=3", bmb_edit_distance(s3, s4), 3);
    bmb_ffi_free_string(s3);
    bmb_ffi_free_string(s4);

    void *hs = bmb_ffi_cstr_to_string("hello");
    int64_t hash = bmb_djb2_hash(hs);
    ASSERT_EQ("djb2_hash!=0", hash != 0, 1);
    bmb_ffi_free_string(hs);

    FFI_END_CHECK("string_ops");
}

int main(void) {
    printf("=== bmb-algo C binding tests ===\n\n");

    test_math();
    test_bit_ops();
    test_array_utils();
    test_sorting();
    test_search_dp();
    test_matrix();
    test_string_ops();

    printf("\n=== Results: %d passed, %d failed ===\n", g_pass, g_fail);
    return g_fail > 0 ? 1 : 0;
}
