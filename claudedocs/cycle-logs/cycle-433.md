# Cycle 433: i32 integration tests — methods, structs, edge cases

## Date
2026-02-13

## Scope
Add integration tests verifying i32 type works end-to-end through the interpreter: methods (abs, min, max, clamp, to_string), struct fields, match expressions, boundary values, function chains, and cast chains.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Research
- Existing i32 tests: 38 type-check-only tests (Cycles 212-215), covering arithmetic, bitwise, comparison, casting, contracts
- Gap identified: No interpreter-level tests for i32 methods, struct fields, or edge cases
- Integer literal inference: bare literals default to i64; i32 method arguments require typed variables

### Tests Added (12 new integration tests)

| Test | Category | Description |
|------|----------|-------------|
| test_i32_method_abs | Method | `.abs()` on negative i32 |
| test_i32_method_min | Method | `.min(y)` with typed i32 variable |
| test_i32_method_max | Method | `.max(y)` with typed i32 variable |
| test_i32_method_clamp | Method | `.clamp(lo, hi)` with typed bounds |
| test_i32_method_to_string | Method | `.to_string()` conversion |
| test_i32_struct_field | Struct | Type-check struct with i32 fields |
| test_i32_struct_field_interp | Struct | Interpreter: struct field access + arithmetic |
| test_i32_in_match | Match | Match on i32 value with i64 return |
| test_i32_max_value | Boundary | i32 max (2147483647) |
| test_i32_min_value | Boundary | i32 min (-2147483648) |
| test_i32_function_chain | Chain | Multi-function i32 call chain |
| test_i32_nested_cast | Cast | i32 → i64 → f64 cast chain (type-check) |

### Key Finding
Integer literal arguments to i32 methods must be pre-bound to typed variables (e.g., `let y: i32 = 3; x.min(y)`) since bare literals default to i64. This is correct behavior — BMB requires explicit types, no implicit narrowing.

## Test Results
- Unit tests: 2845 passed
- Main tests: 47 passed
- Integration tests: 2274 passed (+12)
- Gotgan tests: 23 passed
- **Total: 5189 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 12 tests pass, covering 5 categories |
| Architecture | 10/10 | Tests follow existing integration test patterns |
| Philosophy Alignment | 10/10 | Verifies i32 end-to-end through interpreter |
| Test Quality | 10/10 | Methods, structs, match, boundaries, chains |
| Code Quality | 10/10 | Concise, follows convention, typed variables |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 434: i32 MIR + codegen integration tests — verify i32 LLVM IR generation
