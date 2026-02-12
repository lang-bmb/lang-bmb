# Cycle 285: Array Higher-Order Methods II — zip_with, each_cons, step_by, chunk_by

## Date
2026-02-12

## Scope
Add advanced array transformation methods: zip_with (combine two arrays with a function), each_cons (consecutive windows alias), step_by (every nth element), chunk_by (group consecutive by key).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `zip_with(other: [U], fn(T, U) -> V) -> [V]` — combines two arrays element-wise with closure
- `each_cons(n: i64) -> [[T]]` — sliding windows (alias of windows)
- `step_by(n: i64) -> [T]` — every nth element
- `chunk_by(fn(T) -> K) -> [[T]]` — groups consecutive elements by key function

### Interpreter
- `zip_with` — zips to min length, applies closure to each pair
- `each_cons` — same as windows implementation
- `step_by` — uses Rust's `.step_by()` iterator adapter
- `chunk_by` — groups consecutive elements where key function returns same value

### Integration Tests
Added 10 tests covering all methods + chaining (step_by→zip_with).

## Test Results
- Standard tests: 3581 / 3581 passed (+10 from 3571)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods work correctly |
| Architecture | 10/10 | Follows established method pattern |
| Philosophy Alignment | 10/10 | Functional combinators |
| Test Quality | 9/10 | Good coverage with chaining |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | each_cons is redundant with windows — provides Ruby-style alias | Keep for API familiarity |
| I-02 | L | Array types in closure params still not parseable | Ongoing parser limitation |

## Next Cycle Recommendation
- String utility completeness or method quality polish
- Consider addressing parser limitation for array types in closures
