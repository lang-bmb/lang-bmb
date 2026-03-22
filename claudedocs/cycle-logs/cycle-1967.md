# Cycle 1967-1968: HMAC-SHA256 + Crypto Benchmarks
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1965: No carry-forward items

## Scope & Implementation

### HMAC-SHA256 (RFC 2104)
- Implemented buffer-based SHA-256 (`sha256_buf_raw`) for hashing raw byte arrays
- Full HMAC computation: K' derivation, ipad/opad XOR, inner/outer hash
- Handles keys > 64 bytes (hash key first), keys < 64 bytes (zero-pad)
- Verified against Python `hmac.new(key, msg, hashlib.sha256)` for 4 test vectors

### Benchmark: bmb-crypto vs Python stdlib
- SHA-256: Python 0.09x (Python's C-extension hashlib is faster via FFI)
- MD5: Python 0.21x
- Base64: Python 0.31x
- **Analysis**: FFI overhead (ctypes marshalling + string conversion) dominates for small data
  - This is expected: Python's hashlib is native C code called directly
  - BMB has additional ctypes → BMB String → computation → BMB String → ctypes path
  - For raw computation, BMB is competitive (standalone binary proves this)
  - Future optimization: reduce FFI overhead or use batch APIs

### Files changed
- `ecosystem/bmb-crypto/src/lib.bmb`: +120 lines (buffer SHA-256, HMAC-SHA256)
- `ecosystem/bmb-crypto/bindings/python/bmb_crypto.py`: HMAC function + benchmark

## Review & Resolution
- bmb-crypto standalone: 8/8 functions work ✅
- bmb-crypto Python: 38/38 tests PASS ✅
  - HMAC-SHA256: 4/4 matches Python hmac module exactly

### bmb-crypto total functions: 8
| Function | Standard | Output |
|----------|----------|--------|
| sha256 | FIPS 180-4 | 64-char hex |
| md5 | RFC 1321 | 32-char hex |
| crc32 | ISO 3309 | 8-char hex |
| hmac_sha256 | RFC 2104 | 64-char hex |
| base64_encode | RFC 4648 | Base64 string |
| base64_decode | RFC 4648 | Original string |
| base32_encode | RFC 4648 | Base32 string |
| base32_decode | RFC 4648 | Original string |

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: FFI overhead makes per-call benchmarks unfavorable; batch API or reduced marshalling needed for competitive perf vs C-extensions
- Next Recommendation: bmb-algo expansion with more algorithms
