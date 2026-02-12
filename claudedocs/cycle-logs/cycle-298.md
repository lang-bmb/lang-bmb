# Cycle 298: Array group_by, intersperse, compact

## Date
2026-02-12

## Scope
Add grouping, element interleaving, and null-removal methods for arrays.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `group_by(fn(T) -> K) -> [[T]]` — groups elements by key function
- `intersperse(T) -> [T]` — inserts separator between elements
- `compact() -> [T]` — removes zero/null elements

### Interpreter
- `group_by` — maintains insertion order of groups using linear scan
- `intersperse` — builds new array with separators between elements
- `compact` — filters out Value::Int(0) (null)

### Integration Tests
Added 8 tests covering all methods + edge cases + chaining.

## Test Results
- Standard tests: 3710 / 3710 passed (+8 from 3702)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods correct |
| Architecture | 10/10 | Follows array method pattern |
| Philosophy Alignment | 10/10 | Useful data manipulation |
| Test Quality | 10/10 | Good coverage |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | group_by uses linear scan for key lookup O(n*k) | Acceptable for interpreter |

## Next Cycle Recommendation
- Array repeat_val, cycle_n, generate methods
