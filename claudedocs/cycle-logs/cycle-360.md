# Cycle 360: Redundant type cast detection

## Date
2026-02-13

## Scope
Add lint warning for redundant type casts (casting a value to its own type).

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
- `RedundantCast { ty, span }` — casting a value to its own type is redundant

Warning kind: `redundant_cast`

### Detection Logic (types/mod.rs)
Added check in `Expr::Cast` handler:
- Compares `src_ty == target_ty` before validating the cast
- Emits warning if types are identical

### Tests
- 3 positive tests: i64→i64, f64→f64, bool→bool
- 2 negative tests: i64→f64, f64→i64 (valid conversions)

## Test Results
- Standard tests: 4151 / 4151 passed (+5)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Exact type equality check |
| Architecture | 10/10 | Follows existing warning infrastructure |
| Philosophy Alignment | 10/10 | Catches unnecessary casts early |
| Test Quality | 10/10 | Positive + negative coverage |
| Code Quality | 10/10 | Minimal, clean implementation |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 361: Linter dedicated test suite
