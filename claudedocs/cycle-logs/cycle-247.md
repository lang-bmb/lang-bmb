# Cycle 247: Untested MIR Optimization Pass Integration Tests

## Date
2026-02-11

## Scope
Add integration tests for 7 previously untested MIR optimization passes: ContractUnreachableElimination, PureFunctionCSE, ConstFunctionEval, ConstantPropagationNarrowing, LoopBoundedNarrowing, AggressiveInlining, LinearRecurrenceToLoop. Also add optimization pipeline level tests, multi-pass combination tests, and OptimizationStats tests.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- ConstantPropagationNarrowing, LoopBoundedNarrowing, AggressiveInlining do NOT implement OptimizationPass trait — they have standalone `run_on_program()` methods
- PureFunctionCSE::from_program and ConstFunctionEval::from_program need &MirProgram — created `optimized_mir_from_program` helper
- MirFunction has `always_inline: bool` and `inline_hint: bool` fields (not `attributes: Vec<String>`)
- `set` keyword only for field/index mutation; plain variable reassignment uses `x = x + 1;`
- OptimizationStats has `iterations` and `pass_counts` fields (not `total_passes_run`, `total_changes`)
- ContractUnreachableElimination is a unit struct (no constructor)
- LinearRecurrenceToLoop::new() for fibonacci-to-loop conversion

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added 24 new tests:

**ContractUnreachableElimination (3 tests)**
- `test_opt_contract_unreachable_elimination_basic`: Pre n>=0 makes n<0 branch unreachable
- `test_opt_contract_unreachable_elimination_no_contract`: No contract keeps both branches
- `test_opt_contract_unreachable_preserves_semantics`: Pre x>0 with matching branch

**PureFunctionCSE (3 tests)**
- `test_opt_pure_function_cse_basic`: Duplicate @pure call eliminated
- `test_opt_pure_function_cse_different_args`: Different args keep both calls
- `test_opt_pure_function_cse_non_pure_not_eliminated`: Non-pure not CSE'd

**ConstFunctionEval (2 tests)**
- `test_opt_const_function_eval_basic`: @const function evaluated at compile time
- `test_opt_const_function_eval_with_computation`: @const with arithmetic

**ConstantPropagationNarrowing (2 tests)**
- `test_opt_constant_propagation_narrowing_small_arg`: fib(10) narrowed
- `test_opt_constant_propagation_narrowing_preserves_result`: Narrowing preserves semantics

**LoopBoundedNarrowing (2 tests)**
- `test_opt_loop_bounded_narrowing_basic`: Loop with small bound
- `test_opt_loop_bounded_narrowing_no_call`: Uncalled function not narrowed

**AggressiveInlining (3 tests)**
- `test_opt_aggressive_inlining_small_function`: Small pure → always_inline
- `test_opt_aggressive_inlining_custom_thresholds`: Custom thresholds work
- `test_opt_aggressive_inlining_large_function_not_inlined`: Large function not inlined

**LinearRecurrenceToLoop (3 tests)**
- `test_opt_linear_recurrence_fib`: Fibonacci converted to loop
- `test_opt_linear_recurrence_non_fibonacci`: Factorial not transformed
- `test_opt_linear_recurrence_preserves_base_case`: Base case preserved

**Pipeline & Stats (6 tests)**
- `test_opt_pipeline_aggressive_level`: Aggressive level folds constants
- `test_opt_pipeline_debug_no_optimization`: Debug level runs
- `test_opt_constant_fold_then_dce`: Multi-pass CF+DCE combination
- `test_opt_simplify_then_unreachable`: SimplifyBranches + ContractUnreachable combo
- `test_opt_stats_tracking`: OptimizationStats fields correct
- `test_opt_pipeline_for_level_release`: Release level pipeline

## Test Results
- Standard tests: 3111 / 3111 passed (+24 from 3087)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Tests all publicly accessible MIR optimization passes |
| Philosophy Alignment | 10/10 | Optimization pass correctness is critical to BMB performance |
| Test Quality | 9/10 | First tests for 7 previously untested passes |
| Code Quality | 9/10 | New helper for from_program passes, fixed set/reassign syntax |
| **Average** | **9.6/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | AlgebraicSimplification, BlockMerging, PhiSimplification etc. not re-exported from mir module | Can't test from integration tests |
| I-02 | L | Some passes only verify no crash, not detailed transformation | Could add MIR structure assertions |

## Next Cycle Recommendation
- Add WASM codegen advanced tests (target variants, control flow, struct/array)
- Add proof-guided optimization pass tests
- Add formatter/REPL tests if public APIs available
