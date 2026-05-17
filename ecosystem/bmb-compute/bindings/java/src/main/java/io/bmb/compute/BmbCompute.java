package io.bmb.compute;

import com.sun.jna.Memory;
import com.sun.jna.Pointer;

import java.util.function.Supplier;

/**
 * Numeric computation powered by BMB — JNA bindings.
 * All calls dispatch into the native bmb_compute shared library.
 *
 * Array results: functions that produce arrays (cumsum, moving_avg, vec_add, etc.)
 * write into a caller-supplied Java long[] — BMB receives a native Memory buffer.
 *
 * Thread safety: each public method wraps calls in bmb_ffi_begin/end.
 * Mean/variance/median results are scaled: divide by 1000 for the actual value.
 */
public final class BmbCompute {
    private static final BmbComputeLib LIB = BmbComputeLib.INSTANCE;

    private BmbCompute() {}

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

    /** Copy Java long[] into a native 64-bit integer buffer. */
    private static Memory toNative(long[] arr) {
        Memory mem = new Memory((long) arr.length * Long.BYTES);
        for (int i = 0; i < arr.length; i++) mem.setLong((long) i * Long.BYTES, arr[i]);
        return mem;
    }

    /** Allocate a zero-filled native buffer for output arrays. */
    private static Memory allocNative(int n) {
        Memory mem = new Memory((long) n * Long.BYTES);
        mem.clear();
        return mem;
    }

    /** Read back a native buffer into a new Java long[]. */
    private static long[] fromNative(Memory mem, int len) {
        long[] arr = new long[len];
        for (int i = 0; i < len; i++) arr[i] = mem.getLong((long) i * Long.BYTES);
        return arr;
    }

    // ── Scalar math ────────────────────────────────────────────────────────────

    public static long abs(long x)                          { return safe(() -> LIB.bmb_c_abs(x)); }
    public static long min(long a, long b)                  { return safe(() -> LIB.bmb_c_min(a, b)); }
    public static long max(long a, long b)                  { return safe(() -> LIB.bmb_c_max(a, b)); }
    public static long clamp(long x, long lo, long hi)      { return safe(() -> LIB.bmb_c_clamp(x, lo, hi)); }
    public static long sign(long x)                         { return safe(() -> LIB.bmb_sign(x)); }
    public static long ipow(long base, long exp)            { return safe(() -> LIB.bmb_ipow(base, exp)); }
    public static long sqrt(long n)                         { return safe(() -> LIB.bmb_sqrt(n)); }
    public static long factorial(long n)                    { return safe(() -> LIB.bmb_factorial(n)); }
    /** Linear interpolation: (a*(1000-t) + b*t) / 1000 scaled × 1000. */
    public static long lerpScaled(long a, long b, long t)   { return safe(() -> LIB.bmb_lerp_scaled(a, b, t)); }
    public static boolean isPowerOfTwo(long n)              { return safe(() -> LIB.bmb_c_is_power_of_two(n)) != 0; }
    public static long nextPowerOfTwo(long n)               { return safe(() -> LIB.bmb_c_next_power_of_two(n)); }

    // ── RNG ────────────────────────────────────────────────────────────────────

    public static long randSeed(long seed)                  { return safe(() -> LIB.bmb_rand_seed(seed)); }
    public static long randNext(long state)                 { return safe(() -> LIB.bmb_rand_next(state)); }
    /** Returns a positive random number derived from {@code state}. */
    public static long randPos(long state)                  { return safe(() -> LIB.bmb_rand_pos(state)); }
    /** Returns a random number in [0, maxVal). */
    public static long randRange(long state, long maxVal)   { return safe(() -> LIB.bmb_rand_range(state, maxVal)); }

    // ── Array statistics ───────────────────────────────────────────────────────

    public static long sum(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_sum(m, arr.length));
    }
    /** Mean × 1000 (divide by 1000 for actual mean). */
    public static long meanScaled(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_mean_scaled(m, arr.length));
    }
    public static long minVal(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_c_min_val(m, arr.length));
    }
    public static long maxVal(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_c_max_val(m, arr.length));
    }
    public static long rangeVal(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_range_val(m, arr.length));
    }
    /** Variance × 1000000. */
    public static long varianceScaled(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_variance_scaled(m, arr.length));
    }
    /** Median × 1000 (modifies a copy; original is unchanged). */
    public static long medianScaled(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_median_scaled(m, arr.length));
    }
    public static long magnitudeSquared(long[] arr) {
        Memory m = toNative(arr); return safe(() -> LIB.bmb_magnitude_squared(m, arr.length));
    }

    // ── Two-array operations ───────────────────────────────────────────────────

    public static long dotProduct(long[] a, long[] b) {
        Memory ma = toNative(a); Memory mb = toNative(b);
        return safe(() -> LIB.bmb_dot_product(ma, mb, a.length));
    }
    public static long distSquared(long[] a, long[] b) {
        Memory ma = toNative(a); Memory mb = toNative(b);
        return safe(() -> LIB.bmb_dist_squared(ma, mb, a.length));
    }
    public static long weightedSum(long[] values, long[] weights) {
        Memory mv = toNative(values); Memory mw = toNative(weights);
        return safe(() -> LIB.bmb_weighted_sum(mv, mw, values.length));
    }

    // ── Output-array operations ────────────────────────────────────────────────

    /** Returns prefix sums of {@code arr}. */
    public static long[] cumsum(long[] arr) {
        Memory mi = toNative(arr);
        Memory mo = allocNative(arr.length);
        safe(() -> LIB.bmb_cumsum(mi, mo, arr.length));
        return fromNative(mo, arr.length);
    }

    /** Moving average (window {@code k}) × 1000 — output is same length as input. */
    public static long[] movingAvgScaled(long[] arr, long k) {
        Memory mi = toNative(arr);
        Memory mo = allocNative(arr.length);
        safe(() -> LIB.bmb_moving_avg_scaled(mi, mo, arr.length, k));
        return fromNative(mo, arr.length);
    }

    /** Element-wise {@code a + b}. */
    public static long[] vecAdd(long[] a, long[] b) {
        Memory ma = toNative(a); Memory mb = toNative(b);
        Memory mo = allocNative(a.length);
        safe(() -> LIB.bmb_vec_add(ma, mb, mo, a.length));
        return fromNative(mo, a.length);
    }

    /** Element-wise {@code a - b}. */
    public static long[] vecSub(long[] a, long[] b) {
        Memory ma = toNative(a); Memory mb = toNative(b);
        Memory mo = allocNative(a.length);
        safe(() -> LIB.bmb_vec_sub(ma, mb, mo, a.length));
        return fromNative(mo, a.length);
    }

    /** Element-wise {@code arr * scalar}. */
    public static long[] vecScale(long[] arr, long scalar) {
        Memory mi = toNative(arr);
        Memory mo = allocNative(arr.length);
        safe(() -> LIB.bmb_vec_scale(mi, scalar, mo, arr.length));
        return fromNative(mo, arr.length);
    }

    /** Element-wise {@code arr[i]^2}. */
    public static long[] mapSquare(long[] arr) {
        Memory mi = toNative(arr);
        Memory mo = allocNative(arr.length);
        safe(() -> LIB.bmb_map_square(mi, mo, arr.length));
        return fromNative(mo, arr.length);
    }
}
