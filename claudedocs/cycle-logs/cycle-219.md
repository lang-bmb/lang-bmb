# Cycle 219: MIR Optimization Pass Unit Tests

## Date
2026-02-11

## Scope
Add unit tests for under-tested MIR optimization passes: DeadCodeElimination, ConstFunctionEval, TailCallOptimization, TailRecursiveToLoop, LinearRecurrenceToLoop.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Analyzed `bmb/src/mir/optimize.rs` (15K LOC, 170 existing tests)
- Identified 5 under-tested optimization passes:
  - DeadCodeElimination: 449 LOC, 1 test → 5 tests
  - ConstFunctionEval: 132 LOC, 2 tests → 5 tests
  - TailCallOptimization: 335 LOC, 2 tests → 5 tests
  - TailRecursiveToLoop: 379 LOC, 2 tests (1 trivial name check) → 5 tests
  - LinearRecurrenceToLoop: 474 LOC, 2 tests → 6 tests
- Studied existing test patterns (make_test_function helper, MirFunction construction)
- Discovered DCE uses single-pass liveness (dead chains with dependencies aren't fully cleaned in one pass)

## Implementation

### MIR Optimize Tests (`bmb/src/mir/optimize.rs`)
Added 17 new tests across 5 optimization passes:

**DeadCodeElimination (+4 tests)**
- `test_dce_preserves_side_effects`: Call with unused result NOT removed (side effects)
- `test_dce_chain_dependency`: a→b→c chain all live when c is returned
- `test_dce_multiple_independent_dead`: Multiple independent dead constants removed
- `test_dce_branch_condition_kept`: Variable used in Branch terminator preserved

**ConstFunctionEval (+3 tests)**
- `test_const_function_eval_side_effects_not_inlined`: Function with print() call NOT inlined
- `test_const_function_eval_variable_return`: Const assigned to var then returned → inlined
- `test_const_function_eval_multi_block_not_inlined`: Multi-block function NOT inlined

**TailCallOptimization (+3 tests)**
- `test_tco_direct_tail_call`: Phase 1 direct tail call detection
- `test_tco_non_tail_call_not_marked`: Call with post-use = NOT tail call
- `test_tco_void_return_no_change`: Return(None) does not trigger TCO

**TailRecursiveToLoop (+3 tests)**
- `test_tail_recursive_gcd`: Both params change (more complex than sum)
- `test_tail_recursive_non_self_call_no_change`: Non-self-recursive call skipped
- `test_tail_recursive_all_invariant_no_change`: All-invariant params = infinite loop, skipped

**LinearRecurrenceToLoop (+4 tests)**
- `test_linear_recurrence_multi_param_no_change`: 2+ params rejected
- `test_linear_recurrence_non_integer_param_no_change`: F64 param rejected
- `test_linear_recurrence_fibonacci_loop_structure`: Verify loop blocks/phi/Add after transform
- `test_linear_recurrence_single_recursive_call_no_change`: Single call (factorial) rejected

## Test Results
- Standard tests: 2571 / 2571 passed (+17 from 2554)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Follows existing test patterns exactly |
| Philosophy Alignment | 10/10 | Testing ensures optimization correctness |
| Test Quality | 9/10 | Good coverage of positive and negative cases |
| Code Quality | 9/10 | Clean, well-documented test functions |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | DCE single-pass doesn't clean transitive dead chains | Consider iterative DCE or document limitation |
| I-02 | L | LinearRecurrenceToLoop only handles fibonacci pattern (threshold=1) | Design decision, not a defect |
| I-03 | L | ConstFunctionEval doesn't handle multi-block const functions | Could be enhanced in future |

## Next Cycle Recommendation
- Continue adding tests for other under-tested passes (StringConcatOptimization, LoopBoundedNarrowing)
- Or begin Rust-side integration test coverage expansion
