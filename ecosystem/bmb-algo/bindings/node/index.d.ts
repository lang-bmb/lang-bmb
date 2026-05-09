/**
 * bmb-algo: High-performance algorithms powered by BMB
 * Node.js FFI bindings via koffi.
 */

/** GCD of two non-negative integers. */
export declare function gcd(a: number, b: number): number;

/** LCM of two positive integers. */
export declare function lcm(a: number, b: number): number;

/** n-th Fibonacci number (0-indexed). */
export declare function fibonacci(n: number): number;

/** Count primes ≤ n (Sieve of Eratosthenes). */
export declare function prime_count(n: number): number;

/** Modular exponentiation: (base^exp) % mod. */
export declare function modpow(base: number, exp: number, mod: number): number;

/** N-Queens solutions count. */
export declare function nqueens(n: number): number;

/** Primality test. Returns true if n is prime. */
export declare function is_prime(n: number): boolean;

/** DJB2 hash of a string. */
export declare function djb2_hash(s: string): number;

/** Longest Common Subsequence length. */
export declare function lcs(a: string, b: string): number;

/** Edit distance (Levenshtein). */
export declare function edit_distance(a: string, b: string): number;

/** Maximum subarray sum (Kadane's algorithm). */
export declare function max_subarray(arr: number[]): number;

/** Minimum coin change. Returns -1 if impossible. */
export declare function coin_change(coins: number[], amount: number): number;

/** Longest Increasing Subsequence length. */
export declare function lis(arr: number[]): number;

/** 0/1 Knapsack maximum value. */
export declare function knapsack(weights: number[], values: number[], capacity: number): number;

/** Sum of array elements. */
export declare function array_sum(arr: number[]): number;

/** Minimum element of array. */
export declare function array_min(arr: number[]): number;

/** Maximum element of array. */
export declare function array_max(arr: number[]): number;

/** Binary search. Returns index or -1. Pre: array must be sorted. */
export declare function binary_search(sortedArr: number[], target: number): number;

/** Is array sorted in non-decreasing order? */
export declare function is_sorted(arr: number[]): boolean;

/** Is n a palindrome number? */
export declare function is_palindrome_num(n: number): boolean;

/** Sum of decimal digits of n. */
export declare function digit_sum(n: number): number;

/** Popcount (number of set bits in n). */
export declare function bit_popcount(n: number): number;
