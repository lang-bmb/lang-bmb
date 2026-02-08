# Cycle 62: Add Exhaustiveness Checker Tests

## Date
2026-02-08

## Scope
Add 22 tests to `types/exhaustiveness.rs` (was 10 tests, 1582 LOC). Cover tuple patterns, struct patterns, Or-patterns, variable/binding patterns, guard handling, 3+ variant enums, multiple unreachable arms, string literals, and context API.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Changes

### bmb/src/types/exhaustiveness.rs (+22 tests)

**Test categories:**

#### Pattern Coverage (8 tests)
| Test | What it verifies |
|------|-----------------|
| `test_variable_pattern_exhaustive` | `Var("x")` matches everything like wildcard |
| `test_tuple_exhaustive_with_wildcard` | `(_, _)` covers all tuple values |
| `test_tuple_non_exhaustive` | `(true, true)` misses other bool combinations |
| `test_struct_pattern_exhaustive` | `Point { x: _, y: _ }` covers all |
| `test_or_pattern_exhaustive` | `true \| false` covers all booleans |
| `test_or_pattern_non_exhaustive` | `true \| 42` doesn't cover false |
| `test_binding_pattern_exhaustive` | `x @ _` binding is exhaustive |
| `test_range_exclusive_end` | `0..10` (exclusive) + wildcard is exhaustive |

#### Guard Handling (2 tests)
| Test | What it verifies |
|------|-----------------|
| `test_guard_without_fallback` | `_ if cond` → `has_guards_without_fallback = true` |
| `test_guard_with_fallback` | Guarded arm + wildcard fallback → no warning |

#### Enum Variants (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_enum_three_variants` | Red/Green/Blue all covered → exhaustive |
| `test_enum_three_variants_missing_one` | Red/Green only → "Color::Blue" missing |
| `test_enum_with_wildcard_exhaustive` | Wildcard covers all enum variants |

#### Unreachable Arms (2 tests)
| Test | What it verifies |
|------|-----------------|
| `test_multiple_unreachable_arms` | `_ → true → false` → arms [1,2] unreachable |
| `test_bool_duplicate_arm_unreachable` | Duplicate `true` arm detected as unreachable |

#### Literal Coverage (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_string_literal_non_exhaustive` | "hello"/"world" can't exhaust String |
| `test_string_with_wildcard_exhaustive` | String literal + wildcard is exhaustive |
| `test_int_literal_multiple_non_exhaustive` | 0/1/2 can't exhaust i64 |

#### Context API (2 tests)
| Test | What it verifies |
|------|-----------------|
| `test_context_get_struct_field_type` | Struct field lookup (found/not found) |
| `test_context_get_enum_variants` | Enum variant listing |

#### Utility (2 tests)
| Test | What it verifies |
|------|-----------------|
| `test_empty_match_non_exhaustive` | Zero arms → non-exhaustive |
| `test_format_missing_pattern` | Identity formatting |

### Files Modified
- `bmb/src/types/exhaustiveness.rs` (+22 tests)

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 831/831 PASS (was 809, +22) |
| Clippy | PASS (0 warnings) |

## Notes
- Initial compile error: `crate::ast::Expr::Literal` doesn't exist. BMB uses `Expr::BoolLit(true)` for guard expressions. Fixed immediately.
- Module went from 10 tests to 32 tests (220% increase)

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 831 tests pass |
| Architecture | 10/10 | Tests all pattern types and API surface |
| Philosophy Alignment | 10/10 | Compile-time exhaustiveness checking is core BMB |
| Test Quality | 10/10 | Positive/negative cases, edge cases covered |
| Code Quality | 10/10 | Consistent with existing test patterns |
| **Average** | **10.0/10** | |
