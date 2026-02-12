# Cycle 363: String glob_match method

## Date
2026-02-13

## Scope
Add glob pattern matching method for strings. Original roadmap suggested regex-like methods, but BMB already has comprehensive string search methods (contains, starts_with, ends_with, index_of, find_all). Added `glob_match` — the one missing pattern-matching capability.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (types/mod.rs)
- `glob_match(pattern: String) -> bool` — simple glob matching with `*` and `?`

### Interpreter (interp/eval.rs)
- `glob_match_impl(text, pattern)` — O(n*m) worst case, O(n+m) typical
  - `*` matches any sequence of characters (including empty)
  - `?` matches exactly one character
  - Uses greedy backtracking algorithm (no regex engine needed)

### Tests
- 6 integration tests: type check, exact match, star pattern, question mark, no match, complex pattern

## Test Results
- Standard tests: 4181 / 4181 passed (+6)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Standard glob algorithm |
| Architecture | 10/10 | Follows existing string method pattern |
| Philosophy Alignment | 10/10 | No runtime overhead — simple algorithm |
| Test Quality | 10/10 | Multiple pattern types tested |
| Code Quality | 10/10 | Clean standalone implementation |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 364: Array window/slide methods
