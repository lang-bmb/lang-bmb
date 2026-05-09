# bmb-crypto

Cryptographic functions powered by [BMB](https://github.com/iyulab/lang-bmb) — Node.js FFI bindings.

## Installation

```bash
npm install bmb-crypto koffi
```

Requires the `bmb_crypto` native shared library.

## Getting the native library

**Option A — Download from GitHub Releases (recommended):**
1. Go to [lang-bmb Releases](https://github.com/iyulab/lang-bmb/releases)
2. Download `bmb-libs-<your-platform>.zip` from the latest release
3. Place `bmb_crypto.dll` / `libbmb_crypto.so` / `libbmb_crypto.dylib` next to `index.js`

**Option B — Build from source:**
```bash
cd /path/to/lang-bmb
cargo build --release
./target/release/bmb build ecosystem/bmb-crypto/src/lib.bmb --shared -o ecosystem/bmb-crypto/bmb_crypto
```

## Functions

| Function | Description |
|----------|-------------|
| `sha256(s)` | SHA-256 hash (hex string) |
| `md5(s)` | MD5 hash (hex string) |
| `crc32(s)` | CRC-32 checksum (hex string) |
| `base64_encode(s)` | Base64 encode |
| `base64_decode(s)` | Base64 decode |
| `base32_encode(s)` | Base32 encode |
| `base32_decode(s)` | Base32 decode |
| `hmac_sha256(key, msg)` | HMAC-SHA256 (hex string) |
| `adler32(s)` | Adler-32 checksum (hex) |
| `fletcher16(s)` | Fletcher-16 checksum (hex) |
| `xor_checksum(s)` | XOR checksum (hex) |
| `rot13(s)` | ROT-13 transform |
| `hex_encode(s)` | Hex encode |
| `hex_decode(s)` | Hex decode |

## License

MIT
