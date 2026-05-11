# Changelog

All notable changes to bmb-algo will be documented in this file.

## [Unreleased]

### Documentation
- README benchmark table re-baselined to **median-of-5 measurements** at v0.98 (2026-05-12). Supersedes both v0.2.0 numbers (2026-03-23) and an intermediate Cycle 2754 single-pair sample that turned out to be an outlier.
- Headline updated to **"Up to ~245× (knapsack(100), median-of-5)"** with a note that the scaling table reaches ~306× at n=300. Earlier "~450×" claim withdrawn — it was a one-shot sample that does not reproduce on this machine under representative load.
- New baseline includes inter-run spread per row (min-max across 5 runs), exposing variance honestly rather than reporting a single value.
- Added a **scaling table** showing knapsack speedup at n=10/30/100/300 and quicksort at n=15/50/100/500/1000, reproducible with `bench_algo.py --runs=5 --scaling`.
- `quicksort(15)` is currently **~1.7× faster** (not slower) than pure Python under median-of-5 — the earlier "0.9× SLOW" disclosure was an artifact of the same one-shot sampling. The associated quicksort FFI overhead ISSUE is closed as not-reproducible.
- v0.2.0 `nqueens(8) 181.6×` remains not reproducible against the current baseline (n=8/10/12 all measure ~4-8×) — flagged in the historical archive.

### Benchmarks
- `bench_algo.py`: added `--runs=N` median-of-N harness with per-row min-max spread reporting.
- `bench_algo.py`: added `--scaling` sweep covering knapsack n=10/30/100/300 and quicksort n=15/50/100/500/1000 — directly reproduces the README scaling table.
- `bench_algo.py`: added `knapsack(100)`, `quicksort(1000)` to the default suite; existing small-input runs retained as the FFI overhead reference point.

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
