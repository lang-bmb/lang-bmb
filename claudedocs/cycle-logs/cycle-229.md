# Cycle 229: Type System Edge Case Tests

## Date
2026-02-11

## Scope
Add integration tests for type system edge cases: generic inference, struct/enum type checking, nullable types, closure types, array types, contract types, and complex expression inference.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Found 18 existing type-related integration tests
- Discovered BMB uses `new StructName { ... }` syntax (not `StructName { ... }`)
- `Box` is a reserved type name — renamed test struct to `Wrapper`
- Generic body type mismatch (`fn bad<T>(x: T) -> T = 42`) is allowed in BMB
- Type error for generics triggers at call site when return type mismatches

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added 31 new tests in 10 categories:

**Generic Type Inference (5 tests)**
- `test_type_generic_identity_i64`: Generic id with i64
- `test_type_generic_identity_bool`: Generic id with bool
- `test_type_generic_identity_string`: Generic id with String
- `test_type_generic_two_params`: Two type parameters
- `test_type_generic_return_type_mismatch`: Wrong return type error

**Struct Type Checking (4 tests)**
- `test_type_struct_field_types_match`: Correct field types
- `test_type_struct_field_wrong_type`: Wrong field type error
- `test_type_struct_return_type_mismatch`: Struct returned as i64 error
- `test_type_nested_struct`: Nested struct construction

**Enum Type Checking (3 tests)**
- `test_type_enum_variant_data_type`: Variant with correct data
- `test_type_enum_variant_wrong_data_type`: Variant with wrong data error
- `test_type_enum_match_exhaustive`: Exhaustive match

**Nullable Type Checking (3 tests)**
- `test_type_nullable_return_value`: Nullable return
- `test_type_nullable_cannot_assign_null_to_non_nullable`: null → non-nullable error
- `test_type_nullable_unwrap_or_returns_base_type`: unwrap_or return type

**Function Type Checking (3 tests)**
- `test_type_function_wrong_return_type`: i64 fn returning bool error
- `test_type_recursive_function_return`: Recursive factorial type
- `test_type_mutual_recursion`: is_even/is_odd mutual recursion

**Expression Type Checking (4 tests)**
- `test_type_if_branches_same_type`: Consistent if/else types
- `test_type_if_branches_different_types_error`: Inconsistent branches
- `test_type_match_branches_consistent`: Match branch types
- `test_type_match_branches_inconsistent_error`: Mismatched match type

**Contract Type Checking (2 tests)**
- `test_type_contract_pre_uses_params`: Precondition syntax
- `test_type_contract_post_uses_ret`: Postcondition with ret

**Closure Type Checking (2 tests)**
- `test_type_closure_inferred_param_types`: Closure parameter inference
- `test_type_closure_captures_outer_var`: Closure captures

**Array Type Checking (2 tests)**
- `test_type_array_elements_same_type`: Array literal typing
- `test_type_array_index_returns_element_type`: Array index type

**Complex Type Expressions (3 tests)**
- `test_type_function_taking_struct_returning_field`: Struct → field
- `test_type_generic_with_struct`: Generic id with struct
- `test_type_complex_expression_inference`: Multi-op inference

## Test Results
- Standard tests: 2732 / 2732 passed (+31 from 2701)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Uses existing type_checks/type_error helpers |
| Philosophy Alignment | 10/10 | Type system correctness is fundamental |
| Test Quality | 9/10 | Both positive and negative type checking |
| Code Quality | 9/10 | Clear test names, discovered `new` syntax |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | BMB allows `fn bad<T>(x: T) -> T = 42;` — generic body not checked against T | May be by design for monomorphization |
| I-02 | L | `Box` is reserved — cannot use as struct name | Expected behavior |
| I-03 | L | Trait-level type checking untested | Future cycle |

## Next Cycle Recommendation
- Add formatter/linter integration tests
- Or add comprehensive error message content tests
