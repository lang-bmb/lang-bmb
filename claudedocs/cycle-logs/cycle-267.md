# Cycle 267: For-In Array Iteration

## Date
2026-02-12

## Scope
Add array iteration support to `for x in arr { ... }` loops in both the type checker and interpreter. Previously only Range and Receiver<T> were supported as iterables.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Type checker `Expr::For` only accepted `Type::Range` and `Type::Receiver<T>`
- Interpreter `Expr::For` only matched `Value::Range`
- Array element type available from `Type::Array(elem, _)` for type binding
- Break/continue already handled in Range iteration — same pattern reused

## Implementation

### Type Checker (`bmb/src/types/mod.rs`)
- Added `Type::Array(elem, _)` arm in `Expr::For` iterable match
- Element type extracted from array type for loop variable binding
- Updated error message to mention Array as valid iterable

### Interpreter (`bmb/src/interp/eval.rs`)
- Added `Value::Array(elements)` arm in `Expr::For` evaluation
- Iterates over array elements, binding each to loop variable
- Supports break/continue (same pattern as Range iteration)

### Integration Tests (`bmb/tests/integration.rs`)
Added 7 new tests:
- `test_for_in_array_sum`: Sum elements of array
- `test_for_in_array_count`: Count elements
- `test_for_in_array_single`: Single-element array
- `test_for_in_array_string`: Iterate over string array
- `test_for_in_array_nested`: Nested for-in loops
- `test_for_in_array_break`: Break from array iteration
- `test_for_in_array_type_error`: Non-iterable rejected

## Test Results
- Standard tests: 3364 / 3364 passed (+7 from 3357)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All iteration patterns work |
| Architecture | 10/10 | Follows existing Range iteration pattern |
| Philosophy Alignment | 10/10 | Natural language feature extension |
| Test Quality | 10/10 | Covers sum, count, string, nested, break, error |
| Code Quality | 10/10 | 12-line addition, reuses existing infrastructure |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No `continue` test for array iteration | Works identically to Range, low risk |
| I-02 | L | MIR/codegen doesn't support for-in-array yet | Interpreter-only for now |

## Next Cycle Recommendation
- Interpreter closure improvements
- WASM codegen for method calls
- Type checker edge cases
