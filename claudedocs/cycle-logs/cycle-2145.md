# Cycle 2145: C header generation script
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2143-2144: Carry-forward was "PyPI publishing, Linux/macOS, WASM."

## Scope & Implementation
Created `ecosystem/gen_headers.py`:
- Parses @export function signatures from .bmb source files
- Maps BMB types to C types (i64→int64_t, String→void*)
- Generates proper .h files with include guards, extern "C", FFI API declarations
- Extracts preceding comments as Doxygen-style docs

Generated headers for all 5 libraries:
| Library | Functions | Output |
|---------|-----------|--------|
| bmb-algo | 49 | include/bmb_algo.h |
| bmb-compute | 33 | include/bmb_compute.h |
| bmb-crypto | 11 | include/bmb_crypto.h |
| bmb-text | 23 | include/bmb_text.h |
| bmb-json | 12 | include/bmb_json.h |

## Review & Resolution
- Fixed Unicode arrow character causing Windows encoding error

## Carry-Forward
- Pending Human Decisions: None
- Next Recommendation: Test C headers with example program (cycle 2146-2147)
