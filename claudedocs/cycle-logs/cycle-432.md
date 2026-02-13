# Cycle 432: Fix phi type inference (mixed ptr/i64)

## Date
2026-02-13

## Scope
Fix the last remaining open issue: phi type inference bug in llvm_text.rs where i64 had higher priority than ptr, causing incorrect type for string conditional branches. Also add 5 integration tests for string if/else branches.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Bug Fix
- **Root cause**: In `llvm_text.rs` phi type widening, the match arm ordering was `i64 > double > ptr`. When a phi node had one ptr value and one i64 value (from ptrtoint), i64 won incorrectly.
- **Fix**: Reordered match arms to `ptr > double > i64 > i32 > i1` in two locations:
  - Line 948: Phi node type inference
  - Line 907: Binary operation type inference
- **llvm.rs**: Already fixed in v0.90 (pointer takes priority via explicit `if let PointerType` check)

### Tests (5 new integration tests)
| Test | Description |
|------|-------------|
| test_string_if_else_branch | true branch returns "yes" |
| test_string_if_else_branch_false | false branch returns "no" |
| test_string_conditional_with_variable | Function returning conditional string |
| test_string_conditional_len | .len() on conditional string result |
| test_string_conditional_nested | Multiple conditional string assignments |

### Issue Resolution
- ISSUE-20260209-phi-type-inference.md marked as RESOLVED
- All 12 issues now RESOLVED (0 open)

## Test Results
- Unit tests: 2845 passed
- Main tests: 47 passed
- Integration tests: 2262 passed (+5)
- Gotgan tests: 23 passed
- **Total: 5177 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Bug fix verified, all tests pass |
| Architecture | 10/10 | Fix follows existing pattern in llvm.rs |
| Philosophy Alignment | 10/10 | Root cause fix, not workaround |
| Test Quality | 10/10 | 5 regression tests for string conditionals |
| Code Quality | 10/10 | Minimal 2-line change, clear comments |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 433: i32 integration tests — verify i32 type works end-to-end
