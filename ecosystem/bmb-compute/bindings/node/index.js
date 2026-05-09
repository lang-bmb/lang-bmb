'use strict';

/**
 * bmb-compute: Numeric computation powered by BMB
 * Node.js bindings via koffi FFI
 */

const koffi = require('koffi');
const path = require('path');
const os = require('os');
const fs = require('fs');

function findLib() {
  const dir = path.resolve(__dirname, '..', '..');
  const name = os.platform() === 'win32'  ? 'bmb_compute.dll'
             : os.platform() === 'darwin' ? 'libbmb_compute.dylib'
                                          : 'libbmb_compute.so';
  const candidate = path.join(dir, name);
  if (fs.existsSync(candidate)) return candidate;
  return path.join(__dirname, name);
}

const _lib = koffi.load(findLib());

// FFI safety
const _ffi_begin   = _lib.func('int bmb_ffi_begin()');
const _ffi_end     = _lib.func('void bmb_ffi_end()');
const _ffi_has_err = _lib.func('int bmb_ffi_has_error()');
const _ffi_errmsg  = _lib.func('const char* bmb_ffi_error_message()');

function _safe(fn) {
  _ffi_begin();
  try {
    const r = fn();
    if (_ffi_has_err()) throw new Error(`BMB FFI error: ${_ffi_errmsg()}`);
    return r;
  } finally { _ffi_end(); }
}

function _pack(arr) { return new BigInt64Array(arr.map(BigInt)); }

// Scalar functions
const _abs     = _lib.func('int64_t bmb_c_abs(int64_t x)');
const _cmin    = _lib.func('int64_t bmb_c_min(int64_t a, int64_t b)');
const _cmax    = _lib.func('int64_t bmb_c_max(int64_t a, int64_t b)');
const _clamp   = _lib.func('int64_t bmb_c_clamp(int64_t x, int64_t lo, int64_t hi)');
const _sign    = _lib.func('int64_t bmb_sign(int64_t x)');
const _ipow    = _lib.func('int64_t bmb_ipow(int64_t base, int64_t exp)');
const _sqrt    = _lib.func('int64_t bmb_sqrt(int64_t n)');
const _fact    = _lib.func('int64_t bmb_factorial(int64_t n)');
const _is_p2   = _lib.func('int64_t bmb_c_is_power_of_two(int64_t n)');
const _next_p2 = _lib.func('int64_t bmb_c_next_power_of_two(int64_t n)');

// Random
const _rseed = _lib.func('int64_t bmb_rand_seed(int64_t seed)');
const _rnext = _lib.func('int64_t bmb_rand_next(int64_t state)');
const _rpos  = _lib.func('int64_t bmb_rand_pos(int64_t state)');
const _rrange = _lib.func('int64_t bmb_rand_range(int64_t state, int64_t max)');

// Array/stats functions (pointer + n)
const _sum     = _lib.func('int64_t bmb_sum(int64_t* arr, int64_t n)');
const _mean    = _lib.func('int64_t bmb_mean_scaled(int64_t* arr, int64_t n)');
const _min_val = _lib.func('int64_t bmb_c_min_val(int64_t* arr, int64_t n)');
const _max_val = _lib.func('int64_t bmb_c_max_val(int64_t* arr, int64_t n)');
const _range   = _lib.func('int64_t bmb_range_val(int64_t* arr, int64_t n)');
const _var     = _lib.func('int64_t bmb_variance_scaled(int64_t* arr, int64_t n)');
const _median  = _lib.func('int64_t bmb_median_scaled(int64_t* arr, int64_t n)');
const _mag_sq  = _lib.func('int64_t bmb_magnitude_squared(int64_t* arr, int64_t n)');

// Vector ops (two arrays + n)
const _dot    = _lib.func('int64_t bmb_dot_product(int64_t* a, int64_t* b, int64_t n)');
const _dist   = _lib.func('int64_t bmb_dist_squared(int64_t* a, int64_t* b, int64_t n)');
const _wsum   = _lib.func('int64_t bmb_weighted_sum(int64_t* vals, int64_t* wts, int64_t n)');

// ── Exports ───────────────────────────────────────────────────────────────────

/** Absolute value. */
function abs(x) { return _safe(() => _abs(Number(x))); }

/** Minimum of two scalars. */
function min(a, b) { return _safe(() => _cmin(Number(a), Number(b))); }

/** Maximum of two scalars. */
function max(a, b) { return _safe(() => _cmax(Number(a), Number(b))); }

/** Clamp x to [lo, hi]. */
function clamp(x, lo, hi) { return _safe(() => _clamp(Number(x), Number(lo), Number(hi))); }

/** Sign: -1, 0, or 1. */
function sign(x) { return _safe(() => _sign(Number(x))); }

/** Integer power: base^exp. */
function ipow(base, exp) { return _safe(() => _ipow(Number(base), Number(exp))); }

/** Integer square root (floor). */
function sqrt(n) { return _safe(() => _sqrt(Number(n))); }

/** Factorial (up to 20!). */
function factorial(n) { return _safe(() => _fact(Number(n))); }

/** Is n a power of two? */
function is_power_of_two(n) { return _safe(() => _is_p2(Number(n))) !== 0; }

/** Smallest power of two ≥ n. */
function next_power_of_two(n) { return _safe(() => _next_p2(Number(n))); }

// Random (XorShift64*)
/** Initialize PRNG. Returns new state. */
function rand_seed(seed) { return _safe(() => _rseed(Number(seed))); }

/** Next PRNG state. */
function rand_next(state) { return _safe(() => _rnext(Number(state))); }

/** Positive random from state. */
function rand_pos(state) { return _safe(() => _rpos(Number(state))); }

/** Random in [0, max). */
function rand_range(state, max) { return _safe(() => _rrange(Number(state), Number(max))); }

// Statistics
/** Sum of array. */
function sum(arr) { const b = _pack(arr); return _safe(() => _sum(b, arr.length)); }

/** Mean × 1000 (e.g. 20000 = 20.000). */
function mean_scaled(arr) { const b = _pack(arr); return _safe(() => _mean(b, arr.length)); }

/** Minimum of array. */
function min_val(arr) { const b = _pack(arr); return _safe(() => _min_val(b, arr.length)); }

/** Maximum of array. */
function max_val(arr) { const b = _pack(arr); return _safe(() => _max_val(b, arr.length)); }

/** Range: max - min. */
function range_val(arr) { const b = _pack(arr); return _safe(() => _range(b, arr.length)); }

/** Variance × 1000000. */
function variance_scaled(arr) { const b = _pack(arr); return _safe(() => _var(b, arr.length)); }

/** Median × 1000. */
function median_scaled(arr) { const b = _pack(arr); return _safe(() => _median(b, arr.length)); }

/** Squared magnitude of vector. */
function magnitude_squared(arr) { const b = _pack(arr); return _safe(() => _mag_sq(b, arr.length)); }

// Vector
/** Dot product of two vectors. */
function dot_product(a, b) {
  const ba = _pack(a), bb = _pack(b);
  return _safe(() => _dot(ba, bb, a.length));
}

/** Squared Euclidean distance. */
function dist_squared(a, b) {
  const ba = _pack(a), bb = _pack(b);
  return _safe(() => _dist(ba, bb, a.length));
}

/** Weighted sum. */
function weighted_sum(values, weights) {
  const bv = _pack(values), bw = _pack(weights);
  return _safe(() => _wsum(bv, bw, values.length));
}

module.exports = {
  // Math
  abs, min, max, clamp, sign, ipow, sqrt, factorial,
  is_power_of_two, next_power_of_two,
  // Random
  rand_seed, rand_next, rand_pos, rand_range,
  // Stats
  sum, mean_scaled, min_val, max_val, range_val, variance_scaled,
  median_scaled, magnitude_squared,
  // Vector
  dot_product, dist_squared, weighted_sum,
};
