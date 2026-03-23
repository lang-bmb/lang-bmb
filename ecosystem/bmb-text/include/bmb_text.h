/**
 * bmb_text.h — String processing
 *
 * Auto-generated from BMB source. Do not edit manually.
 * Generated: 2026-03-23
 *
 * Usage:
 *   #include "bmb_text.h"
 *   // Link with bmb_text.dll / libbmb_text.so / libbmb_text.dylib
 */

#ifndef BMB_TEXT_H
#define BMB_TEXT_H

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

/* String processing — 23 functions */

/** KMP search (returns first match index, or -1) */
int64_t bmb_kmp_search(void* text, void* pattern);

/** Substring find (returns first index, or -1) */
int64_t bmb_str_find(void* haystack, void* needle);

/** Substring reverse find (returns last index, or -1) */
int64_t bmb_str_rfind(void* haystack, void* needle);

/** Count non-overlapping occurrences */
int64_t bmb_str_count(void* haystack, void* needle);

/** Contains check (returns 1 or 0) */
int64_t bmb_str_contains(void* haystack, void* needle);

/** Starts with check (returns 1 or 0) */
int64_t bmb_str_starts_with(void* s, void* prefix);

/** Ends with check (returns 1 or 0) */
int64_t bmb_str_ends_with(void* s, void* suffix);

/** Byte find (returns index or -1) */
int64_t bmb_str_find_byte(void* s, int64_t needle);

/** Byte count */
int64_t bmb_str_count_byte(void* s, int64_t needle);

/** Palindrome check (returns 1 or 0) */
int64_t bmb_is_palindrome(void* s);

/** Token count by delimiter byte */
int64_t bmb_token_count(void* s, int64_t delim);

/** String reverse */
void* bmb_str_reverse(void* s);

/** String replace (first occurrence) */
void* bmb_str_replace(void* s, void* old_pat, void* new_pat);

/** String replace all */
void* bmb_str_replace_all(void* s, void* old_pat, void* new_pat);

/** Hamming distance (-1 if different lengths) */
int64_t bmb_str_hamming(void* a, void* b);

/** Word count (space-separated) */
int64_t bmb_word_count(void* s);

/** To uppercase */
void* bmb_str_to_upper(void* s);

/** To lowercase */
void* bmb_str_to_lower(void* s);

/** Trim whitespace */
void* bmb_str_trim(void* s);

/** Repeat string n times */
void* bmb_str_repeat(void* s, int64_t n);

/** String length (number of bytes) */
int64_t bmb_text_len(void* s);

/** Char at index (returns ASCII value, -1 if out of bounds) */
int64_t bmb_str_char_at(void* s, int64_t idx);

/** Compare two strings (0=equal, <0 if a<b, >0 if a>b) */
int64_t bmb_str_compare(void* a, void* b);

#ifdef __cplusplus
}
#endif

#endif /* BMB_TEXT_H */