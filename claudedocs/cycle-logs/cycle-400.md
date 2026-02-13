# Cycle 400: Chained comparison detection lint

## Date
2026-02-13

## Scope
Add new lint rule to detect chains of 3+ equality comparisons on the same variable, suggesting `match` instead.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Core Change
- Added `ChainedComparison` variant to `CompileWarning` enum with name, count, span
- Detection in If expression handler: follows if-else-if chains counting `x == literal` with same LHS variable
- Fires when 3+ chained equality comparisons found on same variable
- Handles both direct and Block-wrapped else branches

### Tests (5 new)
| Test | Description |
|------|-------------|
| test_warning_chained_comparison | Constructor test |
| test_chained_comparison_3_arms | 3 chained `x == N` → warning |
| test_no_chained_comparison_2_arms | Only 2 arms → no warning |
| test_no_chained_comparison_different_vars | Different variables → no warning |
| test_no_chained_comparison_not_equality | Uses `<` not `==` → no warning |

## Test Results
- Unit tests: 2188 passed (+1)
- Main tests: 15 passed
- Integration tests: 2206 passed (+4)
- Gotgan tests: 23 passed
- **Total: 4432 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Follows existing lint pattern |
| Philosophy Alignment | 10/10 | Promotes idiomatic match usage |
| Test Quality | 10/10 | Positive + 3 negative cases |
| Code Quality | 10/10 | Clean, uses let-chains |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 401: Lint rule integration tests for all new rules
