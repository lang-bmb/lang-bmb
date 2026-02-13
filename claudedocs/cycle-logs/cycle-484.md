# Cycle 484: Local Variable Mutation — set var = expr

## Date
2025-02-12

## Scope
Add `set var = expr` support for mutable local variables declared with `let mut`.
Enables natural loop accumulation patterns without array-based workarounds.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Bootstrap already had `let_mut` support (v0.92) with alloca+store+load pattern
- For loop variables already used alloca for mutation
- `set var = expr` needed: parser + step machine + recursive lowering
- Variable reference (`copy %name`) already works for alloca'd variables
- Known limitation: duplicate variable names in same function cause LLVM name collision (pre-existing)

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - `parse_set_expr`: Added `TK_EQ()` branch for `set var = expr` → `(set_var <name> expr)` AST
   - Step dispatch: Added SV (byte 86='V') and SW (byte 87='W') handlers
   - `step_set_var_start`: Evaluates value expression, pushes SW continuation
   - `step_set_var_final`: Emits `store %name, %result_temp`, returns unit
   - `lower_set_var_sb`: Recursive path — lower value, emit store, return unit
   - `lower_expr_sb`: Added `set_var` case to dispatch chain

2. **`tests/bootstrap/test_golden_set_var.bmb`** (NEW):
   - Tests: basic set, multiple sets, for-range accumulation, for-in accumulation,
     nested loops with counter, conditional set inside loop
   - Expected output: 179

3. **`tests/bootstrap/golden_tests.txt`**: Added set_var test

### Key Design Decisions
- **Builds on existing `let_mut` infrastructure**: `let mut x = 5` already emits `alloca %x` + `store %x, initial`. `set x = expr` simply emits `store %x, result`.
- **Two-step step machine**: SV evaluates value expression, SW emits the store. Same pattern as set_index/set_field.
- **Returns unit (const 0)**: Consistent with set_index and set_field semantics.
- **No SSA renaming needed**: Variables declared with `let_mut` use alloca, so LLVM handles the phi-node insertion via mem2reg.

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 25/25 PASS |
| Golden tests (Stage 2) | 25/25 PASS |
| Fixed point (S2==S3) | VERIFIED (73,268 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, set_var works in all contexts (basic, loops, for-in, nested) |
| Architecture | 9/10 | Clean integration with existing alloca infrastructure |
| Philosophy Alignment | 10/10 | Proper language feature, not a workaround |
| Test Quality | 9/10 | 6 test scenarios covering basic, multi-set, loops, for-in, nested, conditional |
| Documentation | 9/10 | Version comments, step machine comments |
| Code Quality | 9/10 | Minimal additions, follows existing patterns |
| **Average** | **9.3/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | Duplicate variable names in same function cause LLVM name collision (pre-existing) | Future: unique alloca names via block_id suffix |
| I-02 | L | No error if `set` is used on non-`let_mut` variable | Bootstrap doesn't type-check, deferred to type checker |
| I-03 | L | Rust compiler MIR lowering also lacks `set var` support | Future cycle |

## Next Cycle Recommendation
- Cycle 485: While loop accumulation patterns OR additional bootstrap features
  - `while` loops with `set` enable imperative algorithms
  - Alternative: String builder / string concatenation optimization
  - Alternative: Nullable T? support (roadmap v0.92 item)
