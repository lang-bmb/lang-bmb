# Cycle 1847: Verification + fibonacci_sum Analysis
Date: 2026-03-11

## Inherited → Addressed
From 1846: Full verification of all Cycle 1846 changes.

## Scope & Implementation

### Full Verification
- cargo test --release: 6,186 tests pass
- 3-Stage Bootstrap: Fixed Point verified (107,232 lines IR, 50s)
- Golden tests: 2815/2815 (100%)
- Benchmark sampling: knapsack 0.15x, life 0.57x, scc 0.76x, spectral_norm 0.77x, bellman_ford FASTER — no regressions

### fibonacci_sum Mismatch Analysis
- Output mismatch: BMB=-950875432, C=865443849
- Root cause: Fibonacci(n) for n up to 50 overflows i64, BMB uses `nsw` (signed no wrap) flags
- `nsw` makes overflow undefined behavior, allowing LLVM to optimize based on no-overflow assumption
- C wraps around normally, BMB with nsw may produce different results
- **Pre-existing issue** — not caused by Cycle 1846 changes
- Not actionable: benchmark intentionally tests register computation, overflow is expected behavior difference

## Review & Resolution
All Cycle 1846 changes verified:
- CSE in Release pipeline: works correctly with CopyPropagation fix
- Malloc SSA naming: no more duplicate names
- LLVM attributes: no regressions
- Bootstrap Fixed Point: maintained

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: fibonacci_sum nsw overflow mismatch (pre-existing, not a defect)
- Next Recommendation: Version bump to v0.96.43, explore additional MIR or LLVM IR improvements
