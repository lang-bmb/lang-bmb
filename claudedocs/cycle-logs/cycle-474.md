# Cycle 474: Leaf Expression Inlining + SB Experiment

## Date
2025-02-12

## Scope
Further optimize lowering phase: re-profile, test SB-based work items,
inline leaf expression handling to reduce trampoline iterations.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary

### Re-profiling (post-Cycle 473)
| Phase | Before | After C473 | % of Total |
|-------|--------|-----------|-----------|
| lower | 1,774ms | 1,192ms | 71% |
| gen_sb | 396ms | 422ms | 25% |
| other | 80ms | 75ms | 4% |
| TOTAL | 2,250ms | 1,689ms | 100% |

### SB-based make_work (FAILED EXPERIMENT)
Converted make_work3/4/6/7/10 to use StringBuilder instead of concatenation.
Result: **+8% regression** (1.81s → 1.95s). Cause: SB function call overhead
(sb_new + N×sb_push + sb_build) exceeds simple concatenation for small strings.
**Reverted.**

### Leaf Expression Inlining (SUCCESS)
`step_expr` dispatched leaf types (int, float, bool, string, var, unit) by creating
a NEW work item, causing 2 trampoline iterations per leaf. Inlining the handling
directly in step_expr eliminates one iteration per leaf expression.

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - `step_expr`: Inlined 6 leaf types (int, float, bool, string, var, unit)
     directly instead of creating work items for IN/FL/BO/ST/VA/UT handlers
   - Each inlined leaf: extract AST data → emit MIR → return make_step_leaf()
   - Eliminated: make_work3() call + make_step() call + full trampoline iteration
   - make_work/make_step SB experiment: tested and reverted (regression)

### Performance Results
| Metric | Cycle 472 | Cycle 473 | Cycle 474 | Total |
|--------|-----------|-----------|-----------|-------|
| Stage 1 emit-ir | 2.34s | 1.81s | **1.70s** | **-27%** |
| Gap vs Rust | 4.7x | 3.6x | **3.4x** | |
| Fixed point lines | 68,993 | 68,953 | 69,112 | |

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 17/17 PASS |
| Golden tests (Stage 2) | 17/17 PASS |
| Fixed point (S2==S3) | VERIFIED (69,112 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified |
| Architecture | 8/10 | Inlining adds code duplication with step_int etc. |
| Philosophy Alignment | 10/10 | Measurable performance improvement |
| Test Quality | 9/10 | Full verification |
| Documentation | 9/10 | Failed experiment documented |
| Code Quality | 8/10 | Some code duplication between inlined and standalone handlers |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | 3.4x gap remains (1.70s vs 0.50s) | Continue optimization |
| I-02 | L | Inlined leaf code duplicates step_int/float/etc handlers | Acceptable |
| I-03 | M | gen_program_sb (422ms / 25%) not yet optimized | Profile next |
| I-04 | H | SB-based make_work is SLOWER than concatenation | Do NOT use SB for small strings |
| I-05 | M | make_step still uses 7 string concatenations | Hard to improve further |

## Next Cycle Recommendation
- Cycle 475: Profile gen_program_sb (25% of time) and optimize
- Consider string hash map optimization (str_key_eq → memcmp)
- Look for algorithmic improvements in LLVM IR generation
