# Cycle 300: String find_all, replace_first, split_once

## Date
2026-02-12

## Scope
Add string search, first-replacement, and one-shot splitting methods.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `find_all(string) -> [i64]` — returns all indices of substring occurrences
- `replace_first(string, string) -> string` — replaces only first occurrence
- `split_once(string) -> [string]` — splits at first delimiter, returns 1 or 2 elements

### Interpreter
- `find_all` — iterative search using Rust `find()` with advancing start position
- `replace_first` — delegates to Rust `replacen(&from, &to, 1)`
- `split_once` — delegates to Rust `split_once()`, returns 2-element array or 1-element if no match

### Integration Tests
Added 8 tests covering all methods + edge cases + chaining.

## Test Results
- Standard tests: 3726 / 3726 passed (+8 from 3718)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods correct |
| Architecture | 10/10 | Follows string method pattern |
| Philosophy Alignment | 10/10 | Useful string utilities |
| Test Quality | 10/10 | Good coverage |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- String char utilities — char_at, is_digit, is_alpha, is_alphanumeric, is_whitespace
