# Cycle 95: compiler.bmb Float Binop Codegen Fix

## Date
2026-02-09

## Scope
Fix compiler.bmb to generate correct LLVM IR for float arithmetic operations (fadd/fsub/fmul/fdiv) and float return types.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 4/5 |
| Dependency Direction | 5/5 |

Proper fix: AST-level float detection propagates through MIR to LLVM IR. Matches the pattern already used by the separate pipeline (lowering.bmb + llvm_ir.bmb).

## Root Cause Analysis

### Problem
compiler.bmb treated all values as i64. Float operations like `3.14 * 2.0` were lowered to `mul nsw i64` instead of `fmul double`. Float function returns used `ret i64` instead of `ret double`.

### Root Cause
compiler.bmb is a self-contained bootstrap compiler with no type system. It type-erases everything to i64. The separate pipeline (lexer.bmb → llvm_ir.bmb) handles floats correctly with `f+`/`f-`/`f*`/`f/` MIR operators.

## Implementation

### AST-Level Float Detection
Added `is_float_expr(ast)` function that recursively checks:
- `(float ...)` nodes → true
- `(unary ...)` → check operand
- `(binop ...)` → check left operand
- Everything else → false

### MIR Lowering Changes
- **Step-based path**: When binop left operand is float, prefix operator with "f" → `f+`, `f-`, `f*`, `f/`, `f<=`, `f>=`, `f<`, `f>`, `f==`, `f!=`
- **SB-based path** (`lower_binop_sb`): Same detection → `let mir_op = if is_float_expr(left_ast) { "f" + op } else { op }`

### LLVM Codegen Changes
Added handlers for float MIR operators before integer handlers:
- `f+` → `llvm_gen_float_binop("fadd", ...)`
- `f-` → `llvm_gen_float_binop("fsub", ...)`
- `f*` → `llvm_gen_float_binop("fmul", ...)`
- `f/` → `llvm_gen_float_binop("fdiv", ...)`
- `f<=`/`f>=`/`f<`/`f>`/`f==`/`f!=` → `llvm_gen_float_cmp(pred, ...)`

New functions:
- `llvm_gen_float_binop(op, line, pos, dest)` → `%dest = <op> double %left, %right`
- `llvm_gen_float_cmp(pred, line, pos, dest)` → `%dest_cmp = fcmp <pred> double ...` + `%dest = zext i1 %dest_cmp to i64`

### Return Type Fix
Modified `llvm_gen_return_typed` to handle f64:
- `ret_type == "f64"` → `"  ret double " + val`

Modified `llvm_gen_fn_header` (already had f64 support for function signatures).

## Generated IR Before/After

### Before (incorrect)
```llvm
define double @mul_floats() nounwind {
entry:
  %_t0 = fadd double 0.0, 3.14
  %_t1 = fadd double 0.0, 2.0
  %_t2 = mul nsw i64 %_t0, %_t1   ; WRONG: i64 mul on double values
  ret i64 %_t2                      ; WRONG: i64 ret for double function
}
```

### After (correct)
```llvm
define double @mul_floats() nounwind {
entry:
  %_t0 = fadd double 0.0, 3.14
  %_t1 = fadd double 0.0, 2.0
  %_t2 = fmul double %_t0, %_t1    ; CORRECT: fmul double
  ret double %_t2                    ; CORRECT: ret double
}
```

## Test Results
- Tests: 1701 / 1701 passed (no regressions)
- Bootstrap: 3-Stage PASS with fixed point (5728ms)
- LLVM opt -O2: PASS on all float IR
- All four float operations verified: fadd, fsub, fmul, fdiv

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | Float binop + return verified, copy/alloca still uses i64 for floats |
| Architecture | 8/10 | AST-level detection is pragmatic; full type tracking would be better |
| Philosophy Alignment | 9/10 | Proper fix, not workaround. Matches existing patterns |
| Test Quality | 7/10 | E2E IR verification; could add more unit tests |
| Documentation | 9/10 | Clear before/after, issue updated |
| Code Quality | 8/10 | Consistent with existing patterns |
| **Average** | **8.3/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | Float copy still uses `add nsw i64 0, %src` instead of identity | Future: need type tracking per-temp |
| I-02 | M | Mutable float alloca uses `alloca i64` not `alloca double` | Future: need type tracking |
| I-03 | L | Float comparisons return i64 (zext from i1) not bool | OK: matches integer comparison pattern |
| I-04 | L | `fadd double 0.0, X` for float constants could be optimized | LLVM mem2reg handles this |

## Files Modified
- `bootstrap/compiler.bmb` — is_float_expr, float MIR ops, float LLVM codegen, float return type

## Next Cycle Recommendation
Continue with remaining plan items: Cycle 96 (bootstrap f64 float literals in lexer/parser) or Cycle 97 (T? nullable parser).
