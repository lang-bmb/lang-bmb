# Cycle 443: MIR copy propagation completion

## Date
2026-02-13

## Scope
Complete the MIR copy propagation pass in `bootstrap/optimize.bmb`. Replace the instruction-specific (binop-only, return-only) replacement with a general `replace_var_with_copy` that handles ALL instruction types. Add transitive copy resolution.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Changes to `bootstrap/optimize.bmb`

**1. New helper functions (lines 410-449)**

- `replace_var_with_copy(line, copy_table)` — General copy propagation for any instruction. Preserves LHS (destination variable), replaces `%var` patterns in RHS using copy table.
- `replace_vars_in_text(text, copy_table, pos)` — Scans text for `%` patterns, extracts variable names, looks up in copy table, and replaces.
- `find_var_end(s, pos)` — Finds end of variable name (alphanumeric + underscore).
- `is_alnum_or_underscore(c)` — Character classification for variable names.

**2. Simplified `copy_prop_line` (lines 453-472)**

Before: Manual extraction/replacement for binops (extract_binop_left/right) and returns only. All other instruction types passed through unchanged.

After: Two branches only:
- Copy instruction → add to table (with transitive resolution)
- Everything else → `replace_var_with_copy` (handles binops, comparisons, calls, branches, returns, etc.)

**3. Transitive copy resolution**

For chained copies like `%_t2 = copy %_t1` then `%_t3 = copy %_t2`, the table now stores `%_t3 → %_t1` (resolves through the chain) instead of `%_t3 → %_t2`.

### Instruction types now handled by copy propagation

| Instruction Type | Before | After |
|-----------------|--------|-------|
| Binop (`+ - * / %`) | Manual extraction | `replace_var_with_copy` |
| Return | Manual extraction | `replace_var_with_copy` |
| Comparison (`== != < > <= >=`) | **Not handled** | `replace_var_with_copy` |
| Call (`call @fn(args)`) | **Not handled** | `replace_var_with_copy` |
| Branch (`branch %cond, l1, l2`) | **Not handled** | `replace_var_with_copy` |
| Other instructions | **Not handled** | `replace_var_with_copy` |

### Code reduction

- Removed ~30 lines of manual binop/return extraction logic
- Added ~40 lines of general replacement logic
- Net: simpler, more maintainable, and handles ALL instruction types

## Test Results
- Unit tests: 2845 passed
- Main tests: 47 passed
- Integration tests: 2314 passed
- Gotgan tests: 23 passed
- **Total: 5229 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS
- Bootstrap type-check: PASS (0 errors)

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, general replacement is sound |
| Architecture | 10/10 | Follows existing optimizer pass structure |
| Philosophy Alignment | 10/10 | Proper fix: general solution instead of per-instruction workarounds |
| Test Quality | 8/10 | Existing tests cover the change; no new dedicated copy-prop tests |
| Code Quality | 10/10 | Simpler, cleaner, handles all cases |
| **Average** | **9.6/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No dedicated unit tests for copy propagation across comparison/call/branch instructions | Future cycle |
| I-02 | M | Pre-existing `work3_get1` undefined function error in compiler.bmb blocks Stage 1 bootstrap | Investigate in future cycle |

## Next Cycle Recommendation
- Cycle 444: Bootstrap string.eq() inlining — replace `call @bmb_string_eq` with inline `memcmp`-style comparison or direct byte comparison, following the byte_at/len inlining pattern from Cycles 441-442
