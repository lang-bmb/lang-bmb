# Cycle 3247: const NAME as Array Size (M11-C Phase 12)
Date: 2026-05-28

## Re-plan
Carry-Forward from Cycle 3246: Fix `const NAME` in array size position. Pattern: `let arr: [i64; SIZE]` was generating `alloca [%_t0 x i64]` (invalid LLVM IR) because `apply_const_map_to_mir` only replaced `"call @NAME()"` but NOT `"copy %NAME"` in MIR. Array declarations use `(var <NAME>)` → `copy %NAME` in MIR (not function calls).

## Scope & Implementation

### Root Cause Analysis
Pipeline trace for `const SIZE = 5; let arr: [i64; SIZE]`:
1. Parser: `SIZE` in array size → `(var <SIZE>)` AST → `lower_var_sb` → `copy %SIZE` in MIR
2. `apply_const_map_to_mir`: replaces `"call @SIZE()"` → `"const 5"` but NOT `"copy %SIZE"`
3. LLVM codegen: `copy %SIZE` + Cycle 3246 fix → `call i64 @SIZE()` → `%_t0 = call i64 @SIZE()`
4. `@stack_i64_new(%_t0)` → `alloca [%_t0 x i64]` — INVALID (array type requires constant)

For literal `5` case (`let arr: [i64; 5]`), the MIR has `%_t0 = const 5` (same form as our fix target). The existing optimization passes (`optimize_copy_prop`, `optimize_const_fn_eval`) convert this to `call @stack_i64_new(5)`, generating valid `alloca [5 x i64]`.

### Fix (compiler.bmb, apply_const_map_to_mir ~line 20745)
After replacing `"call @NAME()"` with `"const VALUE"`, also replace `"copy %NAME"` in MIR:

```bmb
-- Cycle 3247: Also replace copy %fn_name in MIR
-- SEP-terminated to avoid matching names that are prefixes of longer names
let copy_pat_mid = "copy %" + fn_name + SEP();
let copy_rep_mid = "const " + value + SEP();
let new_mir2 = replace_all_in_mir(new_mir, copy_pat_mid, copy_rep_mid);
-- Handle last instruction (no trailing SEP)
let copy_pat_end = "copy %" + fn_name;
let new_mir3 = if new_mir2.ends_with(copy_pat_end) {
    new_mir2.slice(0, new_mir2.len() - copy_pat_end.len()) + "const " + value
} else { new_mir2 };
apply_const_map_to_mir(new_mir3, const_map, comma_pos + 1)
```

After the fix, `copy %SIZE` becomes `const 5` in MIR. This is identical to the literal `5` case, so the same optimization passes handle the rest.

Safety: `"copy %SIZE" + SEP()` (not `"copy %SIZE"` alone) avoids matching `"copy %SIZE_BIG"` as a prefix.

### Golden Test Added
`tests/bootstrap/test_golden_const_array_size.bmb` — const SIZE=5 with explicit element assignment, sum=150 ✅

## Verification & Defect Resolution

- Golden tests: **31/31 PASS** (was 30, +1 new: `test_golden_const_array_size`)
- `cargo test --release`: **23/23 PASS**
- Stage 1 bootstrap: ✅ (compiler.bmb rebuilt successfully)
- Fixed Point (S3 IR == S4 IR): **diff 0** ✅

## Reflection

**Scope fit**: `const NAME` in array size now works. `let arr: [i64; SIZE]` generates valid LLVM IR.

**Latent defects**: None discovered. The SEP-boundary approach prevents false matches for all realistic const names.

**Structural improvement opportunities**:
- The Phase 9 belt-and-suspenders dead code (`val.ends_with("_cmp")` checks in `store_ptr_bool/u8/i32_sb`) still remains. Low priority.

**Philosophy drift**: None.

**Roadmap impact**: M11-C Phase 12 ✅ COMPLETE. `const` declarations are now fully functional: expression context (Phase 11) + array size context (Phase 12).

## Carry-Forward
- Actionable: None (const declarations fully fixed in both expression and array size contexts)
- Structural Improvement Proposals:
  - Phase 9 belt-and-suspenders dead code cleanup (low priority)
- Pending Human Decisions: None
- Roadmap Revisions: M11-C Phase 12 ✅ COMPLETE in ROADMAP.md
- Next Recommendation: Cycle 3248 — explore next language gap. Candidates: (a) `[T; N]` array with const size for other element types (f64, u8, i32, bool), (b) const-as-condition in if expressions, (c) const in function default argument context, (d) other M11-C features.
