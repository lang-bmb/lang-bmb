package io.bmb.compute;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

/** Raw JNA interface — direct 1:1 mapping of the bmb_compute native C ABI. */
interface BmbComputeLib extends Library {
    BmbComputeLib INSTANCE = Native.load("bmb_compute", BmbComputeLib.class);

    // FFI Safety API
    int  bmb_ffi_begin();
    void bmb_ffi_end();
    int  bmb_ffi_has_error();
    Pointer bmb_ffi_error_message();

    // Scalar math
    long bmb_c_abs(long x);
    long bmb_c_min(long a, long b);
    long bmb_c_max(long a, long b);
    long bmb_c_clamp(long x, long lo, long hi);
    long bmb_sign(long x);
    long bmb_ipow(long base, long exp);
    long bmb_sqrt(long n);
    long bmb_factorial(long n);
    long bmb_lerp_scaled(long a, long b, long t);
    long bmb_c_is_power_of_two(long n);
    long bmb_c_next_power_of_two(long n);

    // RNG (LCG — pass state in/out)
    long bmb_rand_seed(long seed);
    long bmb_rand_next(long state);
    long bmb_rand_pos(long state);
    long bmb_rand_range(long state, long maxVal);

    // Array statistics (Pointer = int64* native buffer)
    long bmb_sum(Pointer arr, long n);
    long bmb_mean_scaled(Pointer arr, long n);
    long bmb_c_min_val(Pointer arr, long n);
    long bmb_c_max_val(Pointer arr, long n);
    long bmb_range_val(Pointer arr, long n);
    long bmb_variance_scaled(Pointer arr, long n);
    long bmb_median_scaled(Pointer arr, long n);
    long bmb_magnitude_squared(Pointer arr, long n);

    // Two-array operations
    long bmb_dot_product(Pointer a, Pointer b, long n);
    long bmb_dist_squared(Pointer a, Pointer b, long n);
    long bmb_weighted_sum(Pointer values, Pointer weights, long n);

    // In-place / output-array operations
    long bmb_array_copy(Pointer src, Pointer dst, long n);
    long bmb_cumsum(Pointer arr, Pointer out, long n);
    long bmb_moving_avg_scaled(Pointer arr, Pointer out, long n, long k);
    long bmb_vec_add(Pointer a, Pointer b, Pointer out, long n);
    long bmb_vec_sub(Pointer a, Pointer b, Pointer out, long n);
    long bmb_vec_scale(Pointer arr, long scalar, Pointer out, long n);
    long bmb_map_square(Pointer arr, Pointer out, long n);
}
