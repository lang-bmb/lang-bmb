# Cycle 359: Single-arm match detection (suggest if-let)

## Date
2026-02-13

## Scope
Add lint warning for match expressions with exactly 2 arms where one is a wildcard/variable catch-all, suggesting `if let` as a simpler alternative.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Warning Type (error/mod.rs)
Added 1 new warning variant:
- `SingleArmMatch { span }` — match with single specific arm + wildcard could use if-let

Warning kind: `single_arm_match`

### Detection Logic (types/mod.rs)
Added detection in `Expr::Match` handler after exhaustiveness checking:
- Fires when match has exactly 2 arms
- No guards present (guards make if-let less applicable)
- Exactly one arm is a catch-all (`Pattern::Wildcard` or `Pattern::Var(_)`)
- The other arm is a specific pattern

### Tests
- 3 positive tests: enum+wildcard, literal+wildcard, variable catch-all
- 2 negative tests: multi-arm match, two specific patterns (bool true/false)

## Test Results
- Standard tests: 4146 / 4146 passed (+5)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Accurate detection of single-arm + wildcard pattern |
| Architecture | 10/10 | Follows existing warning infrastructure |
| Philosophy Alignment | 10/10 | Helps AI write cleaner code |
| Test Quality | 10/10 | Positive + negative coverage |
| Code Quality | 10/10 | Minimal, clean implementation |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 360: Redundant type cast detection
