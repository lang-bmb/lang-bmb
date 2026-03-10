# 20-Cycle Roadmap: Housekeeping + Final FAIL Resolution (Cycles 1835-1854)
Date: 2026-03-10

## Goal
Clean up project state, close resolved issues, and make final attempts at resolving the 2 remaining benchmark FAILs (tower_of_hanoi 1.17x, max_consecutive_ones 1.13x).

## Phase 1: Housekeeping (Cycles 1835-1837)
- Close all 14 resolved issues (move to closed/)
- Fix bootstrap script to detect text/inkwell backend properly
- Clean up stale memory entries and documentation

## Phase 2: FAIL Analysis + Resolution (Cycles 1838-1845)
- Deep analysis of tower_of_hanoi 1.17x — compare assembly, identify specific divergence
- Deep analysis of max_consecutive_ones 1.13x — unroller behavior analysis
- Attempt MIR-level or codegen-level fixes
- If truly LLVM-only: document definitive evidence and close

## Phase 3: Quality + Verification (Cycles 1846-1854)
- Run full benchmark suite to check for any new WARNs
- Golden test analysis — which of the 39 failures are fixable?
- Final verification + version bump
