'use strict';

const koffi = require('koffi');
const path = require('path');
const os = require('os');
const fs = require('fs');

function findLib() {
  const dir = path.resolve(__dirname, '..', '..');
  const name = os.platform() === 'win32'  ? 'bmb_json.dll'
             : os.platform() === 'darwin' ? 'libbmb_json.dylib'
                                          : 'libbmb_json.so';
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

const _s    = (str) => _cstr_to_str(str);
const _from = (ptr) => _str_data(ptr);

// String → i64
const _validate  = _lib.func('int64_t bmb_json_validate(void* s)');
const _arr_len   = _lib.func('int64_t bmb_json_array_len(void* s)');
const _obj_len   = _lib.func('int64_t bmb_json_object_len(void* s)');
const _count     = _lib.func('int64_t bmb_json_count(void* s)');

// String×String → i64
const _get_num   = _lib.func('int64_t bmb_json_get_number(void* json, void* key)');
const _has_key   = _lib.func('int64_t bmb_json_has_key(void* json, void* key)');
const _get_bool  = _lib.func('int64_t bmb_json_get_bool(void* json, void* key)');

// String → String
const _stringify = _lib.func('void* bmb_json_stringify(void* s)');
const _get_type  = _lib.func('void* bmb_json_type(void* s)');

// String×String → String
const _get       = _lib.func('void* bmb_json_get(void* json, void* key)');
const _get_str   = _lib.func('void* bmb_json_get_string(void* json, void* key)');

// String×i64 → String
const _arr_get   = _lib.func('void* bmb_json_array_get(void* json, int64_t idx)');

// ── Helpers ───────────────────────────────────────────────────────────────────
function _s1i(fn, s) { const p = _s(s); const r = _safe(() => fn(p)); _free_str(p); return r; }
function _s1str(fn, s) { const p = _s(s); const ptr = _safe(() => fn(p)); _free_str(p); return _from(ptr); }
function _ss_i(fn, a, b) { const pa = _s(a), pb = _s(b); const r = _safe(() => fn(pa, pb)); _free_str(pa); _free_str(pb); return r; }
function _ss_str(fn, a, b) { const pa = _s(a), pb = _s(b); const ptr = _safe(() => fn(pa, pb)); _free_str(pa); _free_str(pb); return _from(ptr); }

// ── API ───────────────────────────────────────────────────────────────────────

/** Is the JSON string valid? */
function validate(json) { return _s1i(_validate, json) !== 0; }

/** Get the JSON type: "object", "array", "string", "number", "bool", "null". */
function get_type(json) { return _s1str(_get_type, json); }

/** Re-stringify (normalize whitespace). */
function stringify(json) { return _s1str(_stringify, json); }

/** Array length (-1 if not array). */
function array_len(json) { return _s1i(_arr_len, json); }

/** Object key count (-1 if not object). */
function object_len(json) { return _s1i(_obj_len, json); }

/** Total element count (array or object). */
function count(json) { return _s1i(_count, json); }

/** Get numeric value at key. */
function get_number(json, key) { return _ss_i(_get_num, json, key); }

/** Does the object have this key? */
function has_key(json, key) { return _ss_i(_has_key, json, key) !== 0; }

/** Get boolean value at key (1 = true, 0 = false, -1 = missing). */
function get_bool(json, key) { return _ss_i(_get_bool, json, key); }

/** Get raw JSON value at key (returns JSON string). */
function get(json, key) { return _ss_str(_get, json, key); }

/** Get string value at key (unquoted). */
function get_string(json, key) { return _ss_str(_get_str, json, key); }

/** Get element at array index (returns JSON string). */
function array_get(json, idx) {
  const p = _s(json);
  const ptr = _safe(() => _arr_get(p, Number(idx)));
  _free_str(p);
  return _from(ptr);
}

module.exports = {
  validate, get_type, stringify,
  array_len, object_len, count,
  get_number, has_key, get_bool,
  get, get_string, array_get,
};
