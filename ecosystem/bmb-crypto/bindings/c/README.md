# bmb-crypto — C Bindings

C bindings for the bmb-crypto library via direct shared library linking.

## Requirements

- GCC or Clang (C99+)
- `bmb_crypto.dll` / `libbmb_crypto.so` / `libbmb_crypto.dylib` in the parent directory (`../../`)

## Build & Run

```bash
make          # build test and example
make run-test # 23 tests, all pass
make run-example
```

### Manual build

```bash
# Windows
gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_crypto.dll

# Linux / macOS
gcc -O2 -I../../include -o example example.c -L../.. -lbmb_crypto -Wl,-rpath,../..
```

## Usage

```c
#include "bmb_crypto.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    /* Hash a string */
    bmb_ffi_begin();
    void *in  = bmb_ffi_cstr_to_string("hello");
    void *out = bmb_sha256(in);
    printf("sha256: %s\n", bmb_ffi_string_data(out));
    bmb_ffi_free_string(in);   /* free the INPUT — it's heap-allocated */
    /* do NOT free out — it is arena-allocated by the BMB runtime */
    bmb_ffi_end();             /* arena is freed here */

    return 0;
}
```

## CRITICAL: Arena allocation rule

All return values from `@export` functions (sha256, base64_encode, etc.) are
**arena-allocated** by the BMB runtime. They live until `bmb_ffi_end()` is called.

**Rules:**
- `bmb_ffi_cstr_to_string(...)` → **heap-allocated** → MUST call `bmb_ffi_free_string`
- Return value of any `bmb_*` function → **arena-allocated** → DO NOT call `bmb_ffi_free_string`

**Read the data before `bmb_ffi_end()`** — the arena is freed at that point.

```c
/* CORRECT */
bmb_ffi_begin();
void *in  = bmb_ffi_cstr_to_string("hello");
void *out = bmb_sha256(in);
char buf[256];
strncpy(buf, bmb_ffi_string_data(out), sizeof(buf) - 1); /* copy before end */
bmb_ffi_free_string(in);   /* free input */
/* out: do NOT free */
bmb_ffi_end();             /* buf is still valid, out is now gone */

/* WRONG — double-free / heap corruption */
bmb_ffi_free_string(out);  /* ❌ never do this */
```

## API

All 14 functions take BMB String inputs and return BMB String outputs:

| Function | Description |
|----------|-------------|
| `bmb_sha256(s)` | SHA-256 hex digest |
| `bmb_md5(s)` | MD5 hex digest |
| `bmb_hmac_sha256(key, msg)` | HMAC-SHA256 hex |
| `bmb_base64_encode(s)` | Base64 encode |
| `bmb_base64_decode(s)` | Base64 decode |
| `bmb_base32_encode(s)` | Base32 encode |
| `bmb_base32_decode(s)` | Base32 decode |
| `bmb_hex_encode(s)` | Hex encode |
| `bmb_hex_decode(s)` | Hex decode |
| `bmb_rot13(s)` | ROT-13 transform |
| `bmb_crc32(s)` | CRC-32 hex |
| `bmb_adler32(s)` | Adler-32 hex |
| `bmb_fletcher16(s)` | Fletcher-16 hex |
| `bmb_xor_checksum(s)` | XOR checksum hex |

See `../../include/bmb_crypto.h` for the full declarations.
