# Cycle 466: Multi-String Concat Functions (concat3/concat5/concat7)

## Date
2025-02-12

## Scope
Implement `concat3`, `concat5`, `concat7` runtime functions that concatenate 3/5/7 strings in a single allocation, and convert hot-path bootstrap codegen functions to use them instead of chained `string_concat` calls.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 4/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Chained `string_concat(a, string_concat(b, c))` requires N-1 allocations for N strings
- Single-allocation concat reduces to 1 allocation regardless of string count
- Hot codegen functions (binop, cmp, not, neg, bnot, gep, load_ptr, store_ptr, string_ref) produce most IR lines

## Implementation

### Runtime (`bmb/runtime/bmb_runtime.c`)
- Added `bmb_string_concat3`, `bmb_string_concat5`, `bmb_string_concat7`
- Each computes total length, does single `bmb_alloc`, copies all segments with `memcpy`

### Rust Compiler
- `bmb/src/types/mod.rs`: Registered concat3/5/7 with String signatures
- `bmb/src/mir/lower.rs`: Added concat3/5/7 to String-returning runtime functions list (CRITICAL)
- `bmb/src/codegen/llvm.rs`: LLVM function declarations + return type tracking

### Bootstrap Compiler (`bootstrap/compiler.bmb`)
- Added extern declarations for `bmb_string_concat3/5/7`
- Added to `gen_runtime_decls_string`, `get_call_return_type`, `get_call_arg_types`, `map_runtime_fn`, `is_string_fn_group1`
- Converted 9 hot codegen functions to use concat3/5/7:
  - `llvm_gen_binop`, `llvm_gen_cmp`, `llvm_gen_not`, `llvm_gen_neg`, `llvm_gen_bnot`
  - `llvm_gen_gep`, `llvm_gen_load_ptr`, `llvm_gen_store_ptr`, `llvm_gen_string_ref`

### Key Bug Fix
- concat3/5/7 were missing from MIR lowering's String-returning function list
- Without this, results typed as MirType::I64, causing `concat_result + "|"` to generate PtrOffset (pointer arithmetic) instead of string concatenation
- Fix: Added to match arm in `bmb/src/mir/lower.rs` line 1207

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests | 13/13 PASS (Stage 1 AND Stage 2) |
| Stage 1 build | SUCCESS |
| Stage 2 build | SUCCESS |
| Fixed point | VERIFIED (S2 == S3, 68,847 lines) |

## Performance
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Stage 1 emit-ir | ~2.34s | ~2.51s | +7% (regression) |
| Rust compiler | ~0.50s | - | baseline |
| IR lines | 68,626 | 68,847 | +221 (new extern decls) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | All tests pass, fixed point verified |
| Architecture | 7/10 | Clean runtime + compiler integration, but conversion overhead negates benefit |
| Philosophy Alignment | 6/10 | Performance regression contradicts "Performance > Everything" |
| Test Quality | 8/10 | Full golden test + fixed-point coverage |
| Documentation | 7/10 | Code comments present, cycle log detailed |
| Code Quality | 7/10 | Clean implementation but inttoptr/ptrtoint overhead issue |
| **Average** | **7.3/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | concat3/5/7 show ~7% regression instead of improvement — inttoptr/ptrtoint conversion overhead for passing ptr args likely exceeds allocation savings | Investigate: the i64↔ptr conversion cost may outweigh fewer allocations |
| I-02 | M | 9 converted functions is small fraction of total string operations in bootstrap compiler | Consider: broader adoption or different approach entirely |
| I-03 | M | The fundamental bottleneck may be string_concat itself (called thousands of times) not allocation count | Profile: identify actual hottest functions |
| I-04 | L | IR line count increased by 221 lines due to new extern declarations | Acceptable: minimal impact |

## Next Cycle Recommendation
- The concat3/5/7 approach didn't deliver expected improvement
- Consider reverting the bootstrap conversions and focusing on different optimization vectors:
  - StringBuilder pattern for multi-part IR generation
  - Reducing total string operations rather than optimizing individual ones
  - Focus on the 4.7x gap root cause (likely not allocation but total operation count)
