# Cycle 1843: Fix Lambda String Builder Encoding — All 18 Lambda/Closure Tests Fixed
Date: 2026-03-11

## Inherited → Addressed
From 1842: "Whether to implement closures/generics in bootstrap compiler (would resolve remaining 18 golden test failures)"

## Scope & Implementation

### Root Cause
`lower_lambda_sb()` in `compiler.bmb` created a raw `sb_new()` builder and passed it directly to `lower_expr_sb()`, which expects an encoded value (`sb_raw * 2 + safe_bit`). When `sb_push_mir()` decoded with `sb / 2`, dividing a raw pointer by 2 gave an invalid address → segfault.

The lambda/closure infrastructure (parser, lowering, codegen) was already fully implemented — the ONLY bug was this string builder encoding mismatch.

### Fix
Changed one line:
```bmb
// Before (BUG):
let lambda_sb = sb_new();
// After (FIX):
let lambda_sb_raw = sb_new();
let lambda_sb = lambda_sb_raw * 2;
```
Also changed `sb_build(lambda_sb)` → `sb_build(lambda_sb_raw)` to use the raw pointer for building.

### Files Changed
- `bootstrap/compiler.bmb` — Fixed `lower_lambda_sb()` string builder encoding (2 lines changed)

### Verification
- **Golden tests**: 2797 → 2814/2815 PASS (1 transient failure on ackermann, passes manually)
- **All 18 lambda/closure/generic tests now PASS**
- **Rust tests**: 6,186 pass
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (S2 == S3, 54s)

## Review & Resolution
- The fix is minimal (2 lines) but resolves all 18 remaining golden test failures
- Lambda infrastructure (parser lines 1143-1180, lowering 4333-4367, codegen 13755-13968) was already complete
- The bug was introduced in v0.96.20 when string builder encoding was added but lambda code wasn't updated

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: ackermann golden test shows transient compile failure in full suite run (passes in isolation)
- Next Recommendation: Evaluate early termination — golden tests at 99.9% pass rate
