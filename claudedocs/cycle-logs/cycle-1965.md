# Cycle 1965-1966: bmb-crypto MD5 + CRC32 + Base32 expansion
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1964: All carry-forward items resolved

## Scope & Implementation

### bmb-crypto expansion: 3 new crypto functions
- **MD5** (RFC 1321): Full 32-char hex digest, proper little-endian register output
  - Adapted from gotgan-packages bmb-md5, rewrote for string-based API
  - Uses existing `aput/aget` helpers, added `h32_le()` for little-endian hex
  - Verified against Python `hashlib.md5` for 4 test vectors including empty/long strings
- **CRC32** (ISO 3309): 8-char hex string output
  - Polynomial 0xEDB88320 (reflected form)
  - Verified against Python `binascii.crc32` for 4 test vectors
- **Base32** (RFC 4648): Encode/decode with proper padding
  - Direct string-based API using `sb_new/sb_push/sb_build`
  - RFC 4648 padding rules: 1→6pad, 2→4pad, 3→3pad, 4→1pad, 5→0pad
  - **Bug found & fixed**: 4-byte input padding (c6 needs `remaining >= 4` not `>= 5`)
  - Verified against RFC 4648 test vectors for all padding cases

### Python binding expansion
- Added `md5()`, `crc32()`, `base32_encode()`, `base32_decode()` to bmb_crypto.py
- Added Windows MSYS2 DLL directory fix (`os.add_dll_directory`)
- Comprehensive test suite: 34 tests vs Python stdlib (hashlib, binascii, base64)
- Same DLL directory fix applied to bmb-algo binding

### Files changed
- `ecosystem/bmb-crypto/src/lib.bmb`: +300 lines (MD5, CRC32, Base32 implementations)
- `ecosystem/bmb-crypto/bindings/python/bmb_crypto.py`: Complete rewrite with 7 functions + 34 tests
- `ecosystem/bmb-algo/bindings/python/bmb_algo.py`: DLL directory fix

## Review & Resolution
- cargo test --release: 6,186 pass ✅
- bmb-crypto standalone: 7/7 functions work ✅
- bmb-crypto Python: 34/34 tests PASS ✅
  - SHA-256: 3/3, MD5: 4/4, CRC32: 4/4, Base64: 11/11, Base32: 12/12
- Base32 4-byte padding bug: Found and fixed in same cycle

### bmb-crypto total functions: 7
| Function | Standard | Output |
|----------|----------|--------|
| sha256 | FIPS 180-4 | 64-char hex |
| md5 | RFC 1321 | 32-char hex |
| crc32 | ISO 3309 | 8-char hex |
| base64_encode | RFC 4648 | Base64 string |
| base64_decode | RFC 4648 | Original string |
| base32_encode | RFC 4648 | Base32 string |
| base32_decode | RFC 4648 | Original string |

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: HMAC-SHA256 + crypto benchmarks vs hashlib
