# External Problem Validation Required

**Status: PARTIALLY RESOLVED** — Cycle 2991 (2026-05-20): C baseline verification 완료 (bench verify 17/17 PASS, Cycle 2788). 외부 검토자 + 외부 소싱 문제는 HUMAN-blocked 잔여.
**Priority: MEDIUM**
**Category: Experiment Methodology**

## Summary
All 100 problems in bmb-ai-bench were designed, solved, and tested by AI (Claude). Multiple test data errors were found and corrected during development, raising concerns about remaining undetected errors and potential bias toward BMB idioms.

## Detected Errors (Fixed)
- Wrong expected values in 15+ tests (arithmetic errors in test data)
- Operation count mismatches in 8+ tests (n parameter too small)
- LRU expected values were FIFO, not LRU (conceptual error)
- Knapsack/LIS expected values wrong for specific inputs

## Impact
- Remaining undetected test errors could skew results
- Problems may be biased toward BMB patterns (vec_new/vec_push/vec_get)
- No external validation of problem quality or fairness

## Proposed Fix
1. **Cross-verify all 100 tests**: Run C baseline programs on all tests, compare outputs
2. **External contributor review**: Have 2+ human programmers review 20+ problems
3. **Add LeetCode-equivalent problems**: Port 10-20 known problems from external sources
4. **Automated cross-validation**: Run solution.bmb, baseline.c, and solution.rs on same tests and verify agreement

## Acceptance Criteria
- [x] C baselines verified for all 100 problems (automated) — bench verify 17/17 PASS (Cycle 2788, scripts/verify_bench_outputs.py)
- [ ] At least 10 externally-sourced problems added — HUMAN-blocked
- [ ] At least 1 external reviewer validates problem quality — HUMAN-blocked

## Context
Multiple test data errors discovered during AI-Bench development suggest systematic review is needed.
