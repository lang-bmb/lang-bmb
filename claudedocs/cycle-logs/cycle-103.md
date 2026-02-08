# Cycle 103: Nullable Null Sentinel Bug Fix + E2E Verification

## Date
2026-02-09

## Scope
Verify nullable types work end-to-end through codegen and fix any bugs found.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Nullable `T?` is represented as `T` at MIR/IR level (zero overhead)
- `null` literal lowered as `Constant::Int(0)` in MIR
- Nullable methods (is_some, is_none, unwrap_or) existed but used `-1` as null sentinel
- **Bug**: Null sentinel mismatch between `Expr::Null → 0` and methods checking `-1`

## Implementation

### Bug Fix: Null Sentinel Mismatch (mir/lower.rs)

**Root cause**: `Expr::Null` lowered to `Constant::Int(0)` but `is_some()`, `is_none()`, `unwrap_or()` compared against `-1`.

**Fix**: Changed all nullable method MIR lowering to use `0` as null sentinel, consistent with `Expr::Null`.

### Files Modified
- `bmb/src/mir/lower.rs` — Null sentinel fix (3 methods: is_some, is_none, unwrap_or)

### E2E Verification Results
```
find_positive(42).is_some()     → true  ✅
find_positive(0).is_none()      → true  ✅
find_positive(42).unwrap_or(0)  → 42    ✅
find_positive(0).unwrap_or(-1)  → -1    ✅ (was 0 before fix!)
```

## Test Results
- Tests: 1708 / 1708 passed
- Bootstrap: Stage 1 PASS (682ms)
- E2E nullable compilation: PASS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Root cause identified and fixed |
| Architecture | 9/10 | Zero-overhead nullable representation preserved |
| Philosophy Alignment | 10/10 | Root cause fix, bug found through verification |
| Test Quality | 7/10 | E2E verified but no MIR-level test for sentinel |
| Documentation | 8/10 | Clear code comments |
| Code Quality | 9/10 | Minimal, focused fix |
| **Average** | **8.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | Nullable i64? can't distinguish value 0 from null | Known design limitation — use *i64 pointer for nullable integers that might be 0 |
| I-02 | L | No MIR-level unit test for null sentinel consistency | Add in future cycle |

## Next Cycle Recommendation
Add more tests around nullable patterns. Consider adding interpreter-level nullable tests.
