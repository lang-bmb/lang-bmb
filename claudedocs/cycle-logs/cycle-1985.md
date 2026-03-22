# Cycle 1985-1988: bmb-json library creation
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1981: Next recommendation was bootstrap @export porting; deferred to create bmb-json first

## Scope & Implementation

### New Library: bmb-json (JSON parser/serializer)
Ported stdlib/json/mod.bmb zero-copy parser to FFI library with 8 @export functions:

| Function | Description | Return |
|----------|------------|--------|
| bmb_json_validate | Check if valid JSON | i64 (1/0) |
| bmb_json_stringify | Roundtrip normalization | String |
| bmb_json_type | Root value type | String |
| bmb_json_get | Object value by key (JSON) | String |
| bmb_json_get_string | Object string by key (unquoted) | String |
| bmb_json_get_number | Object number by key | i64 |
| bmb_json_array_len | Array length | i64 |
| bmb_json_array_get | Array element at index (JSON) | String |

### Design decisions
- String-in/String-out API for FFI compatibility
- Internal zero-copy parser re-serializes for output
- Supports: null, bool, number, string, array, object, nested structures, escapes
- Numbers are integer-only (i64) — f64 not supported in BMB yet

### Files created
- `ecosystem/bmb-json/src/lib.bmb`: 425 lines
- `ecosystem/bmb-json/bindings/python/bmb_json.py`: 8 functions + 39 tests
- `ecosystem/bmb-json/bmb_json.dll`: Shared library

## Review & Resolution
- bmb-json standalone: 9/9 outputs correct ✅
- bmb-json Python: 39/39 tests PASS ✅
- Cross-validated against Python json.loads/dumps for 5 complex cases ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: f64 support (BMB language limitation for JSON floats)
- Next Recommendation: bmb-algo expansion with more algorithms
