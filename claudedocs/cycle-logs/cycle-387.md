# Cycle 387: Negated if-condition detection lint

## Date
2026-02-13

## Scope
Add lint rule to detect negated if-conditions (`if not x { a } else { b }`) and suggest swapping branches to remove the negation.

**Pivot**: Original roadmap planned "empty block detection" — pivoted to negated if-condition detection which is more useful for code clarity.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Warning Variant
- `NegatedIfCondition { span: Span }`
- Message: `"negated if condition: consider swapping branches to remove negation"`
- Kind: `"negated_if_condition"`

### Detection Logic
In `types/mod.rs` at `Expr::If`: check if `cond` is `Expr::Unary { op: UnOp::Not, .. }`

### Tests (5 new)
| Test | Description |
|------|-------------|
| test_negated_if_condition_not_var | `if not x { ... }` detected |
| test_negated_if_condition_not_expr | `if not (a == b) { ... }` detected |
| test_no_negated_if_simple_var | `if x { ... }` NOT detected |
| test_no_negated_if_comparison | `if a > 0 { ... }` NOT detected |
| test_no_negated_if_bool_lit | `if true { ... }` NOT detected (fires constant_condition instead) |

## Test Results
- Standard tests: 4365 / 4365 passed (+5)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Follows existing lint pattern |
| Philosophy Alignment | 10/10 | Code clarity improvement |
| Test Quality | 10/10 | 2 positive + 3 negative tests |
| Code Quality | 10/10 | Minimal, clean detection |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 388: Redundant else-after-divergence detection lint
