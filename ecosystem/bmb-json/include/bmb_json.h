/**
 * bmb_json.h — JSON parsing
 *
 * Auto-generated from BMB source. Do not edit manually.
 * Generated: 2026-03-23
 *
 * Usage:
 *   #include "bmb_json.h"
 *   // Link with bmb_json.dll / libbmb_json.so / libbmb_json.dylib
 */

#ifndef BMB_JSON_H
#define BMB_JSON_H

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

/* JSON parsing — 12 functions */

/** Simple validation: try parse + serialize roundtrip */
int64_t bmb_json_validate(void* input);

/** Parse and re-serialize (roundtrip normalization) */
void* bmb_json_stringify(void* input);

/** Get value type as string: "null", "bool", "number", "string", "array", "object" */
void* bmb_json_type(void* input);

/** Get object value by key (returns JSON string representation) */
void* bmb_json_get(void* input, void* key);

/** Get string value from object by key (returns raw string, no quotes) */
void* bmb_json_get_string(void* input, void* key);

/** Get number from object by key (returns i64, 0 if not found) */
int64_t bmb_json_get_number(void* input, void* key);

/** Get array length (-1 if not an array) */
int64_t bmb_json_array_len(void* input);

/** Get array element at index as JSON string */
void* bmb_json_array_get(void* input, int64_t idx);

/** Check if object has a key (returns 1 if found, 0 otherwise) */
int64_t bmb_json_has_key(void* input, void* key);

/** Get number of keys in a JSON object (returns -1 for non-objects) */
int64_t bmb_json_object_len(void* input);

/** Get boolean value by key (returns 1=true, 0=false, -1=missing) */
int64_t bmb_json_get_bool(void* input, void* key);

/** Count total number of elements in a JSON structure (recursive) */
int64_t bmb_json_count(void* input);

#ifdef __cplusplus
}
#endif

#endif /* BMB_JSON_H */