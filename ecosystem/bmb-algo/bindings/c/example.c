/**
 * bmb-algo C bindings — example usage
 *
 * Build:
 *   Windows: gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_algo.dll
 *   Linux:   gcc -O2 -I../../include -o example example.c -L../.. -lbmb_algo -Wl,-rpath,../..
 *   macOS:   gcc -O2 -I../../include -o example example.c -L../.. -lbmb_algo -Wl,-rpath,../..
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "bmb_algo.h"

/* Helper: check FFI error and abort if set */
static void check_err(const char *ctx) {
    if (bmb_ffi_has_error()) {
        fprintf(stderr, "BMB FFI error in %s: %s\n", ctx, bmb_ffi_error_message());
        exit(1);
    }
}

int main(void) {
    /* ── Math ─────────────────────────────────────────────────── */
    bmb_ffi_begin();
    printf("gcd(48, 18)       = %lld\n", (long long)bmb_gcd(48, 18));
    printf("lcm(4, 6)         = %lld\n", (long long)bmb_lcm(4, 6));
    printf("fibonacci(10)     = %lld\n", (long long)bmb_fibonacci(10));
    printf("prime_count(100)  = %lld\n", (long long)bmb_prime_count(100));
    printf("modpow(2,10,1000) = %lld\n", (long long)bmb_modpow(2, 10, 1000));
    printf("is_prime(97)      = %lld\n", (long long)bmb_algo_is_prime(97));
    printf("power_set_size(4) = %lld\n", (long long)bmb_power_set_size(4));
    printf("nqueens(8)        = %lld\n", (long long)bmb_nqueens(8));
    check_err("math");
    bmb_ffi_end();

    /* ── Number theory ────────────────────────────────────────── */
    bmb_ffi_begin();
    printf("digit_sum(12345)       = %lld\n", (long long)bmb_digit_sum(12345));
    printf("is_palindrome(121)     = %lld\n", (long long)bmb_is_palindrome_num(121));
    printf("is_palindrome(123)     = %lld\n", (long long)bmb_is_palindrome_num(123));
    check_err("number theory");
    bmb_ffi_end();

    /* ── Bit ops ──────────────────────────────────────────────── */
    bmb_ffi_begin();
    int64_t v = 0;
    v = bmb_bit_set(v, 3);    /* set bit 3 → 8 */
    v = bmb_bit_set(v, 1);    /* set bit 1 → 10 */
    printf("bit_set(0,3)|bit_set(1) = %lld\n", (long long)v);
    printf("bit_test(10, 3)         = %lld\n", (long long)bmb_bit_test(v, 3));
    printf("bit_popcount(255)       = %lld\n", (long long)bmb_bit_popcount(255));
    check_err("bit ops");
    bmb_ffi_end();

    /* ── Array ops ────────────────────────────────────────────── */
    int64_t arr[] = {5, 3, 8, 1, 9, 2, 7, 4, 6};
    int64_t n = 9;

    bmb_ffi_begin();
    printf("array_sum([5..])        = %lld\n", (long long)bmb_array_sum((int64_t)arr, n));
    printf("array_min([5..])        = %lld\n", (long long)bmb_array_min((int64_t)arr, n));
    printf("array_max([5..])        = %lld\n", (long long)bmb_array_max((int64_t)arr, n));
    printf("array_contains(8)       = %lld\n", (long long)bmb_array_contains((int64_t)arr, n, 8));
    printf("array_index_of(7)       = %lld\n", (long long)bmb_array_index_of((int64_t)arr, n, 7));
    check_err("array ops");
    bmb_ffi_end();

    /* ── Sorting ──────────────────────────────────────────────── */
    int64_t to_sort[] = {5, 3, 8, 1, 9, 2, 7, 4, 6};
    bmb_ffi_begin();
    bmb_quicksort((int64_t)to_sort, n);
    check_err("quicksort");
    bmb_ffi_end();

    printf("quicksort result:       ");
    for (int64_t i = 0; i < n; i++) printf("%lld ", (long long)to_sort[i]);
    printf("\n");

    /* ── Search & DP ──────────────────────────────────────────── */
    bmb_ffi_begin();
    printf("binary_search(sorted,7) = %lld\n", (long long)bmb_binary_search((int64_t)to_sort, n, 7));
    printf("kth_smallest(orig,3)    = %lld\n", (long long)bmb_kth_smallest((int64_t)arr, n, 3));

    int64_t coins[] = {1, 5, 10, 25};
    printf("coin_change([1,5,10,25],36) = %lld\n", (long long)bmb_coin_change((int64_t)coins, 4, 36));

    int64_t seq[] = {3, 1, 4, 1, 5, 9, 2, 6};
    printf("lis([3,1,4,1,5,9,2,6]) = %lld\n", (long long)bmb_lis((int64_t)seq, 8));

    int64_t sub[] = {3, 1, 4, 1, 5, 9};
    printf("max_subarray([3,1,4,1,5,9]) = %lld\n", (long long)bmb_max_subarray((int64_t)sub, 6));
    check_err("search & dp");
    bmb_ffi_end();

    /* ── String ops via BMB FFI ───────────────────────────────── */
    bmb_ffi_begin();
    void *s1 = bmb_ffi_cstr_to_string("ABCBDAB");
    void *s2 = bmb_ffi_cstr_to_string("BDCAB");
    printf("lcs(ABCBDAB, BDCAB)     = %lld\n", (long long)bmb_lcs(s1, s2));
    void *s3 = bmb_ffi_cstr_to_string("kitten");
    void *s4 = bmb_ffi_cstr_to_string("sitting");
    printf("edit_dist(kitten,sitting)= %lld\n", (long long)bmb_edit_distance(s3, s4));
    void *hs = bmb_ffi_cstr_to_string("hello");
    printf("djb2_hash(hello)        = %lld\n", (long long)bmb_djb2_hash(hs));
    bmb_ffi_free_string(s1); bmb_ffi_free_string(s2);
    bmb_ffi_free_string(s3); bmb_ffi_free_string(s4);
    bmb_ffi_free_string(hs);
    check_err("string ops");
    bmb_ffi_end();

    printf("All examples passed.\n");
    return 0;
}
