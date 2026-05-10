using System.Runtime.InteropServices;

namespace BmbAlgo;

/// <summary>
/// High-performance algorithms powered by BMB — P/Invoke bindings.
/// All functions call into the native bmb_algo shared library.
/// </summary>
public static class Algo
{
    private const string LibName = "bmb_algo";

    // ── FFI Safety API ───────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern int    bmb_ffi_begin();
    [DllImport(LibName)] private static extern void   bmb_ffi_end();
    [DllImport(LibName)] private static extern int    bmb_ffi_has_error();
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_error_message();

    // ── String FFI API ───────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_cstr_to_string([MarshalAs(UnmanagedType.LPStr)] string s);
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_string_data(IntPtr s);
    [DllImport(LibName)] private static extern void   bmb_ffi_free_string(IntPtr s);

    // ── Scalar functions ─────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern long bmb_gcd(long a, long b);
    [DllImport(LibName)] private static extern long bmb_lcm(long a, long b);
    [DllImport(LibName)] private static extern long bmb_fibonacci(long n);
    [DllImport(LibName)] private static extern long bmb_prime_count(long n);
    [DllImport(LibName)] private static extern long bmb_modpow(long @base, long exp, long modulus);
    [DllImport(LibName)] private static extern long bmb_nqueens(long n);
    [DllImport(LibName)] private static extern long bmb_algo_is_prime(long n);
    [DllImport(LibName)] private static extern long bmb_is_palindrome_num(long n);
    [DllImport(LibName)] private static extern long bmb_digit_sum(long n);
    [DllImport(LibName)] private static extern long bmb_bit_popcount(long x);
    [DllImport(LibName)] private static extern long bmb_bit_set(long value, long pos);
    [DllImport(LibName)] private static extern long bmb_bit_clear(long value, long pos);
    [DllImport(LibName)] private static extern long bmb_bit_test(long value, long pos);
    [DllImport(LibName)] private static extern long bmb_bit_toggle(long value, long pos);
    [DllImport(LibName)] private static extern long bmb_power_set_size(long n);

    // ── Array functions (pointer args) ───────────────────────────────────────
    [DllImport(LibName)] private static extern long bmb_array_sum(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_array_min(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_array_max(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_array_product(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_is_sorted(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_binary_search(IntPtr arr, long n, long target);
    [DllImport(LibName)] private static extern long bmb_max_subarray(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_lis(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_coin_change(IntPtr coins, long n_coins, long amount);
    [DllImport(LibName)] private static extern long bmb_knapsack(IntPtr weights, IntPtr values, long n, long capacity);
    [DllImport(LibName)] private static extern long bmb_quicksort(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_merge_sort(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_heap_sort(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_insertion_sort(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_selection_sort(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_bubble_sort(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_shell_sort(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_counting_sort(IntPtr arr, long n, long max_val);
    [DllImport(LibName)] private static extern long bmb_array_reverse(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_array_rotate(IntPtr arr, long n, long k);
    [DllImport(LibName)] private static extern long bmb_unique_count(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_prefix_sum(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_array_fill(IntPtr arr, long n, long value);
    [DllImport(LibName)] private static extern long bmb_array_contains(IntPtr arr, long n, long target);
    [DllImport(LibName)] private static extern long bmb_array_index_of(IntPtr arr, long n, long target);
    [DllImport(LibName)] private static extern long bmb_subset_sum(IntPtr arr, long n, long target);
    [DllImport(LibName)] private static extern long bmb_kth_smallest(IntPtr arr, long n, long k);
    [DllImport(LibName)] private static extern long bmb_array_mode(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_two_sum(IntPtr arr, long n, long target);

    // ── String-argument functions ────────────────────────────────────────────
    [DllImport(LibName)] private static extern long bmb_lcs(IntPtr a, IntPtr b);
    [DllImport(LibName)] private static extern long bmb_edit_distance(IntPtr a, IntPtr b);
    [DllImport(LibName)] private static extern long bmb_djb2_hash(IntPtr s);

    // ── Helpers ──────────────────────────────────────────────────────────────

    private static T Safe<T>(Func<T> fn)
    {
        bmb_ffi_begin();
        try
        {
            T r = fn();
            if (bmb_ffi_has_error() != 0)
                throw new InvalidOperationException(
                    Marshal.PtrToStringAnsi(bmb_ffi_error_message()) ?? "BMB FFI error");
            return r;
        }
        finally
        {
            bmb_ffi_end();
        }
    }

    private static T WithPinned<T>(long[] arr, Func<IntPtr, T> fn)
    {
        var h = GCHandle.Alloc(arr, GCHandleType.Pinned);
        try { return fn(h.AddrOfPinnedObject()); }
        finally { h.Free(); }
    }

    private static T WithPinned2<T>(long[] a, long[] b, Func<IntPtr, IntPtr, T> fn)
    {
        var ha = GCHandle.Alloc(a, GCHandleType.Pinned);
        var hb = GCHandle.Alloc(b, GCHandleType.Pinned);
        try { return fn(ha.AddrOfPinnedObject(), hb.AddrOfPinnedObject()); }
        finally { ha.Free(); hb.Free(); }
    }

    private static T WithBmbString<T>(string s, Func<IntPtr, T> fn)
    {
        IntPtr p = bmb_ffi_cstr_to_string(s);
        try { return fn(p); }
        finally { bmb_ffi_free_string(p); }
    }

    private static T WithBmbStrings<T>(string a, string b, Func<IntPtr, IntPtr, T> fn)
    {
        IntPtr pa = bmb_ffi_cstr_to_string(a);
        IntPtr pb = bmb_ffi_cstr_to_string(b);
        try { return fn(pa, pb); }
        finally { bmb_ffi_free_string(pa); bmb_ffi_free_string(pb); }
    }

    // ── Public API ───────────────────────────────────────────────────────────

    /// <summary>Greatest common divisor.</summary>
    public static long Gcd(long a, long b) => Safe(() => bmb_gcd(a, b));

    /// <summary>Least common multiple.</summary>
    public static long Lcm(long a, long b) => Safe(() => bmb_lcm(a, b));

    /// <summary>n-th Fibonacci number (0-indexed, F(0)=0).</summary>
    public static long Fibonacci(long n) => Safe(() => bmb_fibonacci(n));

    /// <summary>Count primes up to n (Sieve of Eratosthenes).</summary>
    public static long PrimeCount(long n) => Safe(() => bmb_prime_count(n));

    /// <summary>Modular exponentiation: (base^exp) % mod.</summary>
    public static long ModPow(long @base, long exp, long mod) => Safe(() => bmb_modpow(@base, exp, mod));

    /// <summary>Count N-Queens solutions for an n×n board.</summary>
    public static long NQueens(long n) => Safe(() => bmb_nqueens(n));

    /// <summary>Primality test.</summary>
    public static bool IsPrime(long n) => Safe(() => bmb_algo_is_prime(n)) != 0;

    /// <summary>Is palindrome number (reads same forwards and backwards)?</summary>
    public static bool IsPalindromeNum(long n) => Safe(() => bmb_is_palindrome_num(n)) != 0;

    /// <summary>Sum of decimal digits.</summary>
    public static long DigitSum(long n) => Safe(() => bmb_digit_sum(n));

    /// <summary>Population count (number of set bits).</summary>
    public static long BitPopcount(long x) => Safe(() => bmb_bit_popcount(x));

    /// <summary>Size of power set: 2^n.</summary>
    public static long PowerSetSize(long n) => Safe(() => bmb_power_set_size(n));

    // Bit operations
    public static long BitSet(long value, long pos)    => Safe(() => bmb_bit_set(value, pos));
    public static long BitClear(long value, long pos)  => Safe(() => bmb_bit_clear(value, pos));
    public static bool BitTest(long value, long pos)   => Safe(() => bmb_bit_test(value, pos)) != 0;
    public static long BitToggle(long value, long pos) => Safe(() => bmb_bit_toggle(value, pos));

    // Array operations (read-only)
    public static long ArraySum(long[] arr)     => WithPinned(arr, p => Safe(() => bmb_array_sum(p, arr.Length)));
    public static long ArrayMin(long[] arr)     => WithPinned(arr, p => Safe(() => bmb_array_min(p, arr.Length)));
    public static long ArrayMax(long[] arr)     => WithPinned(arr, p => Safe(() => bmb_array_max(p, arr.Length)));
    public static long ArrayProduct(long[] arr) => WithPinned(arr, p => Safe(() => bmb_array_product(p, arr.Length)));
    public static bool IsSorted(long[] arr)     => WithPinned(arr, p => Safe(() => bmb_is_sorted(p, arr.Length))) != 0;
    public static long BinarySearch(long[] sortedArr, long target) =>
        WithPinned(sortedArr, p => Safe(() => bmb_binary_search(p, sortedArr.Length, target)));
    public static long MaxSubarray(long[] arr)    => WithPinned(arr, p => Safe(() => bmb_max_subarray(p, arr.Length)));
    public static long Lis(long[] arr)            => WithPinned(arr, p => Safe(() => bmb_lis(p, arr.Length)));
    public static long CoinChange(long[] coins, long amount) =>
        WithPinned(coins, p => Safe(() => bmb_coin_change(p, coins.Length, amount)));
    public static bool SubsetSum(long[] arr, long target) =>
        WithPinned(arr, p => Safe(() => bmb_subset_sum(p, arr.Length, target))) != 0;
    public static long UniqueCount(long[] arr)   => WithPinned(arr, p => Safe(() => bmb_unique_count(p, arr.Length)));
    public static bool ArrayContains(long[] arr, long target) =>
        WithPinned(arr, p => Safe(() => bmb_array_contains(p, arr.Length, target))) != 0;
    public static long ArrayIndexOf(long[] arr, long target) =>
        WithPinned(arr, p => Safe(() => bmb_array_index_of(p, arr.Length, target)));

    /// <summary>0/1 Knapsack. Returns maximum value.</summary>
    public static long Knapsack(long[] weights, long[] values, long capacity) =>
        WithPinned2(weights, values, (pw, pv) => Safe(() => bmb_knapsack(pw, pv, weights.Length, capacity)));

    // Sorting (mutates a copy — BMB sorts in-place on the pointer)
    public static long[] QuickSort(long[] arr)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_quicksort(p, copy.Length)); return 0; });
        return copy;
    }

    public static long[] MergeSort(long[] arr)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_merge_sort(p, copy.Length)); return 0; });
        return copy;
    }

    public static long[] HeapSort(long[] arr)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_heap_sort(p, copy.Length)); return 0; });
        return copy;
    }

    public static long[] InsertionSort(long[] arr)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_insertion_sort(p, copy.Length)); return 0; });
        return copy;
    }

    public static long[] SelectionSort(long[] arr)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_selection_sort(p, copy.Length)); return 0; });
        return copy;
    }

    public static long[] BubbleSort(long[] arr)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_bubble_sort(p, copy.Length)); return 0; });
        return copy;
    }

    public static long[] ShellSort(long[] arr)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_shell_sort(p, copy.Length)); return 0; });
        return copy;
    }

    public static long[] CountingSort(long[] arr, long maxVal)
    {
        var copy = (long[])arr.Clone();
        WithPinned(copy, p => { Safe(() => bmb_counting_sort(p, copy.Length, maxVal)); return 0; });
        return copy;
    }

    /// <summary>k-th smallest element (1-indexed).</summary>
    public static long KthSmallest(long[] arr, long k) =>
        WithPinned(arr, p => Safe(() => bmb_kth_smallest(p, arr.Length, k)));

    /// <summary>Mode of sorted array.</summary>
    public static long ArrayMode(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_array_mode(p, arr.Length)));

    /// <summary>Two-sum: returns i*10000+j index pair, or -1.</summary>
    public static (int i, int j) TwoSum(long[] arr, long target)
    {
        long r = WithPinned(arr, p => Safe(() => bmb_two_sum(p, arr.Length, target)));
        if (r < 0) return (-1, -1);
        return ((int)(r / 10000), (int)(r % 10000));
    }

    // String functions
    /// <summary>Longest Common Subsequence length.</summary>
    public static long Lcs(string a, string b) =>
        WithBmbStrings(a, b, (pa, pb) => Safe(() => bmb_lcs(pa, pb)));

    /// <summary>Levenshtein edit distance.</summary>
    public static long EditDistance(string a, string b) =>
        WithBmbStrings(a, b, (pa, pb) => Safe(() => bmb_edit_distance(pa, pb)));

    /// <summary>DJB2 hash of a string.</summary>
    public static long Djb2Hash(string s) =>
        WithBmbString(s, p => Safe(() => bmb_djb2_hash(p)));
}
