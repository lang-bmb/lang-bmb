# Changelog

All notable changes to bmb-algo will be documented in this file.

## [Unreleased]

### Documentation
- README benchmark table re-baselined to v0.98 measurements (2026-05-12). Supersedes v0.2.0 numbers from 2026-03-23.
- Headline updated to **"Up to ~450× (knapsack(100))"** — confirms v0.2.0 90× knapsack as the n=10 floor; new measurement shows the speedup scales with input size.
- Added `knapsack(100)` and `quicksort(1000)` to the benchmark suite to demonstrate **scaling behavior**: BMB's advantage amplifies as algorithmic work exceeds FFI overhead.
- Added a **scaling table** showing knapsack speedup at n=10/30/100/300 and quicksort at n=15/100/1000.
- Disclosed quicksort(15) regression with input-size guidance — FFI overhead crossover at ~n=50; recommend ≥100 elements for measurable speedup.
- v0.2.0 `nqueens(8) 181.6×` flagged as not reproducible at any tested size — likely a baseline configuration difference.

### Benchmarks
- `bench_algo.py`: added `knapsack(100)`, `quicksort(1000)` runs. Existing small-input runs retained for the FFI overhead reference point.

## [0.3.0] - 2026-03-23

### Added
- 6 new algorithms: is_palindrome_num, digit_sum, kth_smallest, array_mode, sorted_intersect_count, two_sum
- 4 more sorting algorithms: shell_sort, insertion_sort, selection_sort, bubble_sort
- is_prime primality test, array_product, subset_sum, matrix_det
- C header file generation (include/bmb_algo.h)
- WASM output support (229 KB WAT)
- `__all__` for clean import * behavior

### Changed
- Total algorithms: 41 -> 55

## [0.2.0] - 2026-03-23

### Added
- 41 algorithms: DP (6), Graph (4), Sort (4), Search (1), Number Theory (6), Matrix (2), Utility (18)
- Python ctypes bindings with FFI safety (setjmp/longjmp error handling)
- Type stubs (.pyi) for IDE autocomplete
- pytest test suite (189 tests)
- Benchmark script vs pure Python (6x-32x faster)
- pyproject.toml (PEP 621 compliant)

### Performance
- knapsack: 90.7x faster than Python, 6.8x faster than C
- nqueens(8): 181.6x faster than Python
- prime_count(10k): 25.6x faster than Python

## [0.1.0] - 2026-03-15

### Added
- Initial release: 8 algorithms (knapsack, lcs, edit_distance, floyd, dijkstra, quicksort, merge_sort, binary_search)
- Python ctypes bindings
