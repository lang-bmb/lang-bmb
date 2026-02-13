# Cycle 388: Absorbing element detection lint

## Date
2026-02-13

## Scope
Add lint rule to detect absorbing elements in arithmetic — expressions whose result is predetermined regardless of the other operand (`x * 0` is always 0, `x % 1` is always 0).

**Pivot**: Original roadmap planned "unreachable else-branch detection" — but BMB's expression-based if-else always requires both branches. Pivoted to absorbing element detection which complements the identity operation lint from Cycle 386.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Warning Variant
- `AbsorbingElement { expr: String, result: String, span: Span }`
- Message: `` `{expr}` is always `{result}` ``
- Kind: `"absorbing_element"`

### Detection Logic
In `types/mod.rs` at `Expr::Binary`:
- `Mul`: `x * 0` or `0 * x` → always 0
- `Mod`: `x % 1` → always 0

### Tests (5 new)
| Test | Description |
|------|-------------|
| test_absorbing_mul_zero_right | `x * 0` detected |
| test_absorbing_mul_zero_left | `0 * x` detected |
| test_absorbing_mod_one | `x % 1` detected |
| test_no_absorbing_normal_mul | `x * 3` NOT detected |
| test_no_absorbing_normal_mod | `x % 3` NOT detected |

## Test Results
- Standard tests: 4370 / 4370 passed (+5)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Follows existing lint pattern |
| Philosophy Alignment | 10/10 | Performance — redundant computation |
| Test Quality | 10/10 | 3 positive + 2 negative tests |
| Code Quality | 10/10 | Clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 389: Lint rule comprehensive test suite
