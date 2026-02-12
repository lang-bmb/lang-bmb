# Cycle 354: Argument count mismatch improvements (show signature)

## Date
2026-02-13

## Scope
Improve argument count mismatch errors to show function name and parameter types.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (types/mod.rs)
Updated 4 argument count mismatch error paths:
- **Closure/function variable calls**: `"closure expects N arguments, got M"` → `"'name' expects N arguments (type1, type2), got M"`
- **Non-generic function calls**: `"expected N arguments, got M"` → `"'func' expects N arguments (type1, type2), got M"`
- **Generic function calls**: Same improvement
- **Trait method calls**: `"method 'x' expects N arguments, got M"` → `"method 'x' expects N arguments (type1, type2), got M"`

Example: `add(1)` where `fn add(a: i64, b: i64)` → `'add' expects 2 arguments (i64, i64), got 1`

### Integration Tests
Added 4 tests:
- `test_arg_count_shows_function_name`: Verifies function name in error
- `test_arg_count_shows_param_types`: Verifies parameter types in error
- `test_arg_count_shows_got_count`: Verifies actual argument count in error
- `test_arg_count_zero_args_provided`: Verifies 0-arg case

## Test Results
- Standard tests: 4085 / 4085 passed (+4)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All error messages accurate |
| Architecture | 10/10 | Minimal changes to existing paths |
| Philosophy Alignment | 10/10 | Better DX for AI users |
| Test Quality | 9/10 | Covers main function call paths |
| Code Quality | 10/10 | Clean, consistent pattern |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Built-in method arg errors still use old format ("method() takes N args") | Could unify in future cycle |

## Next Cycle Recommendation
- Cycle 355: Chained method error context (show receiver type in chain)
