# Issue: compiler.bmb float binop/copy uses i64 instructions

## Date
2026-02-09

## Severity
Medium

## Status
RESOLVED (Cycle 95, v0.89.13) — Float binop codegen fix: AST-level float detection, f+/f-/f*/f/ MIR ops, fadd/fsub/fmul/fdiv + fcmp LLVM IR, ret double for f64 functions

## Description
compiler.bmb's lowering and codegen treat all values as i64. When float operations like `x * y` (where x, y are f64) are lowered, they produce `mul nsw i64` instead of `fmul double`. Similarly, `copy` of float values uses `add nsw i64` instead of `fadd double`.

## Root Cause
compiler.bmb has no type system — it is a self-contained bootstrap compiler that type-erases everything to i64. The separate pipeline (lexer.bmb + parser_ast.bmb + types.bmb + lowering.bmb + llvm_ir.bmb) handles float types correctly with `f+`/`f-`/`f*`/`f/` MIR operations and `fadd`/`fsub`/`fmul`/`fdiv` LLVM IR.

## Impact
- Float arithmetic in programs compiled by Stage 1/2 compilers will produce incorrect results
- Does NOT affect bootstrap itself (compiler.bmb doesn't use float values in its logic)
- The separate pipeline (lexer.bmb → llvm_ir.bmb) handles floats correctly

## Proposed Fix
Option A: Add minimal type tracking to compiler.bmb
- Track "float" flag through AST → MIR → LLVM pipeline
- When binop operands are float, emit `f+`/`f-`/`f*`/`f/` MIR and `fadd`/`fsub`/`fmul`/`fdiv` LLVM

Option B: Use naming convention
- Float temps use `%_ft` prefix instead of `%_t`
- Codegen detects `%_ft` and emits double operations

## Example
```
Input: let x: f64 = 3.14; let y: f64 = 2.0; let z: f64 = x * y;

Current (wrong):
  %_t0 = fadd double 0.0, 3.14  ; ← correct
  %_t1 = fadd double 0.0, 2.0   ; ← correct
  %_t2 = add nsw i64 0, %_t0    ; ← WRONG: should be fadd double
  %_t3 = add nsw i64 0, %_t1    ; ← WRONG: should be fadd double
  %_t4 = mul nsw i64 %_t2, %_t3 ; ← WRONG: should be fmul double

Expected:
  %_t0 = fadd double 0.0, 3.14
  %_t1 = fadd double 0.0, 2.0
  %_t2 = fadd double 0.0, %_t0
  %_t3 = fadd double 0.0, %_t1
  %_t4 = fmul double %_t2, %_t3
```

## Priority
LOW-MEDIUM: Doesn't block bootstrap; blocks correct float programs via bootstrap compiler only.
