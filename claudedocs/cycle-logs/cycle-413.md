# Cycle 413: LLVM text codegen — control flow + instruction variants

## Date
2026-02-13

## Scope
Add unit tests for untested LLVM text codegen instruction variants: ArrayInit, IndexLoad, Cast, Copy, PtrLoad/Store/Offset, ArrayAlloc, Branch, Switch, Goto terminators, and source-level round-trip tests.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Tests (17 new)
| Test | Description |
|------|-------------|
| test_mir_array_init_codegen | ArrayInit: alloca + GEP + store sequence |
| test_mir_index_load_codegen | IndexLoad: GEP + load for array access |
| test_mir_cast_i64_to_f64_codegen | Cast: sitofp i64 to double |
| test_mir_cast_f64_to_i64_codegen | Cast: fptosi double to i64 |
| test_mir_cast_i32_to_i64_codegen | Cast: sext i32 to i64 |
| test_mir_copy_instruction_codegen | Copy: place-to-place value transfer |
| test_mir_array_alloc_codegen | ArrayAlloc: stack array alloca [N x T] |
| test_mir_ptr_load_codegen | PtrLoad: load through pointer |
| test_mir_ptr_store_codegen | PtrStore: store through pointer |
| test_mir_ptr_offset_codegen | PtrOffset: GEP inbounds for pointer arithmetic |
| test_mir_const_string_global | String constant: @.str global emission |
| test_mir_multi_block_branch | Branch terminator: br i1 with labels |
| test_mir_goto_terminator | Goto terminator: unconditional br (bb_ prefix) |
| test_mir_switch_terminator | Switch terminator: switch i64 with cases + default |
| test_rt_string_concatenation_codegen | String concat via runtime call |
| test_rt_tuple_creation_and_access | Tuple: insertvalue emission |
| test_rt_deeply_nested_expressions | Complex arithmetic with multiple operations |

### Key Findings
- LLVM text codegen prefixes all block labels with `bb_` (e.g., `bb_exit`, `bb_case0`)
- MirInst::Copy has `src: Place` (not `Operand`)
- Terminator::Branch uses `then_label`/`else_label` fields
- Terminator::Switch uses `discriminant` field (not `cond`)

## Test Results
- Unit tests: 2295 passed (+17)
- Main tests: 26 passed
- Integration tests: 2257 passed
- Gotgan tests: 23 passed
- **Total: 4601 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Direct MIR + source-level tests |
| Philosophy Alignment | 10/10 | Codegen critical path |
| Test Quality | 10/10 | Covers instructions + terminators + round-trips |
| Code Quality | 10/10 | Clean, follows existing patterns |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 414: WASM text codegen edge cases
