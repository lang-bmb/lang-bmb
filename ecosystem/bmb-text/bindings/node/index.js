'use strict';

const koffi = require('koffi');
const path = require('path');
const os = require('os');
const fs = require('fs');

function findLib() {
  const dir = path.resolve(__dirname, '..', '..');
  const name = os.platform() === 'win32'  ? 'bmb_text.dll'
             : os.platform() === 'darwin' ? 'libbmb_text.dylib'
                                          : 'libbmb_text.so';
  const candidate = path.join(dir, name);
  if (fs.existsSync(candidate)) return candidate;
  return path.join(__dirname, name);
}

const _lib = koffi.load(findLib());

const _ffi_begin   = _lib.func('int bmb_ffi_begin()');
const _ffi_end     = _lib.func('void bmb_ffi_end()');
const _ffi_has_err = _lib.func('int bmb_ffi_has_error()');
const _ffi_errmsg  = _lib.func('const char* bmb_ffi_error_message()');
const _cstr_to_str = _lib.func('void* bmb_ffi_cstr_to_string(const char* s)');
const _str_data    = _lib.func('const char* bmb_ffi_string_data(void* s)');
const _free_str    = _lib.func('void bmb_ffi_free_string(void* s)');

function _safe(fn) {
  _ffi_begin();
  try {
    const r = fn();
    if (_ffi_has_err()) throw new Error(`BMB FFI error: ${_ffi_errmsg()}`);
    return r;
  } finally { _ffi_end(); }
}

const _s = (str) => _cstr_to_str(str);
const _from = (ptr) => { const s = _str_data(ptr); _free_str(ptr); return s; };

// String×String → i64
const _kmp_search   = _lib.func('int64_t bmb_kmp_search(void* text, void* pattern)');
const _str_find     = _lib.func('int64_t bmb_str_find(void* s, void* pattern)');
const _str_rfind    = _lib.func('int64_t bmb_str_rfind(void* s, void* pattern)');
const _str_count    = _lib.func('int64_t bmb_str_count(void* s, void* pattern)');
const _str_contains = _lib.func('int64_t bmb_str_contains(void* s, void* pattern)');
const _str_starts   = _lib.func('int64_t bmb_str_starts_with(void* s, void* prefix)');
const _str_ends     = _lib.func('int64_t bmb_str_ends_with(void* s, void* suffix)');
const _str_hamming  = _lib.func('int64_t bmb_str_hamming(void* a, void* b)');
const _str_compare  = _lib.func('int64_t bmb_str_compare(void* a, void* b)');

// String+i64 → i64
const _find_byte  = _lib.func('int64_t bmb_str_find_byte(void* s, int64_t b)');
const _count_byte = _lib.func('int64_t bmb_str_count_byte(void* s, int64_t b)');
const _token_cnt  = _lib.func('int64_t bmb_token_count(void* s, int64_t sep)');
const _char_at    = _lib.func('int64_t bmb_str_char_at(void* s, int64_t i)');

// String → i64
const _is_pal   = _lib.func('int64_t bmb_is_palindrome(void* s)');
const _word_cnt = _lib.func('int64_t bmb_word_count(void* s)');
const _str_len  = _lib.func('int64_t bmb_text_len(void* s)');

// String → String
const _str_rev      = _lib.func('void* bmb_str_reverse(void* s)');
const _to_upper     = _lib.func('void* bmb_str_to_upper(void* s)');
const _to_lower     = _lib.func('void* bmb_str_to_lower(void* s)');
const _trim         = _lib.func('void* bmb_str_trim(void* s)');

// String×String×String → String
const _replace     = _lib.func('void* bmb_str_replace(void* s, void* from, void* to)');
const _replace_all = _lib.func('void* bmb_str_replace_all(void* s, void* from, void* to)');

// String×i64 → String
const _repeat = _lib.func('void* bmb_str_repeat(void* s, int64_t n)');

// ── Helpers ───────────────────────────────────────────────────────────────────
function _ss(fn, a, b) {
  const pa = _s(a), pb = _s(b);
  const r = _safe(() => fn(pa, pb));
  _free_str(pa); _free_str(pb);
  return r;
}
function _sss(fn, a, b, c_str) {
  const pa = _s(a), pb = _s(b), pc = _s(c_str);
  const ptr = _safe(() => fn(pa, pb, pc));
  _free_str(pa); _free_str(pb); _free_str(pc);
  return _from(ptr);
}
function _s1(fn, a) {
  const pa = _s(a);
  const r = _safe(() => fn(pa));
  _free_str(pa);
  return r;
}
function _s1str(fn, a) {
  const pa = _s(a);
  const ptr = _safe(() => fn(pa));
  _free_str(pa);
  return _from(ptr);
}

// ── API ───────────────────────────────────────────────────────────────────────

/** KMP substring search. Returns index or -1. */
function kmp_search(text, pattern) { return _ss(_kmp_search, text, pattern); }

/** First occurrence of pattern in s. Returns index or -1. */
function str_find(s, pattern) { return _ss(_str_find, s, pattern); }

/** Last occurrence of pattern in s. Returns index or -1. */
function str_rfind(s, pattern) { return _ss(_str_rfind, s, pattern); }

/** Count non-overlapping occurrences of pattern. */
function str_count(s, pattern) { return _ss(_str_count, s, pattern); }

/** Does s contain pattern? */
function str_contains(s, pattern) { return _ss(_str_contains, s, pattern) !== 0; }

/** Does s start with prefix? */
function str_starts_with(s, prefix) { return _ss(_str_starts, s, prefix) !== 0; }

/** Does s end with suffix? */
function str_ends_with(s, suffix) { return _ss(_str_ends, s, suffix) !== 0; }

/** Hamming distance (equal length required). */
function hamming_distance(a, b) { return _ss(_str_hamming, a, b); }

/** Lexicographic compare: <0, 0, >0. */
function str_compare(a, b) { return _ss(_str_compare, a, b); }

/** Find first occurrence of byte (char code). */
function find_byte(s, b) {
  const ps = _s(s);
  const r = _safe(() => _find_byte(ps, typeof b === 'string' ? b.charCodeAt(0) : Number(b)));
  _free_str(ps);
  return r;
}

/** Count occurrences of byte. */
function count_byte(s, b) {
  const ps = _s(s);
  const r = _safe(() => _count_byte(ps, typeof b === 'string' ? b.charCodeAt(0) : Number(b)));
  _free_str(ps);
  return r;
}

/** Count tokens separated by byte. */
function token_count(s, sep) {
  const ps = _s(s);
  const r = _safe(() => _token_cnt(ps, typeof sep === 'string' ? sep.charCodeAt(0) : Number(sep)));
  _free_str(ps);
  return r;
}

/** Char code at index. */
function str_char_at(s, i) {
  const ps = _s(s);
  const r = _safe(() => _char_at(ps, Number(i)));
  _free_str(ps);
  return r;
}

/** Is s a palindrome? */
function is_palindrome(s) { return _s1(_is_pal, s) !== 0; }

/** Word count (whitespace-delimited). */
function word_count(s) { return _s1(_word_cnt, s); }

/** String length in bytes. */
function str_len(s) { return _s1(_str_len, s); }

/** Reverse string. */
function str_reverse(s) { return _s1str(_str_rev, s); }

/** Uppercase. */
function to_upper(s) { return _s1str(_to_upper, s); }

/** Lowercase. */
function to_lower(s) { return _s1str(_to_lower, s); }

/** Trim whitespace. */
function trim(s) { return _s1str(_trim, s); }

/** Replace first occurrence of `from` with `to`. */
function str_replace(s, from, to) { return _sss(_replace, s, from, to); }

/** Replace all occurrences of `from` with `to`. */
function str_replace_all(s, from, to) { return _sss(_replace_all, s, from, to); }

/** Repeat string n times. */
function repeat(s, n) {
  const ps = _s(s);
  const ptr = _safe(() => _repeat(ps, Number(n)));
  _free_str(ps);
  return _from(ptr);
}

module.exports = {
  kmp_search, str_find, str_rfind, str_count,
  str_contains, str_starts_with, str_ends_with,
  find_byte, count_byte, token_count, str_char_at,
  hamming_distance, str_compare,
  is_palindrome, word_count, str_len,
  str_reverse, to_upper, to_lower, trim,
  str_replace, str_replace_all, repeat,
};
