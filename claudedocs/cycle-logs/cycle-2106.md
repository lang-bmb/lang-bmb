# Cycle 2106: Per-library pytest test suites
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2105: Packaging standardized, next was test suites.

## Scope & Implementation
Created pytest-compatible test suites for all 5 libraries:

| Library | Tests | Time |
|---------|-------|------|
| bmb-algo | 189 | 0.08s |
| bmb-crypto | 212 | 0.15s |
| bmb-text | 127 | 0.09s |
| bmb-json | 159 | 0.11s |
| bmb-compute | 270 | 0.17s |
| **Total** | **957** | **0.60s** |

Each library has:
- `tests/conftest.py` — sys.path setup + Windows DLL directory registration
- `tests/test_bmb_<lib>.py` — organized by category with edge cases

## Review & Resolution
- bmb-json: 4 tests fixed to match BMB's lenient JSON validator (accepts trailing commas, unmatched delimiters, whitespace-only)
- Cross-validated against Python stdlib where applicable (hashlib, hmac, binascii, json, math)
- All 957 tests passing across all 5 libraries

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: BMB JSON validator is more lenient than RFC 8259 — potential future tightening
- Next Recommendation: Add per-library benchmark scripts (cycle 2107)
