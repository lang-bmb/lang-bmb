# bmb-crypto — Fast Cryptographic Functions

Standards-compliant hashing, encoding, and checksums compiled from [BMB](https://github.com/iyulab/lang-bmb).

## Installation

```bash
pip install bmb-crypto
```

## Quick Start

```python
import bmb_crypto

bmb_crypto.sha256("hello")
# '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'

bmb_crypto.md5("hello")
# '5d41402abc4b2a76b9719d911017c592'

bmb_crypto.hmac_sha256("key", "message")
# RFC 2104 compliant HMAC-SHA256

bmb_crypto.base64_encode("hello")   # 'aGVsbG8='
bmb_crypto.base64_decode("aGVsbG8=")  # 'hello'

bmb_crypto.crc32("hello")           # '3610a686'
bmb_crypto.adler32("Wikipedia")     # '11e60398'
```

## Full API (11 functions)

| Function | Standard | Output |
|----------|----------|--------|
| `sha256(data)` | FIPS 180-4 | 64-char hex |
| `md5(data)` | RFC 1321 | 32-char hex |
| `crc32(data)` | ISO 3309 | 8-char hex |
| `hmac_sha256(key, msg)` | RFC 2104 | 64-char hex |
| `base64_encode(data)` | RFC 4648 | Base64 string |
| `base64_decode(data)` | RFC 4648 | Original string |
| `base32_encode(data)` | RFC 4648 | Base32 string |
| `base32_decode(data)` | RFC 4648 | Original string |
| `adler32(data)` | RFC 1950 | 8-char hex |
| `fletcher16(data)` | Fletcher-16 | 4-char hex |
| `xor_checksum(data)` | XOR | 2-char hex |

All outputs cross-validated against Python's `hashlib`, `binascii`, `hmac`, and `base64`.

## How?

Written in [BMB](https://github.com/iyulab/lang-bmb) — compile-time contracts prove correctness, then generate code faster than hand-tuned C.

## License

MIT
