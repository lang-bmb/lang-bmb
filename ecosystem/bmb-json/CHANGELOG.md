# Changelog

All notable changes to bmb-json will be documented in this file.

## [0.2.0] - 2026-03-23

### Added
- 4 new functions: has_key, object_len, get_bool, count
- C header file generation (include/bmb_json.h)
- WASM output support (104 KB WAT) with experimental bindings/wasm/
- `__all__` for clean import * behavior

### Changed
- Total functions: 8 -> 12

## [0.1.0] - 2026-03-23

### Added
- 8 functions: validate, stringify, get_type, get, get_string, get_number, array_len, array_get
- Zero-copy JSON parser
- Python ctypes bindings with FFI safety
- Type stubs (.pyi) for IDE autocomplete
- pytest test suite (159 tests) cross-validated against Python json stdlib
- Benchmark script
- pyproject.toml (PEP 621 compliant)
