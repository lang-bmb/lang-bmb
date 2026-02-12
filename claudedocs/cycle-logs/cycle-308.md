# Cycle 308: Bool toggle, then_val, then_fn, choose

## Date
2026-02-12

## Scope
Add boolean utility methods.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `toggle() -> bool` — returns negation
- `then_val(T) -> T?` — returns Some(val) if true, None if false
- `then_fn(fn() -> T) -> T?` — lazy then_val
- `choose(T, T) -> T` — returns first arg if true, second if false

### Interpreter
- `toggle` — returns `!b`
- `then_val` — wraps in Option::Some if true
- `then_fn` — calls closure and wraps if true
- `choose` — selects between two values

### Notes
- Originally named `select` but this is a BMB keyword; renamed to `choose`

### Integration Tests
Added 8 tests covering all methods + chaining.

## Test Results
- Standard tests: 3791 / 3791 passed (+8 from 3783)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods correct |
| Architecture | 10/10 | Follows method pattern |
| Philosophy Alignment | 10/10 | Useful bool utilities |
| Test Quality | 10/10 | Good coverage |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Integer bit manipulation: leading_zeros, trailing_zeros, rotate_left, rotate_right
