# Cycle 254: Trait Impl Return Type & Parameter Validation

## Date
2026-02-12

## Scope
Fix type checker to validate that trait impl methods match the declared signatures in the trait definition. Previously `impl GetVal for S { fn get(self: Self) -> bool = self.v; }` was silently accepted when the trait declared `-> i64`.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- ImplBlock registration (types/mod.rs:915-939) stores methods without comparing against trait
- TraitInfo stores method signatures with `ret_type` and `param_types`
- Key insight: Both trait and impl may use `Self` — must `substitute_self()` on BOTH sides before comparing
- Type comparison via `type_to_string()` since Type enum doesn't derive PartialEq
- Existing test `test_type_trait_multiple_methods` uses `Self` return type — initially broke until Self substitution was applied to trait side too

## Implementation

### Type Checker (`bmb/src/types/mod.rs`)
Added comprehensive impl validation after method registration:
1. **Return type check**: Compares impl return type vs trait return type (both Self-substituted)
2. **Parameter count check**: Verifies same number of non-self parameters
3. **Parameter type check**: Verifies each parameter type matches (both Self-substituted)

Error messages include method name, trait name, target type, and the mismatched types.

### Integration Tests (`bmb/tests/integration.rs`)
- Updated `test_trait_impl_wrong_return_type_allowed` → `test_trait_impl_wrong_return_type_rejected` (now expects error)
- Added 8 new tests:
  - `test_trait_impl_correct_return_type_ok`: Matching return type passes
  - `test_trait_impl_wrong_return_type_error_message`: Error message verification
  - `test_trait_impl_wrong_param_count`: Parameter count mismatch
  - `test_trait_impl_wrong_param_type`: Parameter type mismatch
  - `test_trait_impl_multiple_methods_one_wrong`: One of multiple methods wrong
  - `test_trait_impl_unit_return_matches`: Unit return type matches
  - `test_trait_impl_f64_return_mismatch`: f64 vs i64 mismatch
  - `test_trait_impl_two_impls_both_correct`: Two correct impls of same trait

## Test Results
- Standard tests: 3279 / 3279 passed (+8 from 3271)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Catches return type, param count, param type mismatches |
| Architecture | 10/10 | Uses existing substitute_self/type_to_string infrastructure |
| Philosophy Alignment | 10/10 | Correctness fix, not workaround |
| Test Quality | 10/10 | Covers positive/negative, Self type, multiple methods |
| Code Quality | 10/10 | Clear error messages, handles Self substitution correctly |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Missing method (impl doesn't provide all trait methods) not checked | Future cycle |
| I-02 | L | Extra methods in impl (not in trait) not warned about | Future cycle |

## Next Cycle Recommendation
- Trait method dispatch in interpreter (runtime support)
- Missing method detection (impl must provide all trait methods)
