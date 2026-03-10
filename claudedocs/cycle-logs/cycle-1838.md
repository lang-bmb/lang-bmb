# Cycle 1838: Fix Bootstrap TRL False Positive — Non-Tail Calls Inside Loops
Date: 2026-03-11

## Inherited → Addressed
From 1837: "Fix bootstrap inttoptr aliasing for recursive array mutations" — ROOT CAUSE IDENTIFIED as incorrect TRL (Tail-Recursive-to-Loop) transformation, not inttoptr aliasing.

## Scope & Implementation

### Root Cause
The bootstrap compiler's `trl_find_tail_call()` in `compiler.bmb` only checked that a self-call was followed by `goto ...`, without verifying the goto target leads to a `return`. This caused it to falsely identify calls INSIDE while loops as tail calls, incorrectly transforming them into loop jumps and dropping all remaining loop iterations and post-loop code.

### Fix
Added `trl_block_has_return()` and `trl_scan_block_for_return()` helper functions that:
1. Extract the goto target label after a self-call
2. Find the target block in the MIR body
3. Verify the block contains a `return` statement (not loop continuation)

Only if the goto target block leads to `return` (possibly via phi) is the call considered a true tail call.

### Files Changed
- `bootstrap/compiler.bmb` — Modified `trl_find_tail_call()`, added `trl_block_has_return()`, `trl_scan_block_for_return()`

### Verification
- **scc_kosaraju**: 1 → 3 (FIXED, expected 3)
- **tree_diameter**: 3 → 4 (FIXED, expected 4)
- **fact (proper tail recursion)**: Still correctly TRL'd into a loop, 3628800 output correct
- **Rust tests**: 6,186 pass (no regression)
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (S2 == S3, 57s total)

## Review & Resolution
- The previous hypothesis (inttoptr aliasing bug) was WRONG — the actual bug was incorrect TRL transformation
- The `norecurse` attribute was a secondary symptom: since TRL removed the recursive call, `mir_has_self_call` found no self-call
- Fix correctly prevents TRL for non-tail calls while preserving TRL for genuine tail recursion

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: ackermann golden test still fails (stack overflow from deep recursion, not TRL related — ackermann has 3 self-calls so TRL never applies)
- Next Recommendation: Run full golden test suite to measure how many of the 39 failures are resolved
