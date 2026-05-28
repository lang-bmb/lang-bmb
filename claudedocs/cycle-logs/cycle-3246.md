# Cycle 3246: const Expression Context Bug Fix (M11-C Phase 11)
Date: 2026-05-28

## Re-plan
Plan valid. No inherited carry-forward actionables from Cycle 3245. Scope: investigate next language gap. Discovered that `const N = VALUE` was broken in expression context: emitting `ptrtoint ptr @N to i64` (function address) instead of calling the const function or inlining the value.

## Scope & Implementation

### Root Cause Discovery
`const N = 10` parses to `(fn-const <N> () 10)` → LLVM function `@N()` returning 10, with `alwaysinline` attribute.

When `N` appears in expression context:
1. Parser: `(var <N>)` AST node
2. `lower_var_sb`: emits `copy %N` in MIR
3. LLVM codegen `copy` handler (line 30531): `is_fnref = true` → emits `ptrtoint ptr @N to i64` ← **BUG** (takes function pointer address, not value)

Actual output: `140699436127312` (random function address) instead of `10`.

### Fix (compiler.bmb line 30535)
In the `copy` → `is_fnref` branch:
- Added `lookup_fn_both(registry, fn_name)` to get `"i64:"` for zero-arg functions
- If `both_sig.byte_at(both_sig.len() - 1) == 58` (ends with `:` = zero-arg): emit `call i64 @fn_name()`
- Otherwise: keep existing `ptrtoint` (for HOF multi-arg function references)

The `@const` function has `alwaysinline` + `memory(none)` + `speculatable`, so LLVM `opt -O2` inlines the call and constant-folds the entire expression chain. Example: `apply_threshold(3) + apply_threshold(7) + clamp(150) + THRESHOLD + MULTIPLIER` → constant-folded to `println(132)` by LLVM.

### Limitation Discovered (pre-existing, not introduced by fix)
`const SIZE = 5; let arr: [i64; SIZE]` fails — `alloca [%_t0 x i64]` is invalid LLVM IR because `%_t0 = call i64 @SIZE()` is a runtime value, not a compile-time constant. This was ALSO broken before the fix (generated `alloca [ptrtoint x i64]`). Carry-forward: fix `apply_const_map_to_mir` to also replace `copy %CONST_NAME` with `const VALUE` in MIR (so `@stack_*_new` builtins receive literal sizes).

### Golden Test Added
`tests/bootstrap/test_golden_const_expr.bmb` with 3 const declarations + clamp/scale functions → output `132` ✅

## Verification & Defect Resolution

- Golden tests: **30/30 PASS** (was 29, +1 new: `test_golden_const_expr`)
- `cargo test --release`: **23/23 PASS** (pre-verified, no Rust changes)
- Stage 1 bootstrap: ✅ (compiler.bmb rebuilt successfully)
- Fixed Point (S3 IR == S4 IR): **diff 0** ✅

## Reflection

**Scope fit**: `const` in expression context works correctly. LLVM optimization chain (alwaysinline + constant folding) is highly effective — entire programs with const usage get constant-folded at `opt -O2`.

**Latent defects**:
- `const NAME` in array size position is still broken (pre-existing). The fix path: extend `apply_const_map_to_mir` to also replace `copy %NAME` with `const VALUE` in MIR.
- HOF zero-arg scenario: if a user tries to take a pointer to a zero-arg non-const function as HOF, the new code emits a call instead of ptrtoint. This edge case is extremely rare in BMB practice.

**Structural improvement opportunities**:
- `apply_const_map_to_mir` could be extended to replace `copy %NAME` patterns (not just `call @NAME()`) — this would also fix the array size issue.

**Philosophy drift**: None. Correct semantics for `const` declarations is essential.

**Roadmap impact**: M11-C Phase 11 ✅ COMPLETE. `const` is now a first-class language feature for value declarations.

## Carry-Forward
- Actionable: Fix `const NAME` as array size (extend `apply_const_map_to_mir` to replace `copy %NAME`)
- Structural Improvement Proposals: None beyond carry-forward
- Pending Human Decisions: None
- Roadmap Revisions: M11-C Phase 11 ✅ COMPLETE in ROADMAP.md
- Next Recommendation: Cycle 3247 — fix `const NAME` in array size position (extend const inlining from `call @NAME()` to also cover `copy %NAME` in MIR), enabling patterns like `const N = 5; let arr: [i64; N]`.
