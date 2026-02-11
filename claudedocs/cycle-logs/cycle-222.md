# Cycle 222: Codegen llvm_text.rs Loop/Control Flow Tests

## Date
2026-02-11

## Scope
Add unit tests for loop, break, continue, return, and recursive call codegen in `codegen/llvm_text.rs` text IR emission.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Analyzed `bmb/src/codegen/llvm_text.rs` (6252 LOC, 89 existing tests)
- Discovered text codegen uses alloca+store+load for mutable variables (NOT phi nodes)
- The `source_to_ir()` helper runs tokenize→parse→MIR lower→TextCodeGen::generate without MIR optimization passes
- Tail call annotation (`is_tail`) requires MIR optimization passes that `source_to_ir()` doesn't run
- Phi instructions are only emitted for SSA merge values (e.g., if/else result merge), not for mutable loop variables
- Key IR patterns: `alloca i64` for vars, `store`/`load` for mutations, `br label %bb_loop_body` for back-edges

## Implementation

### Codegen Tests (`bmb/src/codegen/llvm_text.rs`)
Added 10 new tests for loop/control flow codegen:

**Loop Structure (2 tests)**
- `test_rt_loop_break_codegen`: loop {} with break produces back-edge and exit blocks
- `test_rt_loop_accumulator_codegen`: Iterative sum uses alloca for accumulators, add nsw, back-edge

**Continue (1 test)**
- `test_rt_continue_codegen`: Continue generates branch to loop header, modulo uses srem

**Return (2 tests)**
- `test_rt_return_expression_codegen`: Early return produces multiple ret instructions
- `test_rt_return_in_loop_codegen`: Return from loop uses mul nsw for i*i

**Nested Loops (1 test)**
- `test_rt_nested_loops_codegen`: Multiple alloca vars, at least 2 loop back-edges

**Recursive Call (1 test)**
- `test_rt_recursive_call_codegen`: Self-referential call instruction, base case comparison

**For Loop (1 test)**
- `test_rt_for_loop_with_break_codegen`: For loop with break uses alloca vars, mul for condition

**Void/Select (2 tests)**
- `test_rt_void_return_codegen`: Unit return type produces ret void
- `test_rt_select_pattern_codegen`: If-else produces select or br i1

## Test Results
- Standard tests: 2604 / 2604 passed (+10 from 2594)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Follows existing test patterns, assertions match actual IR output |
| Philosophy Alignment | 10/10 | Codegen correctness testing is essential |
| Test Quality | 8/10 | Good coverage but limited by source_to_ir() not running optimizations |
| Code Quality | 9/10 | Clear test names, accurate comments explaining alloca pattern |
| **Average** | **9.2/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | source_to_ir() doesn't run MIR optimization passes, limiting what can be tested | Consider adding source_to_optimized_ir() helper |
| I-02 | L | codegen/llvm.rs (inkwell backend, 6204 LOC, 42 tests) has even lower coverage | Future cycle |
| I-03 | L | Text codegen alloca pattern may produce suboptimal IR before LLVM opt | By design — LLVM opt handles mem2reg |

## Next Cycle Recommendation
- Add tests for other under-tested modules (main.rs CLI, types/infer.rs, types/unify.rs)
- Or add source_to_optimized_ir() helper for testing optimized codegen output
