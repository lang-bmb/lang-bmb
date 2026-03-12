# Cycle 1866: Inline f64/i32 Load/Store + nocapture Fix + norecurse Dedup
Date: 2026-03-12

## Inherited → Addressed
From 1865: GEP-based addressing (deferred — too complex for single cycle), load_f64/store_f64 inline codegen missing, loop metadata not yet added.

## Scope & Implementation

### 1. Inline load_f64/store_f64/load_i32/store_i32 Codegen
Added inline LLVM IR generation for 4 memory builtins in BOTH `llvm_gen_call` and `llvm_gen_call_reg`:
- `store_f64(addr, val)` → `inttoptr + store double, !tbaa !905`
- `load_f64(addr)` → `inttoptr + load double + bitcast double→i64, !tbaa !905`
- `store_i32(addr, val)` → `inttoptr + trunc i64→i32 + store i32`
- `load_i32(addr)` → `inttoptr + load i32 + sext i32→i64`

Previously these went through runtime function calls. Now they generate inline IR with proper TBAA metadata.

### 2. Duplicate norecurse Fix
`bmb_user_main` had `norecurse` applied twice: once from `base_attrs` (Cycle 1865) and again from `add_norecurse_attr` pass. Fixed by excluding `fn_name == "main"` from the norecurse pass.

### 3. Inkwell nocapture→none Bug Fix (CRITICAL)
**Root cause**: In LLVM 21, the `nocapture` enum attribute was replaced by `captures(none)`. `Attribute::get_named_enum_kind_id("nocapture")` returns 0, so `create_enum_attribute(0, 0)` created the `none` attribute instead.

**Impact**: All String parameters in inkwell-generated IR had `ptr none` instead of `ptr nocapture`, causing `opt` to reject the IR. This blocked ALL inkwell-based `--emit-ir` builds and was the root cause of the segfault when building compiler.bmb with inkwell (the segfault was actually during LLVM module verification, not a memory issue).

**Fix**: Guard `nocapture_id != 0`; fallback to string attribute `"nocapture"` for LLVM 21+.

Files changed: `bootstrap/compiler.bmb`, `bmb/src/codegen/llvm.rs`

## Review & Resolution
- `cargo test --release`: 6,186 tests PASS (3762 + 47 + 2354 + 23)
- **3-Stage Bootstrap**: Fixed Point VERIFIED (S2 == S3 at 108,418 lines)
- Golden test (accumulator): PASS — correct output (5050)
- IR verification: TBAA f64 metadata, inline load/store, single norecurse, no `ptr none`
- IR size: 107,567 → 108,418 lines (+851, from inline expansion of load/store builtins)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope:
  - GEP-based addressing (replace inttoptr) — HIGH IMPACT, requires instruction chain analysis or post-processing pass
  - Loop metadata (vectorize.enable/width) not yet added to bootstrap
  - Ptr-provenance (dual-alloca for malloc results) not yet in bootstrap
  - Text backend also emits `nocapture` which may need updating for LLVM 21 (currently only used in declarations where it's quoted as string attributes)
  - store_f64 type safety: MIR doesn't track whether a register holds i64 or double — inline store_f64 assumes double input, which is correct for f64 arithmetic chains but breaks for direct load_f64→store_f64 (bitcast double→i64→store double mismatch). This is the same limitation as the old runtime call path.
- Next Recommendation: Runtime modularization (Phase A from ROADMAP) or loop metadata addition
