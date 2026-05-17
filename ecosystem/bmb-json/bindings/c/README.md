# bmb-json — C Bindings

C bindings for the bmb-json library via direct shared library linking.

## Requirements

- GCC or Clang (C99+)
- `bmb_json.dll` / `libbmb_json.so` / `libbmb_json.dylib` in the parent directory (`../../`)

## Build & Run

```bash
make          # build test and example
make run-test # 28 tests, all pass
make run-example
```

### Manual build

```bash
# Windows
gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_json.dll

# Linux / macOS
gcc -O2 -I../../include -o example example.c -L../.. -lbmb_json -Wl,-rpath,../..
```

## Usage

```c
#include "bmb_json.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    const char *json = "{\"name\":\"Alice\",\"age\":30}";

    /* Scalar functions */
    bmb_ffi_begin();
    void *obj = bmb_ffi_cstr_to_string(json);
    void *key = bmb_ffi_cstr_to_string("age");
    printf("valid: %lld\n", (long long)bmb_json_validate(obj));  // 1
    printf("age:   %lld\n", (long long)bmb_json_get_number(obj, key)); // 30
    bmb_ffi_free_string(obj);
    bmb_ffi_free_string(key);
    bmb_ffi_end();

    /* String-returning functions: read before ffi_end, do NOT free output */
    bmb_ffi_begin();
    obj = bmb_ffi_cstr_to_string(json);
    key = bmb_ffi_cstr_to_string("name");
    void *out = bmb_json_get_string(obj, key);
    printf("name: %s\n", bmb_ffi_string_data(out)); // Alice
    bmb_ffi_free_string(obj);
    bmb_ffi_free_string(key);
    /* out: arena-backed — do NOT call bmb_ffi_free_string(out) */
    bmb_ffi_end();

    return 0;
}
```

## Arena Rule

String-returning functions (`bmb_json_stringify`, `bmb_json_type`, `bmb_json_get`,
`bmb_json_get_string`, `bmb_json_array_get`) return **arena-allocated** strings.

- **DO NOT** call `bmb_ffi_free_string` on them.
- **DO** read or copy the data before calling `bmb_ffi_end()`.

## Notes on `bmb_json_count`

`bmb_json_count` counts all nodes recursively **including the root**:
- `[1,2,3]` → 4 (1 root + 3 elements)
- `{"a":1}` → 3 (1 root + 1 key + 1 value)

## Function Reference

See `../../include/bmb_json.h` for all 12 functions.

### Scalar (int64_t return)
| Function | Description |
|----------|-------------|
| `bmb_json_validate(s)` | 1 if valid JSON, 0 otherwise |
| `bmb_json_get_number(s, key)` | Get number by key (0 if missing) |
| `bmb_json_array_len(s)` | Array length (-1 if not array) |
| `bmb_json_has_key(s, key)` | 1 if key exists, 0 otherwise |
| `bmb_json_object_len(s)` | Number of keys (-1 if not object) |
| `bmb_json_get_bool(s, key)` | 1/0/-1 (true/false/missing) |
| `bmb_json_count(s)` | Total node count (recursive, includes root) |

### String-returning (arena-backed)
| Function | Description |
|----------|-------------|
| `bmb_json_stringify(s)` | Parse + re-serialize (normalize) |
| `bmb_json_type(s)` | "null","bool","number","string","array","object" |
| `bmb_json_get(s, key)` | Value by key (JSON representation) |
| `bmb_json_get_string(s, key)` | String value by key (no quotes) |
| `bmb_json_array_get(s, idx)` | Array element as JSON string |
