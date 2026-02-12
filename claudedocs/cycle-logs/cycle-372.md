# Cycle 372: Constant condition detection lint

## Date
2026-02-13

## Scope
Add lint warning for constant conditions in if/while expressions (literal true/false).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (types/mod.rs)
- `Expr::If`: Detect `BoolLit(true/false)` as condition → emit warning
- `Expr::While`: Only detect `BoolLit(false)` (skip `while true` — intentional loop pattern)

### Error Module (error/mod.rs)
- `ConstantCondition { value: bool, context: String, span: Span }` variant
- Constructor: `constant_condition(value, context, span)`
- Message: `` `{context}` condition is always `{value}` ``
- Kind: `"constant_condition"`

### Tests (5 new)
| Test | Type | Description |
|------|------|-------------|
| test_lint_constant_condition_if_true | positive | `if true` triggers warning |
| test_lint_constant_condition_if_false | positive | `if false` triggers warning |
| test_lint_constant_condition_while_false | positive | `while false` triggers warning |
| test_lint_no_constant_condition_variable | negative | variable condition no warning |
| test_lint_no_constant_condition_while_true | negative | `while true` intentional — no warning |

## Test Results
- Standard tests: 4245 / 4245 passed (+5)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Properly handles all cases including while true exception |
| Architecture | 10/10 | Follows established warning pattern |
| Philosophy Alignment | 10/10 | Catches dead code, improves DX |
| Test Quality | 10/10 | 3 positive + 2 negative tests |
| Code Quality | 10/10 | Clean, minimal implementation |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 373: Self-comparison detection (x == x, x != x)
