# Cycle 289: Cross-type Conversions + Array reject, tap, count_by

## Date
2026-02-12

## Scope
Add cross-type conversion methods (to_bool for int and string) and functional array methods (reject, tap, count_by).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `i64.to_bool() -> bool` — non-zero = true
- `String.to_bool() -> bool` — "true" = true, else false
- `[T].reject(fn(T) -> bool) -> [T]` — inverse of filter
- `[T].tap(fn(T) -> ()) -> [T]` — side-effect, returns original array
- `[T].count_by(fn(T) -> K) -> i64` — count distinct keys

### Interpreter
- `to_bool` on int — `n != 0`
- `to_bool` on string — `s == "true"`
- `reject` — filter with inverted predicate
- `tap` — calls closure for each element, returns original array
- `count_by` — collects distinct keys, returns count

### Integration Tests
Added 10 tests covering all methods + filter/reject complement verification.

## Test Results
- Standard tests: 3622 / 3622 passed (+10 from 3612)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods work correctly |
| Architecture | 10/10 | Follows established patterns |
| Philosophy Alignment | 10/10 | Useful combinators and conversions |
| Test Quality | 10/10 | Good coverage including complement test |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | count_by uses linear scan for distinct values (O(n*k)) | Acceptable for interpreter |

## Next Cycle Recommendation
- Comprehensive integration tests for complex programs
- Edge case testing
