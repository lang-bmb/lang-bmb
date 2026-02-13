# Cycle 397: Redundant if-expression detection lint rule

## Date
2026-02-13

## Scope
Add new lint rule to detect redundant if-expressions where both branches return boolean literals (`if c { true } else { false }` → `c`).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Core Change
- Added `RedundantIfExpression` variant to `CompileWarning` enum
- Detection in type checker's If expression handling unwraps single-element Block wrappers
- Detects both `true/false` (→ `cond`) and `false/true` (→ `not cond`) patterns

### Tests (7 new)
| Test | Description |
|------|-------------|
| test_warning_redundant_if_expression | Constructor for "cond" suggestion |
| test_warning_redundant_if_expression_negated | Constructor for "not cond" suggestion |
| test_redundant_if_true_false | `if x { true } else { false }` triggers |
| test_redundant_if_false_true | `if x { false } else { true }` triggers |
| test_no_redundant_if_both_true | Both true — no warning |
| test_no_redundant_if_non_bool_branches | Integer branches — no warning |
| test_no_redundant_if_expr_branches | Variable branches — no warning |

## Test Results
- Unit tests: 2185 passed (+2)
- Main tests: 15 passed
- Integration tests: 2189 passed (+5)
- Gotgan tests: 23 passed
- **Total: 4412 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, handles Block wrapper |
| Architecture | 10/10 | Follows existing lint pattern |
| Philosophy Alignment | 10/10 | Eliminates redundant code |
| Test Quality | 10/10 | Positive + negative cases |
| Code Quality | 10/10 | Clean, clippy-clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 398: Bitwise identity/absorbing detection lint (`x & 0`, `x | 0`, `x ^ 0`)
