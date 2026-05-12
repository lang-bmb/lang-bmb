# ISSUE: sort_presorted Codegen Dominance Error

## Status: RESOLVED (Cycle 100)

## Date: 2026-02-09

## Description
The `surpass/sort_presorted` benchmark fails to compile with:
```
Instruction does not dominate all uses!
  %call1 = tail call i64 @sum_array(ptr %0, i64 0, i32 0)
  ret i64 %call1
```

This occurs during `opt --passes=default<O3>,scalarizer` optimization.

## Root Cause (Suspected)
The benchmark passes `[i64; 32]` arrays by value to `@pure` functions with preconditions. The LLVM IR generated for array-by-value parameters may have dominance issues when the opt pass performs aggressive inlining/tail-call optimization.

## Reproduction
```bash
bmb build ecosystem/benchmark-bmb/benches/surpass/sort_presorted/bmb/main.bmb -o /tmp/test
```

## Severity: Medium
Only affects programs with stack-allocated arrays passed by value to pure functions with preconditions.

## Files
- `ecosystem/benchmark-bmb/benches/surpass/sort_presorted/bmb/main.bmb`
- `bmb/src/codegen/llvm.rs` (IR generation)
