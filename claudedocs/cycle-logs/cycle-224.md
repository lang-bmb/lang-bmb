# Cycle 224: Error Detection Coverage Tests

## Date
2026-02-11

## Scope
Add integration tests for error detection: type mismatch, argument count, undefined references, struct/enum errors, control flow type errors, and parse errors.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Analyzed existing error tests: only 5 type_error tests existed
- Added `parse_error()` helper for checking parse-stage failures
- Discovered extra struct fields are NOT a type error in BMB (flexible construction)
- All error tests use `type_error()` (compile fails) or `parse_error()` (parse fails) helpers

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added 21 new tests in 7 categories:

**Argument Count Mismatch (2 tests)**
- `test_error_too_few_args`: Missing argument detected
- `test_error_too_many_args`: Extra argument detected

**Type Mismatch (5 tests)**
- `test_error_bool_where_int_expected`: bool → i64
- `test_error_int_where_bool_expected`: i64 → bool
- `test_error_string_where_int_expected`: string → i64
- `test_error_arithmetic_on_bool`: true + false
- `test_error_comparison_mixed_types`: 42 == "hello"

**Undefined References (3 tests)**
- `test_error_undefined_function`: Call to nonexistent function
- `test_error_undefined_variable`: Use of undefined variable
- `test_error_undefined_struct`: Construction of undefined struct

**Struct Errors (3 tests)**
- `test_error_missing_struct_field`: Incomplete construction
- `test_error_field_access_on_non_struct`: .field on integer
- `test_error_wrong_field_type_in_struct`: Wrong type in field

**Enum Errors (1 test)**
- `test_error_unknown_enum_variant`: Nonexistent variant

**Return Type / Control Flow (4 tests)**
- `test_error_void_function_returning_value`: () fn returning i64
- `test_error_non_void_function_returning_unit`: i64 fn returning ()
- `test_error_if_condition_not_bool`: if 42 { ... }
- `test_error_while_condition_not_bool`: while 1 { ... }

**Parse Errors (3 tests)**
- `test_parse_error_unclosed_brace`: Missing }
- `test_parse_error_missing_return_type`: fn f() = ...
- `test_parse_error_missing_semicolon`: Statement without ;

## Test Results
- Standard tests: 2641 / 2641 passed (+21 from 2620)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All error tests correctly detect failures |
| Architecture | 9/10 | Follows existing test patterns, added parse_error helper |
| Philosophy Alignment | 10/10 | Error detection ensures compiler correctness |
| Test Quality | 9/10 | Comprehensive coverage of common error categories |
| Code Quality | 9/10 | Clear test names describing the error condition |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Extra struct fields silently accepted — potential correctness issue | Language design choice, may warrant lint warning |
| I-02 | L | Error message content not verified (only presence of error) | Could add type_error_contains helper for message quality testing |

## Next Cycle Recommendation
- Add tests for warning detection (unreachable code, unused variables)
- Or expand parser edge case coverage
