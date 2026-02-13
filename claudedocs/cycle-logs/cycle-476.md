# Cycle 476: Codegen Fast-Paths — map_runtime_fn + Merged Registry Lookup

## Date
2025-02-12

## Scope
Optimize LLVM IR generation phase with fast-path dispatch for runtime function
mapping and merged registry lookups.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary

### Profile Analysis (post-Cycle 475)
| Phase | Cycle 475 | Cycle 476 | Change |
|-------|-----------|-----------|--------|
| lower | 1,082ms | 1,101ms | ~0% (noise) |
| gen_sb | 371ms | **294ms** | **-20.8%** |
| total (core) | 1,530ms | **1,467ms** | **-4.1%** |

### Optimization 1: map_runtime_fn Fast-Path (SUCCESS)
**Problem**: `map_runtime_fn` has 100+ if/else string comparisons. For user-defined
functions (the majority of calls), ALL comparisons fail before returning the original name.
Called for every function call instruction in LLVM IR generation.

**Solution**: Byte-level first-character check. Runtime functions have second chars in
{a,b,c,f,g,m,o,p,r,s,t,w}. User functions starting with d,e,h-l,n,q,u-v,x-z return
immediately without any string comparisons.

### Optimization 2: Merged Registry Lookup (SUCCESS)
**Problem**: Each function call in codegen called `lookup_fn_ret(registry, fn)` and
`lookup_fn_params(registry, fn)` separately — two full O(N) linear scans of ~853 entries.

**Solution**: `lookup_fn_both` performs a single scan, returning "ret_type:param_sig".
Caller splits the result on `:`. Halves the number of registry scans at the main call site.

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - `map_runtime_fn`: New byte-level fast-path wrapper — checks `fn_name.byte_at(1)`
     against runtime function character set, skips to `map_runtime_fn_full` only when needed
   - `map_runtime_fn_full`: Extracted from original `map_runtime_fn` (all 100+ comparisons)
   - `lookup_fn_both` + `lookup_fn_both_at`: New merged lookup function returning
     "ret_type:param_sig" in single scan
   - `llvm_gen_call_reg` (line ~4930): Updated to use `lookup_fn_both` instead of
     separate `lookup_fn_ret` + `lookup_fn_params`

### Performance Results (Cumulative: Cycles 472-476)
| Metric | Cycle 472 | Cycle 474 | Cycle 476 | Total Change |
|--------|-----------|-----------|-----------|--------------|
| Stage 1 emit-ir (wall) | 2.34s | 1.70s | **1.61s** | **-31%** |
| Core compilation | 2,250ms | 1,689ms | **1,467ms** | **-35%** |
| lower phase | 1,774ms | 1,192ms | **1,101ms** | **-38%** |
| gen_sb phase | 396ms | 422ms | **294ms** | **-26%** |
| Gap vs Rust | 4.7x | 3.4x | **3.2x** | |
| Fixed point lines | 68,993 | 69,112 | 69,718 | +725 |

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 17/17 PASS |
| Golden tests (Stage 2) | 17/17 PASS |
| Fixed point (S2==S3) | VERIFIED (69,718 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified |
| Architecture | 9/10 | Clean separation: fast-path + full-path |
| Philosophy Alignment | 10/10 | Significant measurable performance improvement |
| Test Quality | 9/10 | Full verification pipeline |
| Documentation | 9/10 | Profiling data, cumulative tracking |
| Code Quality | 9/10 | Clean fast-path pattern, well-structured |
| **Average** | **9.3/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | 3.2x gap remains (1.61s vs 0.50s) | Continue optimization |
| I-02 | H | Registry lookups still O(N) linear scan; hash map would be O(1) | Future cycle |
| I-03 | M | map_runtime_fn full path still has 100+ comparisons for matching prefixes | Group by first char |
| I-04 | M | lower phase (1.1s, 75%) dominates — further trampoline optimization needed | Profile step handlers |
| I-05 | L | IR lines growing (+725 from optimization functions) | Natural consequence |
| I-06 | L | do_step handlers called in lowering could benefit from compound inlining | Future cycle |

## Next Cycle Recommendation
- Phase A complete (5 optimization cycles, 472-476)
- Cumulative: 2.34s → 1.61s (-31%), gap 4.7x → 3.2x
- Transition to Phase B: Bootstrap Feature Expansion (Cycles 477-483)
- Cycle 477: Float method support (floor, ceil, round, sqrt, to_int)
- OR continue optimization: hash map registry + compound expression inlining
