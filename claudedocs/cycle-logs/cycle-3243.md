# Cycle 3243: Typed Pointer Parameter Subscript (*u8, *i32, *f64, *bool) — Phase 8
Date: 2026-05-28

## Re-plan
Plan valid; inherited scope from Cycle 3242 carry-forward: implement `arr: *u8`, `arr: *i32`, `arr: *f64`, `arr: *bool` pointer parameter subscript with correct element-size strides.

## Scope & Implementation

**Goal**: `fn fill_u8(arr: *u8, n: i64)` — subscript `arr[i]` with 1-byte stride, `arr: *i32` with 4-byte stride, `arr: *f64` with 8-byte double stride, `arr: *bool` with 1-byte stride.

**Key insight**: `get_child(child, 1)` for `(param <arr> *u8)` returns `"*"` only (operator-char scanner stops before `u8`). Must use `get_child(child, 2)` to get the type suffix.

### Files changed: `bootstrap/compiler.bmb`

1. **`parse_param` TK_STAR branch** (~line 5658): Extended to emit typed param AST:
   - `TK_I32` after `*` → `"(param <" + name + "> *i32)"`
   - `TK_I64` after `*` → `"(param <" + name + "> *i64)"`
   - `TK_F64` after `*` → `"(param <" + name + "> *f64)"`
   - `TK_BOOL` after `*` → `"(param <" + name + "> *bool)"`
   - (u8 was already handled)

2. **`rewrite_ptr_index`** (~line 13787): Rewrites `(index (var <arr>)...)` → typed variant based on `get_child(child, 2)`:
   - `"u8"` → `ptr_index_u8` / `set_ptr_index_u8`
   - `"i32"` → `ptr_index_i32` / `set_ptr_index_i32`
   - `"f64"` → `ptr_index_f64` / `set_ptr_index_f64`
   - `"bool"` → `ptr_index_bool` / `set_ptr_index_bool`
   - default (bare `"*"`, `"i64"`, struct name) → `ptr_index` / `set_ptr_index` (i64/8-byte stride)

### Golden test: `tests/bootstrap/test_golden_typed_ptr_param.bmb`
- `fill_u8(arr: *u8, n=4)`: fills [1,2,3,4], sum = 10 (s1)
- `sum_i32(arr: *i32, n=4)`: sums [10,20,30,40] = 100 (s2)
- `fill_f64/sum_f64_as_i64(arr: *f64, n=3)`: fills [1.0,2.0,3.0], sum=6 (s3)
- Expected: `s1+s2+s3 = 116`

## Verification & Defect Resolution

**First attempt (WRONG)**: `rewrite_ptr_index` checked `param_type_raw == "*u8"` etc. but `get_child(child, 1)` returns only `"*"` for all typed pointers (op-char scanner stops at ident). Symptom: wrong stride (i64 GEP for *u8), exit code 3, no output.

**Root cause fix**: Use `get_child(child, 2)` to get the type-suffix token (`"u8"`, `"i32"`, `"f64"`, `"bool"`, or `""` for bare pointer).

**Verification**:
- Golden test `test_golden_typed_ptr_param.bmb` → output `116` ✅
- Regression (Phase 3-7 golden tests): all 5 PASS ✅
- `cargo test --release`: 3800+2390+... all PASS ✅
- Stage 2 bootstrap: ✅
- Fixed Point (S2 IR == S3 IR): ✅

## Reflection

**Scope fit**: Typed pointer parameter subscript fully implemented for u8/i32/f64/bool primitive types with correct stride dispatch.

**Latent defects**: None found. The existing `ptr_index_u8`/`i32`/`f64`/`bool` MIR commands from Phases 4-7 are already correct; only the AST rewriting dispatch needed fixing.

**Structural**: `test_golden_stack_i64_subscript` still has no source file (only `.out`). Low priority — the Phase 3 behavior is covered by `test_golden_stack_i32_subscript` and other tests.

**Philosophy drift**: None. Correct stride per element type is essential for zero-overhead pointer arithmetic.

**Roadmap impact**: M11-C Phase 8 complete. Next: `arr: *` bare pointer (already existing), `*struct` pointer params, or `[f32; N]` (4-byte float) if needed.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: Create `test_golden_stack_i64_subscript.bmb` source (currently only `.out` exists)
- Pending Human Decisions: None
- Roadmap Revisions: M11-C Phase 8 ✅ COMPLETE in ROADMAP.md
- Next Recommendation: Cycle 3244 — explore next language gap (see ROADMAP.md §M11)
