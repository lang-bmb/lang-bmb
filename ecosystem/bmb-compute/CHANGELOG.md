# Changelog

All notable changes to bmb-compute will be documented in this file.

## [0.2.0] - 2026-03-23

### Added
- 8 new functions: median_scaled, cumsum, moving_avg_scaled, magnitude_squared, vec_add, vec_sub, vec_scale, map_square
- Comprehensive docstrings for all 33 functions
- C header file generation (include/bmb_compute.h)
- WASM output support (62 KB WAT)
- `__all__` for clean import * behavior

### Changed
- Total functions: 25 -> 33

## [0.1.0] - 2026-03-23

### Added
- 25 functions: Math (8), Statistics (5), Random (4), Vector (4), Utility (4)
- Python ctypes bindings with FFI safety
- Type stubs (.pyi) for IDE autocomplete
- pytest test suite (270 tests)
- Benchmark script
- pyproject.toml (PEP 621 compliant)
