package io.bmb.algo;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

/** Raw JNA interface — direct 1:1 mapping of the bmb_algo native C ABI. */
interface BmbAlgoLib extends Library {
    BmbAlgoLib INSTANCE = Native.load("bmb_algo", BmbAlgoLib.class);

    // FFI Safety API
    int     bmb_ffi_begin();
    void    bmb_ffi_end();
    int     bmb_ffi_has_error();
    Pointer bmb_ffi_error_message();

    // String FFI API — BmbString* is opaque from Java
    Pointer bmb_ffi_cstr_to_string(String s);
    Pointer bmb_ffi_string_data(Pointer s);
    void    bmb_ffi_free_string(Pointer s);

    // Scalar functions
    long bmb_gcd(long a, long b);
    long bmb_lcm(long a, long b);
    long bmb_fibonacci(long n);
    long bmb_prime_count(long n);
    long bmb_modpow(long base, long exp, long modulus);
    long bmb_nqueens(long n);
    long bmb_algo_is_prime(long n);
    long bmb_is_palindrome_num(long n);
    long bmb_digit_sum(long n);
    long bmb_bit_popcount(long x);
    long bmb_bit_set(long value, long pos);
    long bmb_bit_clear(long value, long pos);
    long bmb_bit_test(long value, long pos);
    long bmb_bit_toggle(long value, long pos);
    long bmb_power_set_size(long n);

    // Array functions (pointer = native long[] buffer)
    long bmb_array_sum(Pointer arr, long n);
    long bmb_array_min(Pointer arr, long n);
    long bmb_array_max(Pointer arr, long n);
    long bmb_array_product(Pointer arr, long n);
    long bmb_is_sorted(Pointer arr, long n);
    long bmb_binary_search(Pointer arr, long n, long target);
    long bmb_max_subarray(Pointer arr, long n);
    long bmb_lis(Pointer arr, long n);
    long bmb_coin_change(Pointer coins, long nCoins, long amount);
    long bmb_knapsack(Pointer weights, Pointer values, long n, long capacity);
    long bmb_quicksort(Pointer arr, long n);
    long bmb_merge_sort(Pointer arr, long n);
    long bmb_heap_sort(Pointer arr, long n);
    long bmb_insertion_sort(Pointer arr, long n);
    long bmb_selection_sort(Pointer arr, long n);
    long bmb_bubble_sort(Pointer arr, long n);
    long bmb_shell_sort(Pointer arr, long n);
    long bmb_counting_sort(Pointer arr, long n, long maxVal);
    long bmb_array_reverse(Pointer arr, long n);
    long bmb_array_rotate(Pointer arr, long n, long k);
    long bmb_unique_count(Pointer arr, long n);
    long bmb_prefix_sum(Pointer arr, long n);
    long bmb_array_fill(Pointer arr, long n, long value);
    long bmb_array_contains(Pointer arr, long n, long target);
    long bmb_array_index_of(Pointer arr, long n, long target);
    long bmb_subset_sum(Pointer arr, long n, long target);
    long bmb_kth_smallest(Pointer arr, long n, long k);
    long bmb_array_mode(Pointer arr, long n);
    long bmb_two_sum(Pointer arr, long n, long target);

    // String-argument functions
    long bmb_lcs(Pointer a, Pointer b);
    long bmb_edit_distance(Pointer a, Pointer b);
    long bmb_djb2_hash(Pointer s);
}
