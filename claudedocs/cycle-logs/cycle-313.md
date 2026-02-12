# Cycle 313: Char conversion + extended classification

## Date
2026-02-12

## Scope
Add conversion and extended classification methods to Char type.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `is_alphanumeric() -> bool` — alphabetic or numeric
- `is_ascii() -> bool` — ASCII range check
- `is_ascii_digit() -> bool` — '0'-'9'
- `is_ascii_hexdigit() -> bool` — hex digit
- `is_control() -> bool` — control character
- `is_ascii_punctuation() -> bool` — ASCII punctuation
- `to_uppercase() -> char` — convert to uppercase
- `to_lowercase() -> char` — convert to lowercase

### Interpreter
- All classification methods delegate to Rust's `char` methods
- `to_uppercase`/`to_lowercase` use iterator `.next().unwrap_or(c)`

### Integration Tests
Added 8 tests covering all new methods.

## Test Results
- Standard tests: 3833 / 3833 passed (+8 from 3825)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods correct |
| Architecture | 10/10 | Extends Char dispatch cleanly |
| Philosophy Alignment | 10/10 | Complete char classification |
| Test Quality | 10/10 | Good coverage |
| Code Quality | 10/10 | Clean implementations |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 314: Char remaining utilities (is_digit with radix, eq_ignore_case)
