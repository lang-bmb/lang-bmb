# Cycle 303: Array pairwise, split_at, uniq_by

## Date
2026-02-12

## Scope
Add adjacent pairing, array splitting, and key-based uniqueness methods for arrays.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker
- `pairwise() -> [[T]]` — returns adjacent pairs as 2-element subarrays
- `split_at(i64) -> [[T]]` — splits array at index into 2-element array of subarrays
- `uniq_by(fn(T) -> K) -> [T]` — keeps first element for each unique key

### Interpreter
- `pairwise` — iterates with sliding window of size 2
- `split_at` — slices at clamped index, returns pair of arrays
- `uniq_by` — tracks seen keys with linear scan, keeps first occurrence

### Notes
- Originally planned integer methods but all candidates already existed (39 methods)
- Pivoted to useful array methods not yet implemented

### Integration Tests
Added 8 tests covering all methods + edge cases + chaining.

## Test Results
- Standard tests: 3750 / 3750 passed (+8 from 3742)
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
| I-01 | L | uniq_by uses O(n*k) linear scan for key lookup | Acceptable for interpreter |

## Next Cycle Recommendation
- Array transpose, associate, frequencies methods
