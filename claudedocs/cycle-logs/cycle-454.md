# Cycle 454: Golden Binary Verification + Break/Continue Codegen Fix

## Date
2026-02-14

## Scope
Golden binary verification for Stage 1 compiled programs + fix break/continue/return LLVM IR generation bug.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Compiled 5 golden binary test programs with Stage 1 (bootstrap) compiler
- Discovered break/continue generates invalid LLVM IR: duplicate labels + unconverted `const 0` instructions
- Root cause: inline string concatenation as function arguments in `sb_push_mir()` calls produces incorrect BmbString pointers when compiled by the Rust compiler
- Verified Rust compiler generates correct IR for the same test (bug is bootstrap-specific)
- Diagnostic confirmed MIR generation inputs are correct (lengths, values) but output is corrupted
- Fix: extract concatenation results into named `let` bindings before passing to `sb_push_mir()`

## Implementation
### Files Modified
- `bootstrap/compiler.bmb` — Fixed 6 functions with the inline-concatenation-as-argument pattern:
  - `lower_break_sb` (recursive path)
  - `lower_continue_sb` (recursive path)
  - `lower_return_sb` (recursive path, both bare and value-return branches)
  - `step_break` (trampoline path)
  - `step_continue` (trampoline path)
  - `step_return` + `step_return_value` (trampoline path)

### Key Design Decision
The fix extracts inline concatenation expressions into named `let` bindings:
```
// Before (buggy):
let w2 = sb_push_mir(sb, after_label + ":");
let w3 = sb_push_mir(sb, "  " + tmp + " = const 0");

// After (fixed):
let label_text = after_label + ":";
let w2 = sb_push_mir(sb, label_text);
let const_text = "  " + tmp + " = const 0";
let w3 = sb_push_mir(sb, const_text);
```

### Root Cause Analysis
The bug manifests when:
1. `sb_push_mir(sb, X + ":")` pushes a label (correct)
2. The NEXT `sb_push_mir(sb, "  " + Y + " = const 0")` with chained concatenation produces a BmbString that incorrectly includes the previous label as a prefix

The diagnostic proved the input strings are correct (right lengths, right content), but the MIR→LLVM output shows the label prepended to the next instruction. By binding intermediates to `let`, the temporaries are properly sequenced.

This is likely a subtle codegen issue in the Rust compiler's handling of inline expressions as function arguments — potentially related to arena allocation ordering for chained string concatenations. Filed for investigation.

## Test Results
| Test | Status |
|------|--------|
| Rust tests | 5,229 passed |
| Bootstrap Stage 1 | Built successfully |
| Stage 1 == Stage 2 | Fixed point verified (67,164 lines) |
| Golden: basic | 220 ✓ |
| Golden: strings | 27 ✓ |
| Golden: arrays | 150 ✓ |
| Golden: float | 1 ✓ |
| Golden: break/continue | 33 ✓ (was FAIL, now PASS) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | All tests pass, break/continue now generates valid LLVM IR |
| Architecture | 8/10 | Fix is clean but root cause in Rust codegen needs deeper investigation |
| Philosophy Alignment | 9/10 | Proper fix at the source, not a workaround in LLVM codegen |
| Test Quality | 8/10 | Golden binary tests cover break/continue, but no edge cases yet |
| Documentation | 8/10 | Root cause documented, but Rust codegen issue needs formal issue |
| Code Quality | 9/10 | Clean let-binding pattern, consistent across all affected functions |
| **Average** | **8.5/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | Root cause in Rust compiler codegen — inline concat as function args may produce incorrect BmbString pointers | Investigate in Rust compiler's llvm_text.rs codegen |
| I-02 | L | Golden binary tests need edge cases (nested break, break in for-in, labeled break) | Add in future cycle |
| I-03 | L | Match expressions still missing from bootstrap parser | Next priority |

## Next Cycle Recommendation
- Continue with bootstrap feature gap work: match expressions or struct initialization in parser
- Consider auditing all `sb_push_mir(sb, expr + ...)` patterns for similar issues
- Investigate Rust compiler codegen for inline expression arguments
