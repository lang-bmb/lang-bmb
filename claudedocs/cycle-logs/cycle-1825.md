# Cycle 1825: Float Algebraic Identities + Noalias Investigation
Date: 2026-03-10

## Inherited → Addressed
From 1824: Continue Phase 1 MIR optimizations. Investigate noalias metadata gap.

## Scope & Implementation

### Float Algebraic Identity Patterns
Added complete float identity patterns to `simplify_binop`:

| Pattern | Operation | Result |
|---------|-----------|--------|
| `x + 0.0`, `0.0 + x` | FAdd | `x` |
| `x * 1.0`, `1.0 * x` | FMul | `x` |
| `x * 0.0`, `0.0 * x` | FMul | `0.0` |
| `x / 1.0` | FDiv | `x` |

### Removed Duplicate Match Arms
Eliminated pre-existing duplicate `FAdd` and `FMul` match arms that caused `unreachable_patterns` warnings (lines 862-901 in old code).

### Noalias Metadata Investigation — NOT A BUG
The agent analysis reported missing noalias metadata in `dijk_up` and `dijk_down`. Investigation revealed this is **correct behavior**: these functions only use `dist` as a single GEP base parameter. Noalias scoping requires 2+ distinct base params. The `dijkstra` main function and `dijk_swap` correctly get noalias metadata because they access multiple array bases.

### IR Quality Analysis Results (from subagent)
Key findings from borderline benchmark IR analysis:

1. **Dead address arithmetic alongside GEP**: IR bloat only — LLVM DCE eliminates it. No runtime impact.
2. **Store-load-store chains**: All temps get stored then immediately loaded. LLVM mem2reg handles it. Compile-time cost, not runtime.
3. **Constants narrowed to i32 + sext**: i32 narrowing creates unnecessary sext. LLVM constant-folds it.
4. **Duplicate `k-1` computation**: Insertion sort computes `sub nsw i64 %k, 1` four times. LLVM GVN merges them.
5. **Mandelbrot `cont` flag**: `while cont == 1` pattern adds one extra icmp+branch per iteration.

None of these affect runtime performance after `opt -O3` except possibly the mandelbrot cont-flag pattern.

### Files Changed
- `bmb/src/mir/optimize.rs` — Added FAdd/FMul/FDiv identity patterns, removed duplicate match arms

## Review & Resolution
- All 6,186 tests pass
- No warnings in build
- No regressions

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Mandelbrot cont-flag loop pattern could be optimized but LLVM handles it
- Next Recommendation: Look for MIR-level optimizations that LLVM can't do (cross-block patterns, loop structure)
