# Cycle 314: Char utilities — is_digit, eq_ignore_case, repeat

## Date
2026-02-12

## Scope
Add remaining practical Char methods: radix-aware digit check, case-insensitive comparison, and character repetition.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `is_digit(i64) -> bool` — check if char is digit in given radix
- `eq_ignore_case(char) -> bool` — case-insensitive equality
- `repeat(i64) -> String` — repeat char N times

### Interpreter
- `is_digit` — delegates to `char::is_digit(radix)`
- `eq_ignore_case` — compares via `to_lowercase()` iterators
- `repeat` — `std::iter::repeat(c).take(n).collect()`

### Integration Tests
Added 4 tests covering all methods + combined chaining.

## Test Results
- Standard tests: 3837 / 3837 passed (+4 from 3833)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods correct |
| Architecture | 10/10 | Follows established pattern |
| Philosophy Alignment | 10/10 | Completes Char type |
| Test Quality | 10/10 | Good coverage |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 315: Result type expansion — map, map_err, and_then
