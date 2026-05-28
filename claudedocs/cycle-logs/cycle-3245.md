# Cycle 3245: f64 Array Comparison/Integer Store Bug Fix + Missing Test Source (M11-C Phase 10)
Date: 2026-05-28

## Re-plan
Inherited scope from Cycle 3244 Carry-Forward: `test_golden_stack_i64_subscript` had no source file (only `.out`). Scope Adjust mid-cycle: while verifying the newly created source file, discovered P0 bug — `llvm_gen_store_ptr_f64_sb` generates invalid LLVM IR when storing non-double values (comparison results, integer literals) into `[f64; N]` arrays. Priority shifted to fix this bug first.

## Scope & Implementation

### Part 1: Missing Source File
Created `tests/bootstrap/test_golden_stack_i64_subscript.bmb`:
```bmb
-- Test: arr[i] subscript for [i64; N] stack arrays (M11-C Phase 3)
fn main() -> i64 = {
    let arr: [i64; 5];
    let mut i = 0;
    while i < 5 { set arr[i] = i * i; i = i + 1 };
    let sum = arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
    println(sum)  -- [0,1,4,9,16] sum = 30
};
```
Output: `30` ✅

### Part 2: P0 Bug Fix — `llvm_gen_store_ptr_f64_sb`

**Root Cause**: `llvm_gen_store_ptr_f64_sb` (line 14888) used `_str_sb: i64` (ignored parameter) and blindly emitted `store double %val` regardless of the actual type of `val`. When val is a comparison result (`%_t8` = `zext i1 to i64`) or integer literal, this produces invalid LLVM IR:
```
error: '%_t8' defined with type 'i64' but expected 'double'
  store double %_t8, ptr %_t9, align 8
```

**Fix** (`compiler.bmb` line 14888-14923):
- Changed `_str_sb: i64` → `str_sb: i64` (activate the parameter)
- Added three-branch logic:
  1. Register (`val` starts with `%`) + in double_marker → store directly
  2. Register (`val` starts with `%`) + NOT in double_marker → emit `sitofp i64 val to double` first
  3. Float literal (contains `.`) → store directly
  4. Integer literal (no `.`) → emit `sitofp i64 val to double` first

**Generated IR for comparison result case**:
```llvm
%_t8_cmp = icmp slt i64 %_t6, 2
%_t8 = zext i1 %_t8_cmp to i64
%_t9 = getelementptr inbounds nuw double, ptr %_t9_gp, i64 %_t5
%_t9_si = sitofp i64 %_t8 to double    ← NEW
store double %_t9_si, ptr %_t9, align 8 ← correct
```

### Part 3: New Golden Test
Created `tests/bootstrap/test_golden_cmp_store_f64_i64.bmb` + `.out` (output: `4`):
- `fill_f64_lt(arr: *f64, n)`: `set arr[i] = (i < 2)` → [1.0, 1.0, 0.0, 0.0], sum = 2.0
- `fill_i64_gt(arr: *i64, n)`: `set arr[i] = (i > 1)` → [0, 0, 1, 1], sum = 2
- Expected: `(sf as i64) + si = 2 + 2 = 4` ✅

Also regenerated `test_golden_cmp_store_f64_i64.ll` to capture correct IR.

## Verification & Defect Resolution

- Golden tests: **29/29 PASS** (was 27, +2 new: `test_golden_stack_i64_subscript` + `test_golden_cmp_store_f64_i64`)
- `cargo test --release`: **23/23 PASS**
- Stage 1 bootstrap: ✅ (`target/release/bmb.exe build compiler.bmb` success)
- Fixed Point (S3 IR == S4 IR): **diff 0** ✅

Note: `--emit-ir` + link step fails with "Permission denied" on `bootstrap/compiler.exe` (Windows file locking — same exe being compiled). Workaround: IR file is generated to `bootstrap/compiler.ll` before linker runs; copy to s3.ll/s4.ll for comparison.

## Reflection

**Scope fit**: Both planned item (missing source) and discovered P0 bug fully resolved.

**Latent defects**:
- The Phase 9 belt-and-suspenders dead code (`val.ends_with("_cmp")` checks in `store_ptr_bool/u8/i32_sb`) remain. These check the wrong variable name (val is `%_t8`, not `%_t8_cmp`). They're harmless (no-op) but misleading. Carry-forward for cleanup.
- `is_double_var_sb` for float literals: the function only checks the double_marker registry. A literal like `0x4000000000000000` (hex float) would fall through to `sitofp` branch. Acceptable: hex float literals are not used in BMB source.

**Structural improvement opportunities**:
- The double_marker tracking mechanism could be extended to track the type of literals directly (e.g., add float literals to the registry when parsed). Low priority — the current literal detection (contains `.`) is sufficient.

**Philosophy drift**: None. Correct type handling is essential correctness.

**Roadmap impact**: M11-C Phase 10 ✅ COMPLETE. The f64 comparison/integer store bug was masked because the common case (float literals, double expressions) worked correctly. The Phase 9 fix exposed this by making comparison results more visible.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - Phase 9 belt-and-suspenders dead code in `store_ptr_bool/u8/i32_sb` (`val.ends_with("_cmp")` checks) should be removed — they check wrong variable name and never trigger. Low priority.
  - Windows file locking: `bootstrap/compiler.exe` locked during Stage 2 build (linker fails). Workaround: use compiler.ll directly. Structural fix would require outputting to a temp file. Not blocking.
- Pending Human Decisions: None
- Roadmap Revisions: M11-C Phase 10 ✅ COMPLETE in ROADMAP.md
- Next Recommendation: Cycle 3246 — explore next language gap. Options: (a) `[f32; N]` array support, (b) struct field array access, (c) nested arrays, (d) other type coverage gaps in M11-C. Recommend investigating what real-world BMB programs need next.
