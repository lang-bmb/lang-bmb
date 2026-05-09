'use strict';

/**
 * bmb-algo: High-performance algorithms powered by BMB
 * Node.js bindings via koffi FFI (mirrors Python ctypes approach)
 */

const koffi = require('koffi');
const path = require('path');
const os = require('os');
const fs = require('fs');

// Resolve DLL path
function findLib() {
  const dir = path.resolve(__dirname, '..', '..');
  const name = os.platform() === 'win32'  ? 'bmb_algo.dll'
             : os.platform() === 'darwin' ? 'libbmb_algo.dylib'
                                          : 'libbmb_algo.so';
  const candidate = path.join(dir, name);
  if (fs.existsSync(candidate)) return candidate;
  return path.join(__dirname, name);
}

const _lib = koffi.load(findLib());

// FFI safety API
const _ffi_begin   = _lib.func('int bmb_ffi_begin()');
const _ffi_end     = _lib.func('void bmb_ffi_end()');
const _ffi_has_err = _lib.func('int bmb_ffi_has_error()');
const _ffi_errmsg  = _lib.func('const char* bmb_ffi_error_message()');

// String FFI (BMB opaque String pointer)
const _cstr_to_str  = _lib.func('void* bmb_ffi_cstr_to_string(const char* s)');
const _str_data     = _lib.func('const char* bmb_ffi_string_data(void* s)');
const _free_string  = _lib.func('void bmb_ffi_free_string(void* s)');

/** Wrap a call in FFI begin/end with error check */
function _safe(fn) {
  _ffi_begin();
  try {
    const r = fn();
    if (_ffi_has_err()) {
      throw new Error(`BMB FFI error: ${_ffi_errmsg()}`);
    }
    return r;
  } finally {
    _ffi_end();
  }
}

/** Convert JS string to BMB string pointer */
function _toStr(s) { return _cstr_to_str(s); }

// ── Scalar functions (int64_t args/return → JS number) ───────────────────────
const _gcd         = _lib.func('int64_t bmb_gcd(int64_t a, int64_t b)');
const _lcm         = _lib.func('int64_t bmb_lcm(int64_t a, int64_t b)');
const _fibonacci   = _lib.func('int64_t bmb_fibonacci(int64_t n)');
const _prime_count = _lib.func('int64_t bmb_prime_count(int64_t n)');
const _modpow      = _lib.func('int64_t bmb_modpow(int64_t base, int64_t exp, int64_t mod)');
const _nqueens     = _lib.func('int64_t bmb_nqueens(int64_t n)');
const _is_prime    = _lib.func('int64_t bmb_algo_is_prime(int64_t n)');
const _palindrome  = _lib.func('int64_t bmb_is_palindrome_num(int64_t n)');
const _digit_sum   = _lib.func('int64_t bmb_digit_sum(int64_t n)');
const _bit_pop     = _lib.func('int64_t bmb_bit_popcount(int64_t n)');

// ── Array functions (int64_t* for array params, int64_t for counts/results) ──
// Note: BMB declares array params as int64_t (pointer-as-integer), but koffi
// handles int64_t* → BigInt64Array correctly via pointer declaration.
const _array_sum     = _lib.func('int64_t bmb_array_sum(int64_t* arr, int64_t n)');
const _array_min     = _lib.func('int64_t bmb_array_min(int64_t* arr, int64_t n)');
const _array_max     = _lib.func('int64_t bmb_array_max(int64_t* arr, int64_t n)');
const _is_sorted     = _lib.func('int64_t bmb_is_sorted(int64_t* arr, int64_t n)');
const _binary_search = _lib.func('int64_t bmb_binary_search(int64_t* arr, int64_t n, int64_t target)');
const _max_sub       = _lib.func('int64_t bmb_max_subarray(int64_t* arr, int64_t n)');
const _lis           = _lib.func('int64_t bmb_lis(int64_t* arr, int64_t n)');
const _coin_change   = _lib.func('int64_t bmb_coin_change(int64_t* coins, int64_t n_coins, int64_t amount)');
const _knapsack      = _lib.func('int64_t bmb_knapsack(int64_t* weights, int64_t* values, int64_t n, int64_t capacity)');

// ── String-argument functions ─────────────────────────────────────────────────
const _lcs        = _lib.func('int64_t bmb_lcs(void* a, void* b)');
const _edit_dist  = _lib.func('int64_t bmb_edit_distance(void* a, void* b)');
const _djb2_hash  = _lib.func('int64_t bmb_djb2_hash(void* s)');

// ── Pack number[] → BigInt64Array ────────────────────────────────────────────
function _pack(arr) {
  return new BigInt64Array(arr.map(BigInt));
}

// ── Exported API ─────────────────────────────────────────────────────────────

/** GCD of two non-negative integers. */
function gcd(a, b) { return _safe(() => _gcd(Number(a), Number(b))); }

/** LCM of two positive integers. */
function lcm(a, b) { return _safe(() => _lcm(Number(a), Number(b))); }

/** n-th Fibonacci number (0-indexed). */
function fibonacci(n) { return _safe(() => _fibonacci(Number(n))); }

/** Count primes ≤ n (Sieve of Eratosthenes). */
function prime_count(n) { return _safe(() => _prime_count(Number(n))); }

/** Modular exponentiation: (base^exp) % mod. */
function modpow(base, exp, mod) {
  return _safe(() => _modpow(Number(base), Number(exp), Number(mod)));
}

/** N-Queens solutions count. */
function nqueens(n) { return _safe(() => _nqueens(Number(n))); }

/** Primality test. */
function is_prime(n) { return _safe(() => _is_prime(Number(n))) !== 0; }

/** DJB2 hash of a string. */
function djb2_hash(s) {
  const p = _toStr(s);
  const r = _safe(() => _djb2_hash(p));
  _free_string(p);
  return r;
}

/** Longest Common Subsequence length. */
function lcs(a, b) {
  const pA = _toStr(a);
  const pB = _toStr(b);
  const r = _safe(() => _lcs(pA, pB));
  _free_string(pA);
  _free_string(pB);
  return r;
}

/** Edit distance (Levenshtein). */
function edit_distance(a, b) {
  const pA = _toStr(a);
  const pB = _toStr(b);
  const r = _safe(() => _edit_dist(pA, pB));
  _free_string(pA);
  _free_string(pB);
  return r;
}

/** Maximum subarray sum (Kadane). */
function max_subarray(arr) {
  const buf = _pack(arr);
  return _safe(() => _max_sub(buf, arr.length));
}

/** Minimum coin change. Returns -1 if impossible. */
function coin_change(coins, amount) {
  const buf = _pack(coins);
  return _safe(() => _coin_change(buf, coins.length, amount));
}

/** Longest Increasing Subsequence length. */
function lis(arr) {
  const buf = _pack(arr);
  return _safe(() => _lis(buf, arr.length));
}

/** 0/1 Knapsack. */
function knapsack(weights, values, capacity) {
  const n = weights.length;
  const wBuf = _pack(weights);
  const vBuf = _pack(values);
  return _safe(() => _knapsack(wBuf, vBuf, n, capacity));
}

/** Sum of array elements. */
function array_sum(arr) {
  const buf = _pack(arr);
  return _safe(() => _array_sum(buf, arr.length));
}

/** Minimum element. */
function array_min(arr) {
  const buf = _pack(arr);
  return _safe(() => _array_min(buf, arr.length));
}

/** Maximum element. */
function array_max(arr) {
  const buf = _pack(arr);
  return _safe(() => _array_max(buf, arr.length));
}

/** Binary search. Returns index or -1. */
function binary_search(sortedArr, target) {
  const buf = _pack(sortedArr);
  return _safe(() => _binary_search(buf, sortedArr.length, target));
}

/** Is array sorted in non-decreasing order? */
function is_sorted(arr) {
  const buf = _pack(arr);
  return _safe(() => _is_sorted(buf, arr.length)) !== 0;
}

/** Is n a palindrome number? */
function is_palindrome_num(n) { return _safe(() => _palindrome(Number(n))) !== 0; }

/** Sum of decimal digits of n. */
function digit_sum(n) { return _safe(() => _digit_sum(Number(n))); }

/** Popcount (number of set bits). */
function bit_popcount(n) { return _safe(() => _bit_pop(Number(n))); }

module.exports = {
  gcd, lcm, fibonacci, prime_count, modpow, nqueens, is_prime,
  knapsack, lcs, edit_distance, max_subarray, coin_change, lis,
  array_sum, array_min, array_max, binary_search, is_sorted,
  djb2_hash, is_palindrome_num, digit_sum, bit_popcount,
};
