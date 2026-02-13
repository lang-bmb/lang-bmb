# Cycle 396: Double negation detection lint rule

## Date
2026-02-13

## Scope
Add new lint rule to detect double negation patterns (`not not x`, `--x`, `bnot bnot x`) and warn that they can be simplified.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Core Change
- Added `DoubleNegation` variant to `CompileWarning` enum in `error/mod.rs`
- Added constructor `double_negation(op, span)`, `span()`, `message()`, `kind()` implementations
- Added detection in `types/mod.rs` Unary expr inference: when outer and inner unary ops match, emit warning

### Tests (7 new)
| Test | Description |
|------|-------------|
| test_warning_double_negation | Constructor for "not" op |
| test_warning_double_negation_neg | Constructor for "-" op |
| test_double_negation_not_not | `not not x` triggers warning |
| test_double_negation_neg_neg | `- -x` triggers warning |
| test_no_double_negation_single_not | Single `not x` no warning |
| test_no_double_negation_single_neg | Single `-x` no warning |
| test_no_double_negation_mixed_ops | Different ops no warning |

## Test Results
- Unit tests: 2183 passed (+2)
- Main tests: 15 passed
- Integration tests: 2184 passed (+5)
- Gotgan tests: 23 passed
- **Total: 4405 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Follows existing lint pattern exactly |
| Philosophy Alignment | 10/10 | Detects redundant code |
| Test Quality | 10/10 | Positive + negative tests |
| Code Quality | 10/10 | Clean, clippy-clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 397: New lint rule — redundant else after return
