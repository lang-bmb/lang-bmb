# Changelog

All notable changes to bmb-compute will be documented in this file.

## [Unreleased]

### Added
- **Zero-copy inputs** for the read-only reductions (`sum`, `mean_scaled`, `min_val`, `max_val`,
  `range_val`, `variance_scaled`, `dot_product`, `dist_squared`, `weighted_sum`, `median_scaled`,
  `magnitude_squared`). The shared `_arr` helper now passes NumPy `int64` arrays and `array.array('q')`
  straight through with no per-element marshalling — ~500× faster than the list path for
  million-element inputs. Plain `list` inputs are unchanged; NumPy is optional. Read-only functions
  never mutate the caller's buffer.

### Performance
- The shared library is now built with clang's runtime loop unrolling disabled
  (`-mllvm -unroll-runtime=false`), a suite-wide change to the binding build pipeline. bmb-compute's
  kernels are reduction/vector loops whose vectorization (and interleaving) is preserved by the flag, so
  this is **performance-neutral here** — `-unroll-runtime=false` disables only the loop unroller, not the
  vectorizer (cf. bmb-algo's `array_sum`, measured 0.95× = parity under this flag). The change targets
  the loop-carried-dependency DP kernels in bmb-algo. Noted for release/reproducibility consistency.

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
