# Cycle 37: SHA-256 Implementation (Pure BMB)

## Date
2026-02-07

## Scope
Implement FIPS 180-4 SHA-256 hash function as pure BMB library. First cryptographic library in BMB ecosystem.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

Proves BMB handles complex bitwise computation. Pure algorithms, zero FFI.

## Research Summary
- SHA-256 spec: FIPS 180-4 (NIST), 8 initial hash values, 64 round constants, 64-round compression ([NIST](https://csrc.nist.gov/pubs/fips/180-4/upd1/final))
- All arithmetic mod 2^32: emulated via `band MASK32()` since BMB only has i64
- Bitwise operators are infix in BMB: `a band b`, `a bor b`, `a bxor b`
- `store_i64()` and `free()` return `()` (unit) — requires wrapper functions for use in i64-returning contexts

## Implementation
- New package: `ecosystem/gotgan-packages/packages/bmb-sha256/`
- 338 LOC in `src/lib.bmb`
- Key design decisions:
  - `st()` and `drop()` wrappers for `store_i64` and `free` builtins (return unit → i64)
  - `aput`/`aget` for typed array access (one i64 per slot)
  - `bput`/`bget` for byte-level buffer access
  - Recursive style for message schedule fill/extend and compression rounds
  - Big-endian byte packing/unpacking for SHA-256 word format
  - All 64 round constants as conditional chain in `K()` function

### Files Added
- `ecosystem/gotgan-packages/packages/bmb-sha256/gotgan.toml`
- `ecosystem/gotgan-packages/packages/bmb-sha256/src/lib.bmb`

### Also Filed Issues (from Phase 1)
- `claudedocs/issues/ISSUE-20260207-let-in-while-block.md` (HIGH)
- `claudedocs/issues/ISSUE-20260207-free-returns-unit.md` (MEDIUM)

## Test Results
- bmb-sha256: 9/9 tests passed (NIST vectors + property tests)
- Rust tests: 694/694 passed (no regressions)

Test vectors verified:
- Empty string, "abc", "hello", "a" (single char)
- 56-byte boundary case (two-block padding)
- 448-bit NIST vector (multi-block)
- Determinism, collision resistance, output length

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | All NIST vectors pass |
| Architecture | 8/10 | Clean package structure, follows gotgan conventions |
| Philosophy Alignment | 9/10 | Pure BMB, zero dependencies, proves bitwise capability |
| Test Quality | 7/10 | 9 tests; could add longer multi-block inputs |
| Documentation | 7/10 | Usage comments but no full API docs |
| Code Quality | 8/10 | Compact, idiomatic; wrappers needed for builtins |
| **Average** | **8.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | store_i64/free return unit, requiring wrapper functions | Known bug (ISSUE-20260207) |
| I-02 | L | No multi-block (>64 byte) test vector beyond 56/448-bit | Add in next cycle |
| I-03 | L | No performance benchmark vs C SHA-256 | Phase 2 |
| I-04 | M | Type error spans point to wrong locations | Known compiler issue |

## Next Cycle Recommendation
1. Add comprehensive test coverage to existing stub packages (bmb-rand, bmb-fmt, bmb-math)
2. Improve bmb-rand with XorShift64* PRNG (currently stub LCG only)
3. Add missing tests to 6 packages identified as having zero tests
