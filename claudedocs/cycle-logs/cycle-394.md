# Cycle 394: LLVM codegen unit tests — builtins + gen_function_body

## Date
2026-02-13

## Scope
Add unit tests for LLVM codegen covering builtin declarations and function body generation with branches, locals, multiple return types.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Tests (11 new)
| Test | Description |
|------|-------------|
| test_generate_ir_declares_println | Verifies bmb_println_i64 builtin declared |
| test_generate_ir_declares_assert | Verifies bmb_assert builtin declared |
| test_generate_ir_declares_math_builtins | Verifies bmb_abs/min/max builtins |
| test_generate_ir_declares_sqrt_intrinsic | Verifies llvm.sqrt.f64 intrinsic |
| test_generate_ir_multi_block_branch | Multi-block function with conditional branch |
| test_generate_ir_with_locals | Function with local variables |
| test_generate_ir_bool_return | Function returning bool (icmp eq) |
| test_generate_ir_f64_arithmetic | Float addition (fadd) |
| test_generate_ir_subtraction | Integer subtraction (sub) |
| test_generate_ir_multiple_functions | Program with 2 functions |
| test_generate_ir_void_return | Unit-returning function (ret void) |

## Test Results
- Unit tests: 2181 passed (LLVM codegen: 53 with --features llvm)
- Main tests: 15 passed
- Integration tests: 2179 passed
- Gotgan tests: 23 passed
- **Total: 4398 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Uses existing test helpers |
| Philosophy Alignment | 10/10 | Verifies codegen correctness |
| Test Quality | 10/10 | Covers builtins, branches, types |
| Code Quality | 10/10 | Clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 395: LLVM codegen tests — gen_instruction + terminator coverage
