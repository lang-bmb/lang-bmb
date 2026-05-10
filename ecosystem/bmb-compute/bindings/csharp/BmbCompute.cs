using System.Runtime.InteropServices;

namespace BmbComputeLib;

/// <summary>
/// Numeric computation powered by BMB — P/Invoke bindings.
/// All functions call into the native bmb_compute shared library.
/// </summary>
public static class Compute
{
    private const string LibName = "bmb_compute";

    // ── FFI Safety API ───────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern int    bmb_ffi_begin();
    [DllImport(LibName)] private static extern void   bmb_ffi_end();
    [DllImport(LibName)] private static extern int    bmb_ffi_has_error();
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_error_message();

    // ── String FFI API ───────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_cstr_to_string([MarshalAs(UnmanagedType.LPStr)] string s);
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_string_data(IntPtr s);
    [DllImport(LibName)] private static extern long   bmb_ffi_string_len(IntPtr s);
    [DllImport(LibName)] private static extern void   bmb_ffi_free_string(IntPtr s);

    // ── Scalar functions ─────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern long bmb_c_abs(long x);
    [DllImport(LibName)] private static extern long bmb_c_min(long a, long b);
    [DllImport(LibName)] private static extern long bmb_c_max(long a, long b);
    [DllImport(LibName)] private static extern long bmb_c_clamp(long x, long lo, long hi);
    [DllImport(LibName)] private static extern long bmb_sign(long x);
    [DllImport(LibName)] private static extern long bmb_ipow(long @base, long exp);
    [DllImport(LibName)] private static extern long bmb_sqrt(long n);
    [DllImport(LibName)] private static extern long bmb_factorial(long n);
    [DllImport(LibName)] private static extern long bmb_lerp_scaled(long a, long b, long t);
    [DllImport(LibName)] private static extern long bmb_c_is_power_of_two(long n);
    [DllImport(LibName)] private static extern long bmb_c_next_power_of_two(long n);
    [DllImport(LibName)] private static extern long bmb_rand_seed(long seed);
    [DllImport(LibName)] private static extern long bmb_rand_next(long state);
    [DllImport(LibName)] private static extern long bmb_rand_pos(long state);
    [DllImport(LibName)] private static extern long bmb_rand_range(long state, long max_val);

    // ── Array functions (pointer args passed as IntPtr) ──────────────────────
    [DllImport(LibName)] private static extern long bmb_sum(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_mean_scaled(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_c_min_val(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_c_max_val(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_range_val(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_variance_scaled(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_median_scaled(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_magnitude_squared(IntPtr arr, long n);
    [DllImport(LibName)] private static extern long bmb_dot_product(IntPtr a, IntPtr b, long n);
    [DllImport(LibName)] private static extern long bmb_dist_squared(IntPtr a, IntPtr b, long n);
    [DllImport(LibName)] private static extern long bmb_weighted_sum(IntPtr values, IntPtr weights, long n);

    // ── Out-parameter array functions ────────────────────────────────────────
    [DllImport(LibName)] private static extern long bmb_cumsum(IntPtr arr, IntPtr @out, long n);
    [DllImport(LibName)] private static extern long bmb_moving_avg_scaled(IntPtr arr, IntPtr @out, long n, long k);
    [DllImport(LibName)] private static extern long bmb_array_copy(IntPtr src, IntPtr dst, long n);
    [DllImport(LibName)] private static extern long bmb_vec_add(IntPtr a, IntPtr b, IntPtr @out, long n);
    [DllImport(LibName)] private static extern long bmb_vec_sub(IntPtr a, IntPtr b, IntPtr @out, long n);
    [DllImport(LibName)] private static extern long bmb_vec_scale(IntPtr arr, long scalar, IntPtr @out, long n);
    [DllImport(LibName)] private static extern long bmb_map_square(IntPtr arr, IntPtr @out, long n);

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

    private static T WithPinned3<T>(long[] a, long[] b, long[] c, Func<IntPtr, IntPtr, IntPtr, T> fn)
    {
        var ha = GCHandle.Alloc(a, GCHandleType.Pinned);
        var hb = GCHandle.Alloc(b, GCHandleType.Pinned);
        var hc = GCHandle.Alloc(c, GCHandleType.Pinned);
        try { return fn(ha.AddrOfPinnedObject(), hb.AddrOfPinnedObject(), hc.AddrOfPinnedObject()); }
        finally { ha.Free(); hb.Free(); hc.Free(); }
    }

    // ── Public API — Scalar ──────────────────────────────────────────────────

    /// <summary>Absolute value.</summary>
    public static long Abs(long x) => Safe(() => bmb_c_abs(x));

    /// <summary>Minimum of two values.</summary>
    public static long Min(long a, long b) => Safe(() => bmb_c_min(a, b));

    /// <summary>Maximum of two values.</summary>
    public static long Max(long a, long b) => Safe(() => bmb_c_max(a, b));

    /// <summary>Clamp x to [lo, hi].</summary>
    public static long Clamp(long x, long lo, long hi) => Safe(() => bmb_c_clamp(x, lo, hi));

    /// <summary>Sign of x: -1, 0, or 1.</summary>
    public static long Sign(long x) => Safe(() => bmb_sign(x));

    /// <summary>Integer power: base^exp.</summary>
    public static long IPow(long @base, long exp) => Safe(() => bmb_ipow(@base, exp));

    /// <summary>Integer square root (floor, Newton's method).</summary>
    public static long Sqrt(long n) => Safe(() => bmb_sqrt(n));

    /// <summary>Factorial of n.</summary>
    public static long Factorial(long n) => Safe(() => bmb_factorial(n));

    /// <summary>Linear interpolation scaled ×1000: a + (b-a)*t/1000.</summary>
    public static long LerpScaled(long a, long b, long t) => Safe(() => bmb_lerp_scaled(a, b, t));

    /// <summary>Returns 1 if n is a power of two, 0 otherwise.</summary>
    public static bool IsPowerOfTwo(long n) => Safe(() => bmb_c_is_power_of_two(n)) != 0;

    /// <summary>Next power of two greater than or equal to n.</summary>
    public static long NextPowerOfTwo(long n) => Safe(() => bmb_c_next_power_of_two(n));

    // ── Public API — Random (xorshift state machine) ─────────────────────────

    /// <summary>Initialize random seed (ensures nonzero state).</summary>
    public static long RandSeed(long seed) => Safe(() => bmb_rand_seed(seed));

    /// <summary>Advance random state and return next state.</summary>
    public static long RandNext(long state) => Safe(() => bmb_rand_next(state));

    /// <summary>Random positive value from state.</summary>
    public static long RandPos(long state) => Safe(() => bmb_rand_pos(state));

    /// <summary>Random value in [0, maxVal) from state.</summary>
    public static long RandRange(long state, long maxVal) => Safe(() => bmb_rand_range(state, maxVal));

    // ── Public API — Array (read-only) ───────────────────────────────────────

    /// <summary>Sum of array elements.</summary>
    public static long Sum(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_sum(p, arr.Length)));

    /// <summary>Mean ×1000 (3 decimal places in integer arithmetic).</summary>
    public static long MeanScaled(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_mean_scaled(p, arr.Length)));

    /// <summary>Minimum value in array.</summary>
    public static long MinVal(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_c_min_val(p, arr.Length)));

    /// <summary>Maximum value in array.</summary>
    public static long MaxVal(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_c_max_val(p, arr.Length)));

    /// <summary>Range (max − min) of array.</summary>
    public static long RangeVal(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_range_val(p, arr.Length)));

    /// <summary>Variance ×1000000 (6 decimal places).</summary>
    public static long VarianceScaled(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_variance_scaled(p, arr.Length)));

    /// <summary>Median ×1000. Caller must pass a sorted array.</summary>
    public static long MedianScaled(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_median_scaled(p, arr.Length)));

    /// <summary>Vector magnitude squared (sum of squares).</summary>
    public static long MagnitudeSquared(long[] arr) =>
        WithPinned(arr, p => Safe(() => bmb_magnitude_squared(p, arr.Length)));

    /// <summary>Dot product of two equal-length vectors.</summary>
    public static long DotProduct(long[] a, long[] b) =>
        WithPinned2(a, b, (pa, pb) => Safe(() => bmb_dot_product(pa, pb, Math.Min(a.Length, b.Length))));

    /// <summary>Squared Euclidean distance between two equal-length vectors.</summary>
    public static long DistSquared(long[] a, long[] b) =>
        WithPinned2(a, b, (pa, pb) => Safe(() => bmb_dist_squared(pa, pb, Math.Min(a.Length, b.Length))));

    /// <summary>Weighted sum: Σ values[i] * weights[i].</summary>
    public static long WeightedSum(long[] values, long[] weights) =>
        WithPinned2(values, weights, (pv, pw) => Safe(() => bmb_weighted_sum(pv, pw, Math.Min(values.Length, weights.Length))));

    // ── Public API — Array with output ───────────────────────────────────────

    /// <summary>Cumulative (prefix) sum. Returns a new array of the same length.</summary>
    public static long[] CumSum(long[] arr)
    {
        var result = new long[arr.Length];
        WithPinned2(arr, result, (pa, po) => { Safe(() => bmb_cumsum(pa, po, arr.Length)); return 0; });
        return result;
    }

    /// <summary>Moving average ×1000 with window size k. Returns a new array of the same length.</summary>
    public static long[] MovingAvgScaled(long[] arr, long k)
    {
        var result = new long[arr.Length];
        WithPinned2(arr, result, (pa, po) => { Safe(() => bmb_moving_avg_scaled(pa, po, arr.Length, k)); return 0; });
        return result;
    }

    /// <summary>Copy src into a new array of length n.</summary>
    public static long[] ArrayCopy(long[] src)
    {
        var dst = new long[src.Length];
        WithPinned2(src, dst, (ps, pd) => { Safe(() => bmb_array_copy(ps, pd, src.Length)); return 0; });
        return dst;
    }

    /// <summary>Element-wise addition: out[i] = a[i] + b[i].</summary>
    public static long[] VecAdd(long[] a, long[] b)
    {
        int n = Math.Min(a.Length, b.Length);
        var result = new long[n];
        WithPinned3(a, b, result, (pa, pb, po) => { Safe(() => bmb_vec_add(pa, pb, po, n)); return 0; });
        return result;
    }

    /// <summary>Element-wise subtraction: out[i] = a[i] − b[i].</summary>
    public static long[] VecSub(long[] a, long[] b)
    {
        int n = Math.Min(a.Length, b.Length);
        var result = new long[n];
        WithPinned3(a, b, result, (pa, pb, po) => { Safe(() => bmb_vec_sub(pa, pb, po, n)); return 0; });
        return result;
    }

    /// <summary>Scalar multiplication: out[i] = arr[i] * scalar.</summary>
    public static long[] VecScale(long[] arr, long scalar)
    {
        var result = new long[arr.Length];
        WithPinned2(arr, result, (pa, po) => { Safe(() => bmb_vec_scale(pa, scalar, po, arr.Length)); return 0; });
        return result;
    }

    /// <summary>Element-wise square: out[i] = arr[i]^2.</summary>
    public static long[] MapSquare(long[] arr)
    {
        var result = new long[arr.Length];
        WithPinned2(arr, result, (pa, po) => { Safe(() => bmb_map_square(pa, po, arr.Length)); return 0; });
        return result;
    }
}
