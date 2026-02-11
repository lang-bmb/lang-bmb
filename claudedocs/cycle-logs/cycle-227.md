# Cycle 227: Full Pipeline Verification Tests

## Date
2026-02-11

## Scope
Add integration tests that verify the full compilation pipeline: parse → type check → MIR lowering → MIR format, ensuring each stage produces correct output.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Analyzed existing pipeline tests: only 2 existed (`test_pipeline_parse_lower_format`, `test_pipeline_parse_lower_codegen`)
- `TextCodeGen` is private — cannot be used from integration tests
- Created `full_pipeline_mir()` helper using public `bmb::mir::format_mir()` and `bmb::mir::lower_program()`
- MIR text format: functions as `fn name(...)`, operators as `+`, `*`, terminators as `return`, `goto`, `branch`
- `MirProgram.struct_defs` is `HashMap<String, Vec<(String, MirType)>>`

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added `full_pipeline_mir()` helper and 10 new tests in 6 categories:

**Full Pipeline with MIR Inspection (4 tests)**
- `test_pipeline_fibonacci_full`: 4-stage verification (parse → type check → MIR → verify recursive call)
- `test_pipeline_struct_operations`: Struct in MIR struct_defs + multiplication op
- `test_pipeline_enum_match`: Enum match with return terminator
- `test_pipeline_loop_accumulator`: Loop creates multiple blocks with goto back-edge

**Pipeline + Interpreter Verification (4 tests)**
- `test_pipeline_closure`: Closure parse + interpret (42)
- `test_pipeline_generic_function`: Generic id<T> type check + interpret
- `test_pipeline_contract_precondition`: Contract pre condition + interpret
- `test_pipeline_match_with_early_return`: Complex match + early return interpret

**MIR Structure Verification (2 tests)**
- `test_pipeline_for_range`: For-range MIR + interpreter verification (0+1+2+3+4=10)
- `test_pipeline_multi_function`: Multi-function MIR with call graph

## Test Results
- Standard tests: 2686 / 2686 passed (+10 from 2676)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Uses public API (format_mir), avoids private TextCodeGen |
| Philosophy Alignment | 10/10 | Pipeline verification ensures compiler correctness |
| Test Quality | 9/10 | Tests verify multiple stages of compilation |
| Code Quality | 9/10 | Clean helper function, MIR text assertions |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | TextCodeGen is private — cannot test LLVM IR generation from integration tests | May need pub(crate) or separate codegen tests |
| I-02 | L | MIR format_mir output not documented — test assertions depend on format details | format_mir is stable enough for testing |

## Next Cycle Recommendation
- Begin Phase D quality assessment (Cycles 228-231)
- Or add shadow_binding and unused_mut warning tests
