# Cycle 319: Array statistical methods — min_index, max_index, average, median

## Date
2026-02-12

## Scope
Add statistical utility methods for numeric arrays.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `min_index() -> i64` — index of minimum element
- `max_index() -> i64` — index of maximum element
- `average() -> f64` — arithmetic mean
- `median() -> f64` — statistical median

### Interpreter
- `min_index`/`max_index` — linear scan comparing numeric values
- `average` — sum / count
- `median` — sort then pick middle value(s)

### Integration Tests
Added 8 tests covering all methods + edge cases.

## Test Results
- Standard tests: 3877 / 3877 passed (+8 from 3869)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods correct |
| Architecture | 10/10 | Follows array pattern |
| Philosophy Alignment | 10/10 | Useful statistical utilities |
| Test Quality | 10/10 | Good coverage |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |
