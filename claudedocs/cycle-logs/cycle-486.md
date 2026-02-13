# Cycle 486: set var = expr in Rust Compiler Parser

## Date
2025-02-12

## Scope
Add `set var = expr` syntax support to the Rust-based compiler's parser. The bootstrap compiler already supported this (Cycle 484), but the Rust compiler's `set` keyword only handled index/field/deref assignments.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Rust compiler's `set` grammar handled 3 cases: `set arr[i] = expr`, `set obj.field = expr`, `set *ptr = expr`
- All other layers (AST `Expr::Assign`, type checker, MIR `Assign`, interpreter, LLVM codegen) already supported variable mutation
- Only the parser was missing: panicked with "Expected index, field access, or dereference" on `set x = expr`
- Fix: Add `Expr::Var(name)` case in both `set` grammar rules → `Expr::Assign { name, value }`

## Implementation

### Files Modified
1. **`bmb/src/grammar.lalrpop`**:
   - Added `Expr::Var(name) => Expr::Assign { name, value }` to `BlockExpr` set rule (line ~1276)
   - Added same case to expression-context set rule (line ~1424)
   - Updated error message to include "variable" in expected types

2. **`bmb/tests/integration.rs`** (5 new tests):
   - `test_set_var_basic`: Basic `set x = 42`
   - `test_set_var_multiple`: Multiple sets on same variable
   - `test_set_var_in_while_loop`: Accumulation with set in while loop
   - `test_set_var_in_for_loop`: Accumulation with set in for range loop
   - `test_set_var_conditional`: Conditional set inside loop

### Key Design Decisions
- **Maps to existing `Expr::Assign`**: `set x = expr` produces the same AST as `x = expr`. No new AST node needed.
- **Two grammar locations**: The `set` rule appears in both `BlockExpr` and a general expression rule. Both updated.
- **Consistent with bootstrap**: Bootstrap's `set var = expr` → `(set_var <name> expr)` maps to the same semantic.

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,234 passed (2845+47+2319+23) |
| Golden tests (Stage 1) | 26/26 PASS |
| Fixed point (S2==S3) | VERIFIED |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, set var works in all contexts |
| Architecture | 10/10 | Minimal change, reuses existing Expr::Assign |
| Philosophy Alignment | 10/10 | Closes gap between bootstrap and Rust compiler |
| Test Quality | 9/10 | 5 integration tests covering basic, multi-set, while/for loops, conditional |
| Documentation | 9/10 | Version comments on grammar changes |
| Code Quality | 10/10 | Two-line change per grammar rule, clean pattern match |
| **Average** | **9.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No type checker enforcement that `set` requires `let mut` | Future: add mutability check in type checker |
| I-02 | L | `set var` and `var = expr` (without set) both produce same AST | By design — `set` is preferred syntax |

## Next Cycle Recommendation
- Cycle 487: Bootstrap or Rust compiler feature additions
  - Nullable T? lowering (v0.92 roadmap item)
  - String concatenation optimization in bootstrap
  - Additional bootstrap features for broader program compilation
