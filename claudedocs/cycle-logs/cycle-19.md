# Cycle 19: Fix Grammar Warnings

## Date
2026-02-07

## Scope
Eliminate 12 unused variable warnings from lalrpop-generated grammar code by removing unnecessary `<l:@L>` and `<r:@R>` span captures from 6 concurrency constructor rules.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 3/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- **Root cause**: 6 concurrency grammar rules (AtomicNew, MutexNew, ChannelNew, RwLockNew, BarrierNew, CondvarNew) captured `<l:@L>` and `<r:@R>` location spans but never used them in AST node construction
- **Fix**: Simply remove the unused captures — these AST nodes don't include span information unlike other spanned expressions

## Implementation
- Modified `bmb/src/grammar.lalrpop`: removed `<l:@L>` and `<r:@R>` from 6 rules
- Net change: -6 captures × 2 variables = 12 fewer warnings

## Test Results
- Rust tests: 357/357 passed
- Build warnings: 0 (down from 12)
- All 23 concurrency type-check tests pass (parsing unaffected)
- Bootstrap Stage 1: compiles successfully

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, parsing unchanged |
| Architecture | 9/10 | Clean removal, not suppression |
| Philosophy Alignment | 8/10 | Build quality, proper fix |
| Test Quality | 10/10 | 357 tests + bootstrap verified |
| Documentation | 8/10 | Cycle log written |
| Code Quality | 10/10 | Zero warnings across entire build |
| **Average** | **9.2/10** | |

## Next Cycle Recommendation
Add MIR lowering integration tests.
