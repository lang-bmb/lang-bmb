# Cycle 54: Add MIR lowering unit tests + source-based test infrastructure

## Date
2026-02-08

## Scope
Add source-based MIR lowering tests to improve code coverage. The `parse_and_lower()` helper enables testing the full pipeline from BMB source → parse → MIR, which is more maintainable than hand-building ASTs.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Changes

### bmb/src/mir/lower.rs (+13 tests)

**New infrastructure:**
- `parse_and_lower(source: &str) -> MirProgram` — parses BMB source and lowers to MIR

**New tests:**
| Test | What it covers |
|------|---------------|
| `test_lower_while_loop_from_source` | While loop → multi-block MIR |
| `test_lower_let_binding_from_source` | Let bindings → instructions |
| `test_lower_if_else_from_source` | If-else → Branch terminator |
| `test_lower_recursive_call_from_source` | Recursive call detection |
| `test_lower_mutable_variable_from_source` | Mutable vars → Copy instructions |
| `test_lower_assign_in_if_branch_from_source` | Cycle 52 grammar fix → MIR |
| `test_lower_let_in_if_branch_from_source` | Let in if-branch → MIR blocks |
| `test_lower_for_loop_from_source` | For loop → multi-block MIR |
| `test_lower_nested_blocks_from_source` | Nested blocks → valid MIR |
| `test_lower_multiple_functions_from_source` | Multiple functions → MIR program |
| `test_lower_string_literal_from_source` | String literal → Return |
| `test_lower_bool_operations_from_source` | Boolean ops → valid MIR |
| `test_lower_match_expression_from_source` | Match → multi-block MIR |

### Files Modified
- `bmb/src/mir/lower.rs` (+13 tests, +parse_and_lower helper)

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 715/715 PASS (was 702, +13 new MIR tests) |
| Clippy | PASS |
| MIR lowering tests | 27/27 PASS (was 14, +13 new) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Source-based tests are maintainable and high-value |
| Philosophy Alignment | 9/10 | Tests verify recent fixes (Cycle 52 grammar) |
| Test Quality | 9/10 | Each test verifies structural MIR properties |
| Code Quality | 10/10 | Clean helper function, consistent test patterns |
| **Average** | **9.6/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | codegen/llvm.rs still has 0 tests (5.5K LOC) | Future cycle: add LLVM codegen tests |
| I-02 | L | MIR optimize tests could use source-based approach too | Future improvement |
