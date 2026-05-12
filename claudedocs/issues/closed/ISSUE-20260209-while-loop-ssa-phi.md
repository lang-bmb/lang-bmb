# Issue: While loop mutable variable updates produce infinite loops in compiler.bmb

## Date
2026-02-09

## Severity
HIGH

## Status
RESOLVED (Cycle 94, v0.89.12) — Implemented Option A: alloca/load/store for mutable variables

## Description
When compiler.bmb compiles a while loop with mutable variable assignments like:
```bmb
let mut i: i64 = 0;
while i < 10 {
    { i = i + 1 }
};
```

The generated LLVM IR uses SSA registers without phi nodes for loop-carried variables. The loop condition always reads the initial value, causing an infinite loop.

## Root Cause
compiler.bmb's lowering is purely SSA-based. Mutable variables are lowered to SSA temps:
- `%_t14 = add nsw i64 0, 0`  (initial i = 0)
- Loop condition: `%_t15 = add nsw i64 0, %_t14`  (always reads %_t14 = 0)
- Body: `%_t24 = add nsw i64 %_t22, %_t23`  (computes i + 1 but result is unused)

Missing: PHI nodes at `loop_0` to select between initial value and updated value.

## Expected IR
```llvm
loop_0:
  %i = phi i64 [ 0, %entry ], [ %i_next, %body_0 ]
  %cmp = icmp slt i64 %i, 10
  br i1 %cmp, label %body_0, label %exit_0
body_0:
  %i_next = add nsw i64 %i, 1
  br label %loop_0
```

## Proposed Fix
Option A: Use alloca/load/store for mutable variables (simpler, LLVM's mem2reg handles it)
Option B: Track loop-carried variables and emit phi nodes (more complex, better IR)

## Impact
- Programs compiled by bootstrap compiler that use `while` with mutations will hang
- Does NOT affect bootstrap itself (compiler.bmb's while loops don't use assignments)
- The separate pipeline (lowering.bmb) handles this correctly with alloca+store
