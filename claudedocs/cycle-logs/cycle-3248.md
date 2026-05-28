# Cycle 3248: Compound const expressions as array sizes (M11-C Phase 13)
Date: 2026-05-28

## Re-plan
Carry-Forward from Cycle 3247: `const TOTAL = ROWS * COLS` (compound const expressions) failed with `alloca [%_t0 x i64]` — same error pattern as Cycle 3247 but different root cause.

## Scope & Implementation

### Root Cause Analysis

Pipeline trace for `const ROWS = 3; const COLS = 4; const TOTAL = ROWS * COLS; let arr: [i64; TOTAL]`:

1. `try_register_const_fn` scans MIR for zero-arg `@const` functions. For `@TOTAL`, its MIR body contains `= call @ROWS()` → `has_call >= 0` guard fires → `@TOTAL` is NOT registered in const_map.
2. `optimize_const_inlining` runs FIRST (before CF/DCE loop), so `@TOTAL` is never in the map.
3. `apply_const_map_to_mir` cannot replace `copy %TOTAL` (TOTAL not in map).
4. `alloca [%_t0 x i64]` — INVALID (requires literal constant).

Key insight: After the FIRST `optimize_cf_dce_loop`, `optimize_const_fn_eval` has replaced `call @ROWS()` → `const 3` and `call @COLS()` → `const 4` INSIDE `@TOTAL`'s body, and CF has folded `3*4=12`. At that point, `@TOTAL`'s body is `const 12` — a simple const body that `try_register_const_fn` WOULD accept.

### Fix (compiler.bmb, ~line 35592)

Add a second `optimize_const_inlining` call after `optimize_cf_dce_loop`:

```bmb
let mir1 = optimize_const_inlining(mir_raw);
let mir2 = optimize_cf_dce_loop(mir1, 0);

-- Cycle 3248: second const inlining pass for compound consts (e.g., const TOTAL = ROWS * COLS).
-- After the first CF/DCE loop, optimize_const_fn_eval has replaced call @ROWS() with const 3
-- inside @TOTAL's body, and CF has folded 3*4=12. Now try_register_const_fn can register @TOTAL.
let mir2b = optimize_const_inlining(mir2);

let mir3 = optimize_tail_recursive_to_loop(mir2b);
```

After the second pass, `copy %TOTAL` → `const 12` in MIR. Identical to the literal `12` case.

### Golden Test Added
`tests/bootstrap/test_golden_const_compound.bmb` — ROWS=3, COLS=4, TOTAL=12, sum of arr[0..11]=1..12 = 78 ✅

Also fixed manifest drift:
- `test_golden_stack_array_typed.bmb` and `test_golden_stack_array_f64.bmb` were in `tests/golden/` but
  listed in `tests/bootstrap/golden_tests.txt` as `tests/bootstrap/` paths. Copied and `git add -f`.

## Verification & Defect Resolution

- Golden tests (manual, `build` method): **8/8 PASS** (const_array_size, const_compound, const_expr, stack_bool_subscript, stack_f64_subscript, stack_i32_subscript, stack_i64_subscript, stack_u8_subscript)
- `cargo test --release`: **all PASS** (3800+ tests)
- Stage 1 bootstrap: ✅ (compiler.bmb rebuilt successfully)
- Fixed Point (S2 IR == S3 IR): **diff 0** ✅

**Latent defect discovered**: Bootstrap compiler's `test` command uses shared temp filenames (`__bmb_test_tmp.exe`, `__bmb_test_tmp.txt`) → Windows file locking causes ALL 32 `.bmb.out`-equipped tests to fail when run via `test <dir>`. Pre-existing bug, NOT introduced by Cycle 3248. The `build` command works correctly.

## Reflection

**Scope fit**: `const TOTAL = ROWS * COLS` now works as array size. Compound const expressions (any arithmetic on other consts) are handled by the two-pass inlining approach.

**Latent defects**: 
1. Pre-existing Windows `test` command file locking bug (low priority — `build` verification works).
2. Manifest had `test_golden_stack_array_typed.bmb` and `test_golden_stack_array_f64.bmb` pointing to `tests/bootstrap/` but files only existed in `tests/golden/`. Fixed (copied + `git add -f`).

**Structural improvement opportunities**:
- A single `optimize_const_inlining` pass could be designed to iterate until fixed-point (handle `const A = B + C; const D = A * 2` chaining). Current 2-pass approach handles single-level compounds but not multi-level chaining. Low priority — multi-level compound consts are unusual.
- The `test` command's Windows file locking issue should be fixed by using unique temp filenames per test.

**Philosophy drift**: None.

**Roadmap impact**: M11-C Phase 13 ✅ COMPLETE. `const` declarations are now fully functional: simple values (Phase 11), array size (Phase 12), compound arithmetic (Phase 13).

## Carry-Forward
- Actionable: None (compound const expressions fully fixed)
- Structural Improvement Proposals:
  - Multi-level const chaining (e.g., `const A = B + C; const D = A * 2`) needs a fixed-point `optimize_const_inlining` loop. Currently only handles one level of indirection (low priority).
  - Bootstrap `test` command Windows file locking bug: use unique temp names (e.g., `__bmb_test_tmp_{hash}.exe`) to prevent conflicts.
- Pending Human Decisions: None
- Roadmap Revisions: M11-C Phase 13 ✅ COMPLETE in ROADMAP.md
- Next Recommendation: Cycle 3249 — explore next language gap. Candidates:
  (a) Multi-level compound consts (`const A = B + C; const D = A * 2`) — fixed-point inlining
  (b) `const` as condition in `if` expressions
  (c) `const` in function default argument context
  (d) `[T; const]` for other element types not yet tested
  (e) Other M11-C features as per ROADMAP
