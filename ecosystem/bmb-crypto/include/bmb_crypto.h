/**
 * bmb_crypto.h — Cryptographic functions
 *
 * Auto-generated from BMB source. Do not edit manually.
 * Generated: 2026-03-23
 *
 * Usage:
 *   #include "bmb_crypto.h"
 *   // Link with bmb_crypto.dll / libbmb_crypto.so / libbmb_crypto.dylib
 */

#ifndef BMB_CRYPTO_H
#define BMB_CRYPTO_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* FFI Safety API */
int bmb_ffi_begin(void);
void bmb_ffi_end(void);
int bmb_ffi_has_error(void);
const char* bmb_ffi_error_message(void);

/* String FFI API */
void* bmb_ffi_cstr_to_string(const char* s);
const char* bmb_ffi_string_data(void* s);
int64_t bmb_ffi_string_len(void* s);
void bmb_ffi_free_string(void* s);

/* Cryptographic functions — 14 functions */

void* bmb_sha256(void* input);

void* bmb_base64_encode(void* input);

void* bmb_base64_decode(void* input);

void* bmb_md5(void* input);

void* bmb_crc32(void* input);

void* bmb_base32_encode(void* input);

void* bmb_base32_decode(void* input);

void* bmb_hmac_sha256(void* key, void* msg);

void* bmb_adler32(void* input);

void* bmb_fletcher16(void* input);

void* bmb_xor_checksum(void* input);

void* bmb_rot13(void* input);

/** Hex Encode — Convert each byte to 2-char hex */
void* bmb_hex_encode(void* input);

void* bmb_hex_decode(void* input);

#ifdef __cplusplus
}
#endif

#endif /* BMB_CRYPTO_H */