# Cycle 284: String Closure Methods — map_chars, filter_chars, any_char, all_chars

## Date
2026-02-12

## Scope
Add higher-order string methods that operate on individual characters via closures: map_chars, filter_chars, any_char, all_chars.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `map_chars(fn(String) -> String) -> String` — transforms each character
- `filter_chars(fn(String) -> bool) -> String` — keeps characters matching predicate
- `any_char(fn(String) -> bool) -> bool` — true if any character matches
- `all_chars(fn(String) -> bool) -> bool` — true if all characters match

### Interpreter
- `map_chars` — iterates chars, applies closure to each, concatenates results
- `filter_chars` — iterates chars, keeps those where closure returns true
- `any_char` — short-circuits on first true
- `all_chars` — short-circuits on first false

### Integration Tests
Added 8 tests covering all methods + chaining (map_chars→filter_chars).

### Discovery
- String `>=` comparison not supported in BMB (comparison operators require numeric types). Fixed test to use equality comparison instead.

## Test Results
- Standard tests: 3571 / 3571 passed (+8 from 3563)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods work correctly |
| Architecture | 10/10 | Follows string method + closure pattern |
| Philosophy Alignment | 10/10 | Functional string operations |
| Test Quality | 9/10 | Good coverage with chaining test |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | String comparison operators (>=, <=, >, <) not supported | Language enhancement needed |
| I-02 | L | Characters passed as String (single char), not a dedicated char type | By design — BMB uses String for chars |

## Next Cycle Recommendation
- More string/array methods or quality improvements
- Consider string comparison operators
