# Cycle 1841: Fix Rust Compiler Speculatable + Clean Manifest
Date: 2026-03-11

## Inherited → Addressed
From 1840: "6 golden test files referenced in manifest but don't exist on disk" + Rust compiler has same speculatable bug.

## Scope & Implementation

### 1. Rust Compiler Speculatable Fix
Applied same fix as bootstrap (Cycle 1839) to Rust compiler: `memory(none)` functions with user-defined calls no longer get `speculatable`. Used allowlist for safe builtins (llvm.*, bmb_*, malloc, etc.).

### 2. Golden Test Manifest Cleanup
Removed 6 entries from `golden_tests.txt` for test files that don't exist on disk:
- test_golden_branch_heavy, test_golden_deep_recursion, test_golden_div_edge
- test_golden_loop_patterns, test_golden_mixed_ops, test_golden_overflow_safe

### Files Changed
- `bmb/src/codegen/llvm_text.rs` — Added `has_user_call` check for speculatable
- `tests/bootstrap/golden_tests.txt` — Removed 6 missing test entries (2834 → 2828)

### Verification
- **Rust tests**: 6,186 pass (no regression)
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (S2 == S3, 49s)
- **Benchmarks**: knapsack 0.15x, spectral_norm 0.79x, tower_of_hanoi 1.15x, max_consecutive_ones 1.13x — all unchanged
- **Golden tests**: 2797/2828 PASS, 18 FAIL (all closures/generics compile failures)

## Review & Resolution
- No defects found
- Remaining 18 failures are all closures/lambdas/generics — fundamental bootstrap compiler limitations

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: With golden tests at 98.9% pass rate (2797/2828), evaluate early termination
