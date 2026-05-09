'use strict';

const koffi = require('koffi');
const path = require('path');
const os = require('os');
const fs = require('fs');

function findLib() {
  const dir = path.resolve(__dirname, '..', '..');
  const name = os.platform() === 'win32'  ? 'bmb_crypto.dll'
             : os.platform() === 'darwin' ? 'libbmb_crypto.dylib'
                                          : 'libbmb_crypto.so';
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
const _from = (ptr) => { const s = _str_data(ptr); _free_str(ptr); return s; };

// All crypto functions: String → String (hash/encode/decode)
const _sha256   = _lib.func('void* bmb_sha256(void* s)');
const _md5      = _lib.func('void* bmb_md5(void* s)');
const _crc32    = _lib.func('void* bmb_crc32(void* s)');
const _b64enc   = _lib.func('void* bmb_base64_encode(void* s)');
const _b64dec   = _lib.func('void* bmb_base64_decode(void* s)');
const _b32enc   = _lib.func('void* bmb_base32_encode(void* s)');
const _b32dec   = _lib.func('void* bmb_base32_decode(void* s)');
const _hmac256  = _lib.func('void* bmb_hmac_sha256(void* key, void* msg)');
const _adler32  = _lib.func('void* bmb_adler32(void* s)');
const _fletch   = _lib.func('void* bmb_fletcher16(void* s)');
const _xor_chk  = _lib.func('void* bmb_xor_checksum(void* s)');
const _rot13    = _lib.func('void* bmb_rot13(void* s)');
const _hex_enc  = _lib.func('void* bmb_hex_encode(void* s)');
const _hex_dec  = _lib.func('void* bmb_hex_decode(void* s)');

function _call1(fn, s) {
  const p = _s(s);
  const ptr = _safe(() => fn(p));
  _free_str(p);
  return _from(ptr);
}

function _call2(fn, a, b) {
  const pa = _s(a), pb = _s(b);
  const ptr = _safe(() => fn(pa, pb));
  _free_str(pa); _free_str(pb);
  return _from(ptr);
}

/** SHA-256 hex digest. */
function sha256(s) { return _call1(_sha256, s); }

/** MD5 hex digest. */
function md5(s) { return _call1(_md5, s); }

/** CRC32 hex. */
function crc32(s) { return _call1(_crc32, s); }

/** Base64 encode. */
function base64_encode(s) { return _call1(_b64enc, s); }

/** Base64 decode. */
function base64_decode(s) { return _call1(_b64dec, s); }

/** Base32 encode. */
function base32_encode(s) { return _call1(_b32enc, s); }

/** Base32 decode. */
function base32_decode(s) { return _call1(_b32dec, s); }

/** HMAC-SHA256. */
function hmac_sha256(key, msg) { return _call2(_hmac256, key, msg); }

/** Adler-32 checksum. */
function adler32(s) { return _call1(_adler32, s); }

/** Fletcher-16 checksum. */
function fletcher16(s) { return _call1(_fletch, s); }

/** XOR checksum. */
function xor_checksum(s) { return _call1(_xor_chk, s); }

/** ROT13 encode/decode. */
function rot13(s) { return _call1(_rot13, s); }

/** Hex encode. */
function hex_encode(s) { return _call1(_hex_enc, s); }

/** Hex decode. */
function hex_decode(s) { return _call1(_hex_dec, s); }

module.exports = {
  sha256, md5, crc32,
  base64_encode, base64_decode,
  base32_encode, base32_decode,
  hmac_sha256,
  adler32, fletcher16, xor_checksum,
  rot13, hex_encode, hex_decode,
};
