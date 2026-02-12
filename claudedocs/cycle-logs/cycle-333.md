# Cycle 333: Float round_to, floor_to, ceil_to

## Date
2026-02-12

## Scope
Add precision rounding methods for floats — round/floor/ceil to N decimal places.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `round_to(i64) -> f64` — round to N decimal places
- `floor_to(i64) -> f64` — floor to N decimal places
- `ceil_to(i64) -> f64` — ceil to N decimal places

### Interpreter
- All use `10f64.powi(places)` as factor, multiply, apply operation, divide back
- Negative places round to tens/hundreds (e.g., `1234.5.round_to(-2)` = 1200.0)

### Notes
- Original plan was String byte operations, but `bytes()`, `byte_at()`, `char_code_at()`, `from_char_code()` already exist — pivoted to Float precision rounding

### Integration Tests
Added 6 tests covering all methods + edge cases + chaining.

## Test Results
- Standard tests: 3972 / 3972 passed (+6 from 3966)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All precision rounding correct |
| Architecture | 10/10 | Follows Float method pattern |
| Philosophy Alignment | 10/10 | Useful numerical computing utility |
| Test Quality | 10/10 | Good coverage including negative places |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 334: Array statistical — variance, stddev, percentile
