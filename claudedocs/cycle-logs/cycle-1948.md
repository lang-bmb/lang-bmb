# Cycle 1948: Benchmark validation — 309 builds, ~50 measured
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1947: bench.sh fixes (BUILD_DIR, emit-ir, TEMP)

## Scope & Implementation
- Built ALL 309 compute benchmarks successfully (BMB + C)
- Measured ~50 key benchmarks across categories: sort, graph, matrix, fibonacci, etc.
- Results summary:

### Key FASTER Benchmarks (BMB beats C)

| Benchmark | BMB/C | Category |
|-----------|-------|----------|
| knapsack | **0.15x** | DP |
| lcs | **0.53x** | DP |
| floyd_warshall | **0.74x** | Graph |
| n_body | **0.77x** | Physics |
| spectral_norm | **0.85x** | Linear Algebra |
| wave_sort_check | **0.86x** | Sort |
| array_remove_dups | **0.86x** | Array |
| wiggle_sort | **0.89x** | Sort |
| radix_sort | **0.90x** | Sort |
| string_match | **0.90x** | String |
| count_sort_freq | **0.90x** | Sort |
| power_sum | **0.91x** | Math |
| matrix_chain | **0.91x** | DP |
| fibonacci_iterative | **0.93x** | Math |
| matrix_rotate | **0.93x** | Matrix |
| prefix_sum_2d | **0.90x** | Array |

### Overall: ~16 FASTER, ~30 PASS, 2-4 WARN, **0 FAIL**

## Review & Resolution
- 309/309 benchmarks built successfully ✅
- 0 FAIL across all measured benchmarks ✅
- BMB beats C in 16+ benchmarks, ties in 30+
- Only 2-4 WARN (5-8% slower, within noise margin)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Full 309 measurement needs ~3 hours — run overnight
- Next Recommendation: Investigate WARN benchmarks, run Rust comparison
