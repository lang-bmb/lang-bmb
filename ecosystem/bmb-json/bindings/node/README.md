# bmb-json

High-performance JSON processing powered by [BMB](https://github.com/iyulab/lang-bmb) — Node.js FFI bindings.

## Installation

```bash
npm install bmb-json koffi
```

Requires the `bmb_json` native shared library.

## Getting the native library

**Option A — Download from GitHub Releases (recommended):**
1. Go to [lang-bmb Releases](https://github.com/iyulab/lang-bmb/releases)
2. Download `bmb-libs-<your-platform>.zip` from the latest release
3. Place `bmb_json.dll` / `libbmb_json.so` / `libbmb_json.dylib` next to `index.js`

**Option B — Build from source:**
```bash
cd /path/to/lang-bmb
cargo build --release
./target/release/bmb build ecosystem/bmb-json/src/lib.bmb --shared -o ecosystem/bmb-json/bmb_json
```

> **Note**: All functions accept JSON as a string. Output strings are library-owned — do not hold references after the function returns.

## Functions

| Function | Description |
|----------|-------------|
| `validate(json)` | True if JSON is valid |
| `get_type(json)` | Top-level type: `"object"`, `"array"`, `"string"`, `"number"`, `"bool"`, `"null"` |
| `stringify(json)` | Compact (minified) JSON string |
| `array_len(json)` | Array element count |
| `object_len(json)` | Object key count |
| `count(json)` | Total value count (recursive) |
| `get_number(json, key)` | Number at key |
| `has_key(json, key)` | True if key exists |
| `get_bool(json, key)` | Boolean at key (0 or 1) |
| `get(json, key)` | Raw JSON at key |
| `get_string(json, key)` | String value at key |
| `array_get(json, idx)` | Array element at index as raw JSON |

## License

MIT
