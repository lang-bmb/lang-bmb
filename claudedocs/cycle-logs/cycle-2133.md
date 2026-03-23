# Cycle 2133: Add __all__ to all 5 libraries
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2131-2132: bmb-json expansion done.

## Scope & Implementation
Added explicit `__all__` lists to all 5 Python binding modules:
- bmb_algo: 40 entries
- bmb_compute: 31 entries
- bmb_crypto: 11 entries
- bmb_text: 23 entries
- bmb_json: 12 entries

This controls what `from bmb_xxx import *` exports, preventing internal symbols (_lib, _arr, helpers) from leaking.

## Review & Resolution
- All 957 tests pass
- No defects found

## Carry-Forward
- Pending Human Decisions: None
- Next Recommendation: Update .pyi stubs for new functions (cycle 2134)
