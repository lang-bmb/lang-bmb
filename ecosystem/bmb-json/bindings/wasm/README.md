# bmb-json WASM Bindings

WebAssembly build of bmb-json for Node.js and browser use.

## Status: Experimental

The WASM output is generated but requires additional work for practical use:
- String parameters need a memory management bridge (BMB strings are heap-allocated handles)
- WASI imports need to be provided or polyfilled for browser environments
- A JS glue layer is needed to convert JS strings to/from BMB string handles

## Building

```bash
# Generate WAT (WebAssembly Text) file
bmb build ecosystem/bmb-json/src/lib.bmb --emit-wasm -o bmb_json.wat

# Convert WAT to WASM binary (requires wabt tools)
wat2wasm bmb_json.wat -o bmb_json.wasm
```

## Exported Functions

All 12 bmb-json functions are exported:
- `bmb_json_validate(input: i64) -> i64`
- `bmb_json_stringify(input: i64) -> i64`
- `bmb_json_type(input: i64) -> i64`
- `bmb_json_get(input: i64, key: i64) -> i64`
- `bmb_json_get_string(input: i64, key: i64) -> i64`
- `bmb_json_get_number(input: i64, key: i64) -> i64`
- `bmb_json_array_len(input: i64) -> i64`
- `bmb_json_array_get(input: i64, idx: i64) -> i64`
- `bmb_json_has_key(input: i64, key: i64) -> i64`
- `bmb_json_object_len(input: i64) -> i64`
- `bmb_json_get_bool(input: i64, key: i64) -> i64`
- `bmb_json_count(input: i64) -> i64`

Parameters are BMB String handles (i64), not raw string pointers. A JS wrapper must allocate strings in WASM memory and convert to handles.
