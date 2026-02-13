# Cycle 386: Identity operation detection lint

## Date
2026-02-13

## Scope
Add lint rule to detect identity operations (x + 0, 0 + x, x - 0, x * 1, 1 * x, x / 1) that have no effect and can be simplified.

**Pivot**: Original roadmap planned "unnecessary parentheses detection" but BMB's AST doesn't preserve parentheses (they're syntactic grouping only). Pivoted to identity operation detection — a useful lint implementable in the current architecture.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Warning Variant
- `IdentityOperation { expr: String, span: Span }`
- Message: `"identity operation \`{expr}\` has no effect"`
- Kind: `"identity_operation"`

### Detection Logic
In `types/mod.rs` at `Expr::Binary`:
- `Add`: `x + 0` or `0 + x`
- `Sub`: only `x - 0` (NOT `0 - x` which is negation)
- `Mul`: `x * 1` or `1 * x`
- `Div`: only `x / 1` (NOT `1 / x`)

### Tests (10 new)
| Test | Description |
|------|-------------|
| test_identity_add_zero_right | `x + 0` detected |
| test_identity_add_zero_left | `0 + x` detected |
| test_identity_sub_zero | `x - 0` detected |
| test_identity_mul_one_right | `x * 1` detected |
| test_identity_mul_one_left | `1 * x` detected |
| test_identity_div_one | `x / 1` detected |
| test_no_identity_sub_from_zero | `0 - x` NOT detected (negation) |
| test_no_identity_div_into_one | `1 / x` NOT detected |
| test_no_identity_normal_add | `x + 3` NOT detected |
| test_no_identity_normal_mul | `x * 2` NOT detected |

## Test Results
- Standard tests: 4360 / 4360 passed (+10)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All edge cases correct |
| Architecture | 10/10 | Follows existing lint pattern |
| Philosophy Alignment | 10/10 | Code quality improvement |
| Test Quality | 10/10 | 6 positive + 4 negative tests |
| Code Quality | 10/10 | Clean, asymmetric handling correct |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 387: Empty match body / unreachable else detection
