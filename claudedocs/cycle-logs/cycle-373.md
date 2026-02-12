# Cycle 373: Self-comparison detection lint

## Date
2026-02-13

## Scope
Add lint warning for self-comparison patterns (x == x, x != x, x < x, etc.).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (types/mod.rs)
- `Expr::Binary`: Detect when both sides are `Var` with same name and op is comparison
- Uses if-let chain: `if let (Expr::Var(l), Expr::Var(r)) && l == r && is_comparison(op)`

### Error Module (error/mod.rs)
- `SelfComparison { name, op, span }` variant
- Message: `"comparing \`{name}\` {op} \`{name}\` is always the same result"`
- Kind: `"self_comparison"`

### Tests (5 new)
| Test | Type | Description |
|------|------|-------------|
| test_lint_self_comparison_eq | positive | `x == x` triggers warning |
| test_lint_self_comparison_ne | positive | `x != x` triggers warning |
| test_lint_self_comparison_lt | positive | `x < x` triggers warning |
| test_lint_no_self_comparison_different_vars | negative | `x == y` no warning |
| test_lint_no_self_comparison_arithmetic | negative | `x + x` no warning |

## Test Results
- Standard tests: 4250 / 4250 passed (+5)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Catches all comparison ops, only simple var patterns |
| Architecture | 10/10 | Follows established warning pattern |
| Philosophy Alignment | 10/10 | Catches likely bugs |
| Test Quality | 10/10 | 3 positive + 2 negative tests |
| Code Quality | 10/10 | Clean if-let chain, clippy clean |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 374: Redundant boolean comparison detection
