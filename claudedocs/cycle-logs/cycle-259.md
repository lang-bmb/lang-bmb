# Cycle 259: Levenshtein Distance Deduplication

## Date
2026-02-12

## Scope
Extract triplicated Levenshtein distance implementation into shared `util` module. Previously identical code existed in `types/mod.rs`, `resolver/mod.rs`, and `query/mod.rs`.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 4/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Found 3 copies of Levenshtein distance across types, resolver, query modules
- Also found `find_similar_name` and `format_suggestion_hint` duplicated in types + resolver
- Resolver's `find_similar_name` had hardcoded threshold=2 (types had parameter)
- Unified to parametric version with threshold argument

## Implementation

### New Module (`bmb/src/util.rs`)
- `levenshtein_distance(a, b) -> usize` — shared implementation
- `find_similar_name(name, candidates, threshold) -> Option<&str>` — unified version
- `format_suggestion_hint(suggestion) -> String` — shared hint formatter
- 11 unit tests covering all edge cases

### Updated Modules
- `types/mod.rs`: Removed local implementation, `use crate::util::*`
- `resolver/mod.rs`: Removed local implementation, `use crate::util::*`, added threshold arg
- `query/mod.rs`: Removed local `levenshtein`, `use crate::util::levenshtein_distance as levenshtein`
- `lib.rs`: Added `pub mod util;`

## Test Results
- Standard tests: 3314 / 3314 passed (+11 from 3303)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All existing functionality preserved |
| Architecture | 10/10 | Clean shared utility module |
| Philosophy Alignment | 9/10 | Code quality, not performance |
| Test Quality | 10/10 | Comprehensive unit tests in util module |
| Code Quality | 10/10 | 3 copies → 1 source of truth |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | query module tests duplicate util tests | Acceptable, provides regression coverage |

## Next Cycle Recommendation
- Verification fallback soundness fix (silent Trust mode when solver unavailable)
- WASM bump allocator (replace memory allocation TODOs)
- Or: Additional compiler quality improvements
