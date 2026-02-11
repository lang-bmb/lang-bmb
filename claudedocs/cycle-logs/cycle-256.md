# Cycle 256: Trait Completeness — Missing Method Detection

## Date
2026-02-12

## Scope
Add validation that impl blocks must provide all methods declared in the trait. Previously you could implement a trait with partial methods without any error.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- After Cycle 254 (return type validation), impl methods were validated individually
- But no check that ALL required methods are provided
- Empty `impl Trait for S { }` silently accepted
- Fix: After per-method validation, compute set difference between trait and impl methods

## Implementation

### Type Checker (`bmb/src/types/mod.rs`)
Added missing method check after per-method validation:
- Collects impl method names into HashSet
- Iterates trait methods and checks each is in impl
- Reports error with missing method name, trait name, and target type
- Uses `i.span` (impl block span) for error location

### Integration Tests (`bmb/tests/integration.rs`)
Added 6 new tests:
- `test_trait_missing_method_rejected`: Missing one of two methods
- `test_trait_missing_method_error_message`: Error names missing method
- `test_trait_all_methods_provided_ok`: Complete impl passes
- `test_trait_single_method_missing`: Empty impl for 1-method trait
- `test_trait_empty_impl_with_methods_error`: Empty impl error message
- `test_trait_complete_impl_at_runtime`: Full impl works end-to-end

## Test Results
- Standard tests: 3293 / 3293 passed (+6 from 3287)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Catches all missing method patterns |
| Architecture | 10/10 | Natural extension of Cycle 254 validation |
| Philosophy Alignment | 10/10 | Type safety improvement |
| Test Quality | 10/10 | Tests missing, empty, complete impls |
| Code Quality | 10/10 | Minimal, focused addition |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No warning for extra methods in impl (methods not in trait) | Low priority, could confuse |

## Next Cycle Recommendation
- Phase 1 complete (5 cycles of critical bug fixes)
- Begin Phase 2: Nullable T? MIR lowering
