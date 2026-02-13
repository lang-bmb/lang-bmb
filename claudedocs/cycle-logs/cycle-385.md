# Cycle 385: Full pipeline regression tests

## Date
2026-02-13

## Scope
Add end-to-end regression tests exercising the full pipeline — functions, structs, enums, arrays, loops, pattern matching, and complex expressions.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Tests (10 new)
| Test | Description |
|------|-------------|
| test_pipeline_fibonacci | Recursive fibonacci(15) = 610 |
| test_pipeline_factorial | Recursive factorial(10) = 3628800 |
| test_pipeline_struct_with_enum | Struct with enum field + match |
| test_pipeline_array_sum_loop | While-loop array sum |
| test_pipeline_nested_function_calls | `add_one(double(add_one(double(5))))` |
| test_pipeline_multiple_structs | Nested structs — Rect with Point |
| test_pipeline_enum_chain | Sequential enum apply operations |
| test_pipeline_while_accumulate_pattern | While-loop with conditional accumulator |
| test_pipeline_complex_expression | Precedence: `2 + 3 * 4` vs `(2 + 3) * 4` |
| test_pipeline_for_in_with_conditional | For-in with if-else conditional sum |

### Fixes
- `if` without `else` not valid in BMB expression context — added else branches
- `return` in while body causes parse error — used accumulator pattern instead

## Test Results
- Standard tests: 4350 / 4350 passed (+20, combined with cycle 384)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Follows existing patterns |
| Philosophy Alignment | 10/10 | Comprehensive pipeline coverage |
| Test Quality | 10/10 | Exercises all major language features |
| Code Quality | 10/10 | Clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 386: Unnecessary parentheses detection lint
