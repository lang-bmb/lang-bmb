# bmb-text — C Bindings

C bindings for the bmb-text library via direct shared library linking.

## Requirements

- GCC or Clang (C99+)
- `bmb_text.dll` / `libbmb_text.so` / `libbmb_text.dylib` in the parent directory (`../../`)

## Build & Run

```bash
make          # build test and example
make run-test # 33 tests, all pass
make run-example
```

### Manual build

```bash
# Windows
gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_text.dll

# Linux / macOS
gcc -O2 -I../../include -o example example.c -L../.. -lbmb_text -Wl,-rpath,../..
```

## Usage

```c
#include "bmb_text.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    /* Scalar functions: wrap in begin/end, free inputs */
    bmb_ffi_begin();
    void *hay = bmb_ffi_cstr_to_string("Hello, World!");
    void *ndl = bmb_ffi_cstr_to_string("World");
    int64_t idx = bmb_str_find(hay, ndl);  // 7
    bmb_ffi_free_string(hay);
    bmb_ffi_free_string(ndl);
    bmb_ffi_end();

    /* String-returning functions: read before ffi_end, do NOT free output */
    bmb_ffi_begin();
    void *in  = bmb_ffi_cstr_to_string("Hello");
    void *out = bmb_str_reverse(in);
    printf("%s\n", bmb_ffi_string_data(out)); // "olleH"
    bmb_ffi_free_string(in);
    /* out: arena-backed — do NOT call bmb_ffi_free_string(out) */
    bmb_ffi_end();

    return 0;
}
```

## Arena Rule

Return values of String-returning functions (`bmb_str_reverse`, `bmb_str_to_upper`,
`bmb_str_to_lower`, `bmb_str_trim`, `bmb_str_repeat`, `bmb_str_replace`,
`bmb_str_replace_all`) are **arena-allocated** and freed by `bmb_ffi_end()`.

```c
/* CORRECT */
bmb_ffi_begin();
void *in  = bmb_ffi_cstr_to_string("hello");
void *out = bmb_str_to_upper(in);
char buf[256];
strncpy(buf, bmb_ffi_string_data(out), sizeof(buf) - 1); /* copy if needed later */
bmb_ffi_free_string(in);   /* free input — safe */
/* bmb_ffi_free_string(out) — DO NOT do this */
bmb_ffi_end();

/* buf is still valid here */
```

## Function Reference

See `../../include/bmb_text.h` for all 23 functions.

### Scalar (int64_t return)
| Function | Description |
|----------|-------------|
| `bmb_str_find(hay, ndl)` | First index of needle, -1 if not found |
| `bmb_str_rfind(hay, ndl)` | Last index of needle |
| `bmb_str_count(hay, ndl)` | Count non-overlapping occurrences |
| `bmb_str_contains(hay, ndl)` | 1 if contains, 0 otherwise |
| `bmb_str_starts_with(s, pfx)` | 1 if starts with prefix |
| `bmb_str_ends_with(s, sfx)` | 1 if ends with suffix |
| `bmb_kmp_search(text, pat)` | KMP first match index |
| `bmb_str_find_byte(s, byte)` | First index of byte value |
| `bmb_str_count_byte(s, byte)` | Count of byte value |
| `bmb_is_palindrome(s)` | 1 if palindrome |
| `bmb_token_count(s, delim)` | Token count by delimiter byte |
| `bmb_str_hamming(a, b)` | Hamming distance, -1 if different lengths |
| `bmb_word_count(s)` | Space-separated word count |
| `bmb_text_len(s)` | Byte length |
| `bmb_str_char_at(s, idx)` | ASCII value at index, -1 if OOB |
| `bmb_str_compare(a, b)` | strcmp-style comparison |

### String-returning (arena-backed output)
| Function | Description |
|----------|-------------|
| `bmb_str_reverse(s)` | Reverse string |
| `bmb_str_to_upper(s)` | Uppercase |
| `bmb_str_to_lower(s)` | Lowercase |
| `bmb_str_trim(s)` | Strip leading/trailing whitespace |
| `bmb_str_repeat(s, n)` | Repeat n times |
| `bmb_str_replace(s, old, new)` | Replace first occurrence |
| `bmb_str_replace_all(s, old, new)` | Replace all occurrences |
