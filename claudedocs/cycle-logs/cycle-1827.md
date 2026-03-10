# Cycle 1827: Narrowing Fix for GEP Index Parameters
Date: 2026-03-10

## Inherited → Addressed
From 1826: Phase 2 — IR quality analysis of borderline benchmarks.

## Scope & Implementation

### Root Cause: i32 Narrowing Creates sext Overhead After Inlining
Subagent analysis of optimized IR revealed that `dijk_swap(dist, node, i32 a, i32 b)` parameters `a` and `b` were narrowed to i32 by `ConstantPropagationNarrowing`. After LLVM inlines these into `dijk_down`, the i32 values create **5 pairs of `shl 32 + ashr exact 32`** sign-extension operations in the hot swap path.

### Fix: Extended `param_directly_indexes_array_standalone`
The existing detection only checked `IndexLoad`/`IndexStore` patterns and tracked through `Copy` instructions. But params that flow through `BinOp` (Shl/Mul/Add) chains into `load_i64`/`store_i64`/`load_f64`/`store_f64` call arguments were not detected.

Extended the function to:
1. Track data flow through `BinOp` instructions (not just Copy)
2. Check if derived values reach `load_i64`/`store_i64`/`load_f64`/`store_f64`/`load_u8`/`store_u8` call arguments

### Result
- **Before**: `dijk_swap` has `i32 %a, i32 %b` → 5 `ashr exact 32` in optimized IR
- **After**: `dijk_swap` has `i64 %a, i64 %b` → 1 `ashr exact 32` (only `size` loop bound, legitimate)
- **dijk_down size param**: Still i32 (loop bound, not address index) — correct behavior

### Files Changed
- `bmb/src/mir/optimize.rs` — Extended `param_directly_indexes_array_standalone` with BinOp tracking and memory builtin detection

## Review & Resolution
- All 6,186 tests pass
- No narrowing regressions in bubble_sort, heap_sort, counting_sort
- 80% reduction in sign-extension overhead for dijkstra (5→1 ashr)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: `ConstantPropagationNarrowing` lacks `is_loop_invariant_bound` check (duplicated only in `LoopBoundedNarrowing`) — the `size` param sext is minor
- Next Recommendation: Investigate wave_equation noalias metadata loss through pointer rotation
