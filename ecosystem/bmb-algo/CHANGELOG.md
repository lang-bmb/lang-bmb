# Changelog

All notable changes to bmb-algo will be documented in this file.

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
