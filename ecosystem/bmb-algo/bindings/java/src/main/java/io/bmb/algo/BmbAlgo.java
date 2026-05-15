package io.bmb.algo;

import com.sun.jna.Memory;
import com.sun.jna.Pointer;

import java.util.function.Function;
import java.util.function.Supplier;

/**
 * High-performance algorithms powered by BMB — JNA bindings.
 * All calls dispatch into the native bmb_algo shared library.
 *
 * Thread safety: each public method wraps calls in bmb_ffi_begin/end.
 * Do not share BmbString* pointers across threads.
 */
public final class BmbAlgo {
    private static final BmbAlgoLib LIB = BmbAlgoLib.INSTANCE;

    private BmbAlgo() {}

    // ── Helpers ────────────────────────────────────────────────────────────────

    private static <T> T safe(Supplier<T> fn) {
        LIB.bmb_ffi_begin();
        try {
            T r = fn.get();
            if (LIB.bmb_ffi_has_error() != 0) {
                String msg = LIB.bmb_ffi_error_message().getString(0);
                throw new RuntimeException("BMB FFI error: " + msg);
            }
            return r;
        } finally {
            LIB.bmb_ffi_end();
        }
    }

    /** Copy Java long[] into a native memory buffer (8 bytes per element). */
    private static Memory toNative(long[] arr) {
        Memory mem = new Memory((long) arr.length * Long.BYTES);
        for (int i = 0; i < arr.length; i++) {
            mem.setLong((long) i * Long.BYTES, arr[i]);
        }
        return mem;
    }

    /** Read back a sorted native buffer into a new Java long[]. */
    private static long[] fromNative(Memory mem, int len) {
        long[] arr = new long[len];
        for (int i = 0; i < len; i++) {
            arr[i] = mem.getLong((long) i * Long.BYTES);
        }
        return arr;
    }

    /** Copy arr to native, sort in-place via BMB, return sorted copy. */
    private static long[] sortCopy(long[] arr, Function<Memory, Long> sortFn) {
        Memory m = toNative(arr);
        safe(() -> sortFn.apply(m));
        return fromNative(m, arr.length);
    }

    // ── Scalar functions ───────────────────────────────────────────────────────

    public static long gcd(long a, long b)                    { return safe(() -> LIB.bmb_gcd(a, b)); }
    public static long lcm(long a, long b)                    { return safe(() -> LIB.bmb_lcm(a, b)); }
    public static long fibonacci(long n)                      { return safe(() -> LIB.bmb_fibonacci(n)); }
    public static long primeCount(long n)                     { return safe(() -> LIB.bmb_prime_count(n)); }
    public static long modPow(long base, long exp, long mod)  { return safe(() -> LIB.bmb_modpow(base, exp, mod)); }
    public static long nQueens(long n)                        { return safe(() -> LIB.bmb_nqueens(n)); }
    public static boolean isPrime(long n)                     { return safe(() -> LIB.bmb_algo_is_prime(n)) != 0; }
    public static boolean isPalindromeNum(long n)             { return safe(() -> LIB.bmb_is_palindrome_num(n)) != 0; }
    public static long digitSum(long n)                       { return safe(() -> LIB.bmb_digit_sum(n)); }
    public static long bitPopcount(long x)                    { return safe(() -> LIB.bmb_bit_popcount(x)); }
    public static long powerSetSize(long n)                   { return safe(() -> LIB.bmb_power_set_size(n)); }
    public static long bitSet(long v, long pos)               { return safe(() -> LIB.bmb_bit_set(v, pos)); }
    public static long bitClear(long v, long pos)             { return safe(() -> LIB.bmb_bit_clear(v, pos)); }
    public static boolean bitTest(long v, long pos)           { return safe(() -> LIB.bmb_bit_test(v, pos)) != 0; }
    public static long bitToggle(long v, long pos)            { return safe(() -> LIB.bmb_bit_toggle(v, pos)); }

    // ── Read-only array functions ──────────────────────────────────────────────

    public static long arraySum(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_array_sum(m, arr.length));
    }
    public static long arrayMin(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_array_min(m, arr.length));
    }
    public static long arrayMax(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_array_max(m, arr.length));
    }
    public static long arrayProduct(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_array_product(m, arr.length));
    }
    public static boolean isSorted(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_is_sorted(m, arr.length)) != 0;
    }
    public static long binarySearch(long[] sortedArr, long target) {
        Memory m = toNative(sortedArr);
        return safe(() -> LIB.bmb_binary_search(m, sortedArr.length, target));
    }
    public static long maxSubarray(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_max_subarray(m, arr.length));
    }
    public static long lis(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_lis(m, arr.length));
    }
    public static long coinChange(long[] coins, long amount) {
        Memory m = toNative(coins);
        return safe(() -> LIB.bmb_coin_change(m, coins.length, amount));
    }
    public static long knapsack(long[] weights, long[] values, long capacity) {
        Memory mw = toNative(weights);
        Memory mv = toNative(values);
        return safe(() -> LIB.bmb_knapsack(mw, mv, weights.length, capacity));
    }
    public static boolean subsetSum(long[] arr, long target) {
        Memory m = toNative(arr);
        return safe(() -> LIB.bmb_subset_sum(m, arr.length, target)) != 0;
    }
    public static long uniqueCount(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_unique_count(m, arr.length));
    }
    public static boolean arrayContains(long[] arr, long target) {
        Memory m = toNative(arr);
        return safe(() -> LIB.bmb_array_contains(m, arr.length, target)) != 0;
    }
    public static long arrayIndexOf(long[] arr, long target) {
        Memory m = toNative(arr);
        return safe(() -> LIB.bmb_array_index_of(m, arr.length, target));
    }
    public static long kthSmallest(long[] arr, long k) {
        Memory m = toNative(arr);
        return safe(() -> LIB.bmb_kth_smallest(m, arr.length, k));
    }
    public static long arrayMode(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_array_mode(m, arr.length));
    }

    /**
     * Two-sum: returns [i, j] index pair where arr[i]+arr[j]==target, or [-1,-1].
     * Encoded as i*10000+j by the native function.
     */
    public static long[] twoSum(long[] arr, long target) {
        Memory m = toNative(arr);
        long r = safe(() -> LIB.bmb_two_sum(m, arr.length, target));
        return r < 0 ? new long[]{-1, -1} : new long[]{r / 10000, r % 10000};
    }

    // ── Sorting (returns a sorted copy — BMB sorts the native buffer in-place) ─

    public static long[] quickSort(long[] arr)     { return sortCopy(arr, m -> LIB.bmb_quicksort(m, arr.length)); }
    public static long[] mergeSort(long[] arr)     { return sortCopy(arr, m -> LIB.bmb_merge_sort(m, arr.length)); }
    public static long[] heapSort(long[] arr)      { return sortCopy(arr, m -> LIB.bmb_heap_sort(m, arr.length)); }
    public static long[] insertionSort(long[] arr) { return sortCopy(arr, m -> LIB.bmb_insertion_sort(m, arr.length)); }
    public static long[] selectionSort(long[] arr) { return sortCopy(arr, m -> LIB.bmb_selection_sort(m, arr.length)); }
    public static long[] bubbleSort(long[] arr)    { return sortCopy(arr, m -> LIB.bmb_bubble_sort(m, arr.length)); }
    public static long[] shellSort(long[] arr)     { return sortCopy(arr, m -> LIB.bmb_shell_sort(m, arr.length)); }
    public static long[] countingSort(long[] arr, long maxVal) {
        return sortCopy(arr, m -> LIB.bmb_counting_sort(m, arr.length, maxVal));
    }

    // ── String-argument functions ──────────────────────────────────────────────

    /** Longest Common Subsequence length. */
    public static long lcs(String a, String b) {
        Pointer pa = LIB.bmb_ffi_cstr_to_string(a);
        Pointer pb = LIB.bmb_ffi_cstr_to_string(b);
        try { return safe(() -> LIB.bmb_lcs(pa, pb)); }
        finally { LIB.bmb_ffi_free_string(pa); LIB.bmb_ffi_free_string(pb); }
    }

    /** Levenshtein edit distance. */
    public static long editDistance(String a, String b) {
        Pointer pa = LIB.bmb_ffi_cstr_to_string(a);
        Pointer pb = LIB.bmb_ffi_cstr_to_string(b);
        try { return safe(() -> LIB.bmb_edit_distance(pa, pb)); }
        finally { LIB.bmb_ffi_free_string(pa); LIB.bmb_ffi_free_string(pb); }
    }

    /** DJB2 hash of a string. */
    public static long djb2Hash(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_djb2_hash(ps)); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }
}
