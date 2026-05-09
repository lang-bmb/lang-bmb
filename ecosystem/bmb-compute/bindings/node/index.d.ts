/**
 * bmb-compute: High-performance numeric computations powered by BMB
 * Node.js FFI bindings via koffi.
 */

// Scalar math
export declare function abs(x: number): number;
export declare function min(a: number, b: number): number;
export declare function max(a: number, b: number): number;
export declare function clamp(x: number, lo: number, hi: number): number;
export declare function sign(x: number): number;
export declare function ipow(base: number, exp: number): number;
export declare function sqrt(n: number): number;
export declare function factorial(n: number): number;
export declare function is_power_of_two(n: number): boolean;
export declare function next_power_of_two(n: number): number;

// XorShift64* PRNG
export declare function rand_seed(seed: number): number;
export declare function rand_next(state: number): number;
export declare function rand_pos(state: number): number;
export declare function rand_range(state: number, max: number): number;

// Statistics (integer arrays; scaled results use *1000 fixed-point)
export declare function sum(arr: number[]): number;
/** Mean × 1000 (fixed-point integer result). */
export declare function mean_scaled(arr: number[]): number;
export declare function min_val(arr: number[]): number;
export declare function max_val(arr: number[]): number;
export declare function range_val(arr: number[]): number;
/** Variance × 1000 (fixed-point integer result). */
export declare function variance_scaled(arr: number[]): number;
/** Median × 1000 (fixed-point integer result). */
export declare function median_scaled(arr: number[]): number;

// Vector operations
export declare function magnitude_squared(arr: number[]): number;
export declare function dot_product(a: number[], b: number[]): number;
export declare function dist_squared(a: number[], b: number[]): number;
export declare function weighted_sum(values: number[], weights: number[]): number;
