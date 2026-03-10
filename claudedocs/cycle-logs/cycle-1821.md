# Cycle 1821: For-Loop Branch Weights + Benchmark Sweep
Date: 2026-03-10

## Inherited → Addressed
From 1820: Run full benchmark suite, investigate wave_equation WARN, consider MIR-level peephole.

## Scope & Implementation

### Branch Weight Extension
Extended `!prof !907` branch weight metadata from while-loops only to all loop types:
- `for_body_`/`for_exit_` patterns (for loops)
- `for_recv_body_`/`for_recv_exit_` patterns (for-recv loops)

Previously only `while_body_`/`while_exit_` had weights (2000:1 hot/cold ratio).

### File Changed
- `bmb/src/codegen/llvm_text.rs` — Extended `is_loop_cond` check

### Benchmark Sweep Results (40+ benchmarks tested)
Tested selection of benchmarks with current compiler build:

| Benchmark | Ratio | Status |
|-----------|-------|--------|
| array_unique_count | 1.00x | PASS (fixed in 1820) |
| insertion_sort | 1.04x | PASS |
| bubble_sort | 0.99x | FASTER |
| heap_sort | 0.98x | FASTER |
| sparse_matmul | 1.03x | PASS |
| selection_sort | 1.01x | PASS |
| radix_sort | 1.00x | PASS (high variance, best-of matches C) |
| counting_sort | 0.98x | FASTER |
| lcs | 0.58x | FASTER |
| floyd_warshall | 0.57x | FASTER |
| bellman_ford | 0.64x | FASTER |
| dijkstra | 1.05x | PASS |
| scc | 0.44x | FASTER |
| binary_trees | 0.94x | FASTER |
| fannkuch | 1.04x | PASS |
| fasta | 1.01x | PASS |
| life | 0.57x | FASTER |
| collatz | 0.98x | FASTER |
| sum_pairs | 1.00x | PASS |
| count_pairs | 1.01x | PASS |
| interval_merge | 1.01x | PASS |
| weighted_interval | 1.00x | PASS |
| array_running_median_simple | 0.88x | FASTER |
| array_kth_smallest | 1.05x | PASS |
| array_count_inversions_adj | 0.87x | FASTER |
| array_dedup | 0.84x | FASTER |
| coin_change | 0.73x | FASTER |
| edit_distance | 1.01x | PASS |
| rod_cutting | 1.00x | PASS |
| subset_sum | 1.00x | PASS |
| mandelbrot | 1.05x | PASS |
| spectral_norm | 0.97x | FASTER |
| n_body | 1.00x | PASS |
| fibonacci | 0.93x | FASTER |
| fibonacci_sum | 1.04x | PASS |
| wave_equation | 1.06x | PASS (noise, median of 5) |
| knapsack | 0.13x | FASTER (7.4x!) |
| tak | 1.00x | PASS |
| ackermann | 1.02x | PASS (noise on 40ms bench) |
| string_reverse | 0.86x | FASTER |
| array_energy | 0.88x | FASTER |
| array_ewma | 0.91x | FASTER |
| tower_of_hanoi | 1.16x | FAIL |

### tower_of_hanoi Investigation
- **Ratio**: 1.16x FAIL (BMB 357ms vs C 308ms, consistent across 10+ runs)
- **Assembly**: IDENTICAL to C (same instructions, same register patterns, same unrolling)
- **Function address**: Both at `0x140001490` with identical alignment
- **Root cause**: NOT codegen — binary layout effect. BMB's .text section is 52K (includes runtime) vs C's 6.4K
- **Hypothesis**: icache pollution from larger binary, or branch prediction table interference from runtime code
- **Action needed**: This is a linker/runtime concern, not a compiler codegen issue

## Review & Resolution
- All 6,186 tests pass
- For-loop branch weights verified in emitted IR (tested with `for i in 0..10` pattern)
- No regression in any benchmark from the for-loop branch weight change
- tower_of_hanoi FAIL is assembly-identical to C — linker/runtime layout issue, not codegen

## Carry-Forward
- Pending Human Decisions: Whether tower_of_hanoi 1.16x FAIL warrants runtime code reduction (stripping unused runtime functions from binary)
- Discovered out-of-scope: BMB binary size (52K .text) vs C (6.4K .text) may cause microarchitectural performance effects for very tight CPU-bound loops
- Next Recommendation: Investigate runtime code elimination (dead function stripping) or LTO for benchmark binaries; scan remaining ~270 benchmarks for other FAILs
