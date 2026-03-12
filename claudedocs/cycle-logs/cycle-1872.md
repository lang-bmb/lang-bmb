# Cycle 1872: Bootstrap GEP inbounds nuw — Complete Coverage

Date: 2026-03-12

## Inherited → Addressed
Cycle 1871: Runtime init fix for inline main wrapper. GEP missing `inbounds nuw` identified as codegen gap.

## Scope & Implementation

### GEP `inbounds nuw` in bootstrap codegen
- Added `inbounds nuw` to ALL GEP instructions in both `compiler.bmb` and `llvm_ir.bmb`
- **compiler.bmb GEP sites updated (10)**:
  - Array element GEP (line 6084)
  - String struct data pointer (line 6452, 6690)
  - String struct length (line 6455, 6693)
  - String byte access (line 6452, 6690)
  - Struct GEP (line 13927)
  - Closure env count store (line 14048)
  - Closure capture stores (line 14073)
  - Closure load (line 14090)
  - Field access (line 14142)
  - Field store (line 14257)
- **llvm_ir.bmb GEP sites updated (9)**:
  - Capture store/load (lines 695, 701)
  - Closure fn/env extract (lines 776, 781)
  - Array literal stores (line 832)
  - Index unchecked (line 889)
  - Receiver len/data/byte access (lines 1361, 1372, 1374)
- Updated test assertions to match new `inbounds nuw` pattern

### Files Changed
- `bootstrap/compiler.bmb` — 10 GEP sites + `inbounds nuw`
- `bootstrap/llvm_ir.bmb` — 9 GEP sites + `inbounds nuw` + 3 test assertion updates

## Review & Resolution
- **Rust tests**: 6,186/6,186 PASS
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (108,519 lines, S2 == S3)
- **GEP coverage**: 2,109/2,109 GEP instructions have `inbounds nuw` (100%)
- Previously 0% had either flag in bootstrap-generated IR

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Review remaining codegen gaps (nonnull on string params, speculatable on read-only functions)
