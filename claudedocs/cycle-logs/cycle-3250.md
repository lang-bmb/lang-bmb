# Cycle 3250: M11-C Phase 14 — Multi-Level Compound Const Fixed-Point Inlining
Date: 2026-05-28

## Re-plan
Plan valid. Inherited from Cycle 3249 carry-forward: fix `try_register_const_fn` to reject
compound const functions (those with `= copy`, `= add`, `= sub`, `= mul`, `= div` in body),
enabling the `optimize_const_inlining_loop` (added in Cycle 3249) to correctly propagate
multi-level const chains of arbitrary depth.

## Scope & Implementation

### Root Cause (confirmed from Cycle 3249 investigation)
For `const B = A + 1`, the lowered MIR body contains:
```
%_t0 = copy %A        ← unresolved const reference
%_t1 = const 1        ← integer literal sub-expression
%_t2 = add %_t0, %_t1 ← arithmetic
return %_t2
```
`try_register_const_fn` only checked `= call ` before registering. It found `%_t1 = const 1`
as the first `= const` occurrence → registered `@B = 1` (WRONG; correct value is 4).

### Fix Applied (bootstrap/compiler.bmb, `try_register_const_fn`)
Extended the guard to also reject functions with:
- `= copy ` — unresolved const reference (not yet inlined)
- `= add ` — arithmetic (not yet folded by CF/DCE)
- `= sub ` — arithmetic
- `= mul ` — arithmetic
- `= div ` — arithmetic

With this fix, on each `optimize_const_inlining` pass:
- Pass 1: Only leaf consts registered (e.g., `const A = 3` whose body is just `const 3`)
- Loop iteration 1: `copy %A` → `const 3` inlined, CF/DCE folds `add 3,1` → `const 4`
  → B registered as 4 ✓
- Loop iteration 2: `copy %B` → `const 4` inlined, CF/DCE folds `mul 4,2` → `const 8`
  → C registered as 8 ✓
- Converges in N+1 iterations for an N-level chain.

### Files Changed
- `bootstrap/compiler.bmb`: `try_register_const_fn` extended guard (lines ~20681-20699)
- `bootstrap/compiler.exe`: rebuilt as new S1 binary
- `tests/golden/test_golden_const_multilevel.bmb`: new golden test
- `tests/golden/test_golden_const_multilevel.bmb.out`: expected output

### Test Cases Verified
| Input | Expected | Actual |
|-------|----------|--------|
| `const A=5; const B=A+1; const C=B*3` | A=5, B=6, C=18 | ✅ |
| `const A=3; const B=A+1; const C=B*2; println(C)` | 8 | ✅ (was: 2) |
| `const ROWS=3; const COLS=4; const TOTAL=ROWS*COLS` | 12 | ✅ (unchanged) |
| `let arr:[i64;C]` (C=B*2=8) | arr[0]+arr[7]=30 | ✅ |

## Verification & Defect Resolution

- `cargo test --release`: 6259 tests, 0 failures ✅
- Stage 1 bootstrap (Rust bmb → compiler.exe): ✅
- Stage 2 bootstrap: S1 compiles compiler.bmb → S2 IR (139,840 lines) ✅
- 3-Stage Fixed Point: S2 IR == S3 IR ✅

No defects found.

## Reflection

**Scope fit**: Exactly targets the M11-C Phase 14 bug. The fix is minimal (6 new let bindings
+ updated guard condition) and the architectural design (fixed-point loop + simple-const-only
registration) is clean and correct.

**Latent defects**: None identified. The `optimize_const_inlining_loop` function added in
Cycle 3249 had the right shape but required this prerequisite fix to actually work.

**Structural improvement**: The guard could also reject `= rem `, `= f+`, `= f-`, `= f*`,
`= f/` for floating-point compound consts if those are ever needed. Not urgent — no current
use case.

**Philosophy drift**: None. This is a correct const propagation fix, not a workaround.

**Roadmap impact**: M11-C Phase 14 COMPLETE. The remaining M11-C work can continue.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - Add `= rem ` guard to `try_register_const_fn` if float/remainder compound consts needed
    (currently no use case, very low priority)
- Pending Human Decisions: None
- Roadmap Revisions: M11-C Phase 14 marked ✅ COMPLETE in ROADMAP.md
- Next Recommendation: Continue M11-C Phase 15 or next M11-C language gap item
