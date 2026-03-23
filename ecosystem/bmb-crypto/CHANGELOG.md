# Changelog

All notable changes to bmb-crypto will be documented in this file.

## [0.3.0] - 2026-03-23

### Added
- 3 new functions: rot13, hex_encode, hex_decode
- C header file generation (include/bmb_crypto.h)
- WASM output support (289 KB WAT)
- `__all__` for clean import * behavior

### Changed
- Total functions: 11 -> 14

## [0.2.0] - 2026-03-23

### Added
- 11 functions: SHA-256, MD5, CRC32, HMAC-SHA256, Base64/32 encode/decode, Adler-32, Fletcher-16, XOR checksum
- Python ctypes bindings with FFI safety
- Type stubs (.pyi) for IDE autocomplete
- pytest test suite (212 tests) cross-validated against Python stdlib
- Benchmark script
- pyproject.toml (PEP 621 compliant)

## [0.1.0] - 2026-03-15

### Added
- Initial release: SHA-256, MD5, Base64, CRC32
