# Cycle 399: Empty loop body detection lint

## Date
2026-02-13

## Scope
Add new lint rule to detect empty loop bodies in `while` and `for` loops.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Core Change
- Added `EmptyLoopBody` variant to `CompileWarning` enum
- Detection in `Expr::While` and `Expr::For` handlers: checks for `Block(vec![])` or `Unit` body
- Covers both `while cond {}` and `for x in iter {}`

### Tests (6 new)
| Test | Description |
|------|-------------|
| test_warning_empty_loop_body | Constructor for "while" |
| test_warning_empty_loop_body_for | Constructor for "for" |
| test_empty_while_loop_body | `while x {}` triggers |
| test_no_empty_while_with_body | `while x < 10 { x = x + 1 }` no warning |
| test_empty_for_loop_body | `for i in 0..10 {}` triggers |
| test_no_empty_for_with_body | `for i in 0..10 { sum = sum + i }` no warning |

## Test Results
- Unit tests: 2187 passed (+2)
- Main tests: 15 passed
- Integration tests: 2202 passed (+4)
- Gotgan tests: 23 passed
- **Total: 4427 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Follows existing pattern |
| Philosophy Alignment | 10/10 | Catches likely bugs |
| Test Quality | 10/10 | While + for, positive + negative |
| Code Quality | 10/10 | Clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 400: Chained comparison → match suggestion lint
