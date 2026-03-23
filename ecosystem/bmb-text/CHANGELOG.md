# Changelog

All notable changes to bmb-text will be documented in this file.

## [0.2.0] - 2026-03-23

### Added
- 3 new functions: str_len, str_char_at, str_compare
- C header file generation (include/bmb_text.h)
- WASM output support (81 KB WAT)
- `__all__` for clean import * behavior

### Changed
- Total functions: 20 -> 23

## [0.1.0] - 2026-03-23

### Added
- 20 functions: Search (9), Transform (7), Analysis (4)
- KMP O(n+m) substring search
- Python ctypes bindings with FFI safety
- Type stubs (.pyi) for IDE autocomplete
- pytest test suite (127 tests)
- Benchmark script
- pyproject.toml (PEP 621 compliant)
