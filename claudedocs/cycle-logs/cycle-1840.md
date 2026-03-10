# Cycle 1840: Add bmb_abs/bmb_min/bmb_max/bmb_clamp Intrinsic Mapping
Date: 2026-03-11

## Inherited → Addressed
From 1839: "3 opt failures from missing runtime declarations (bmb_clamp, int methods, method_chain)" + benchmark regression check.

## Scope & Implementation

### Root Cause
The bootstrap compiler handled `@abs`, `@min`, `@max`, `@clamp` → LLVM intrinsics, but NOT `@bmb_abs`, `@bmb_min`, `@bmb_max`, `@bmb_clamp`. Method-call syntax (e.g., `(-42).abs()`) lowers to `bmb_abs` in MIR, which the codegen didn't recognize.

### Fix
Added `or fn_name == "@bmb_abs"` (and similar for min/max/clamp) to both `llvm_gen_call` and `llvm_gen_call_reg` codegen paths in `compiler.bmb`.

### Files Changed
- `bootstrap/compiler.bmb` — Extended `llvm_gen_call` and `llvm_gen_call_reg` for bmb_abs/min/max/clamp

### Verification
- **Golden tests**: 2794 → 2797 PASS (27 → 24 FAIL, +3 tests fixed)
- **Remaining 24 failures**: 6 file not found + 18 compile (closures/generics) — all pre-existing infrastructure
- **Zero wrong-output or runtime failures**
- **Benchmarks**: Not affected (benchmarks use Rust compiler, not bootstrap)
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (S2 == S3, 52s)

## Review & Resolution
- No new defects found
- Cumulative improvement: 39 → 24 failures across Cycles 1838-1840

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: 6 golden test files referenced in manifest but don't exist on disk
- Next Recommendation: Investigate remaining 6 "file not found" tests — remove from manifest or create the test files
