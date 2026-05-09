# bmb-text

High-performance string processing powered by [BMB](https://github.com/iyulab/lang-bmb) — Node.js FFI bindings.

## Installation

```bash
npm install bmb-text koffi
```

Requires the `bmb_text` native shared library.

## Getting the native library

**Option A — Download from GitHub Releases (recommended):**
1. Go to [lang-bmb Releases](https://github.com/iyulab/lang-bmb/releases)
2. Download `bmb-libs-<your-platform>.zip` from the latest release
3. Place `bmb_text.dll` / `libbmb_text.so` / `libbmb_text.dylib` next to `index.js`

**Option B — Build from source:**
```bash
cd /path/to/lang-bmb
cargo build --release
./target/release/bmb build ecosystem/bmb-text/src/lib.bmb --shared -o ecosystem/bmb-text/bmb_text
```

## Functions

| Function | Description |
|----------|-------------|
| `kmp_search(text, pattern)` | KMP search, returns first match index or -1 |
| `str_find(s, pattern)` | Find first occurrence |
| `str_rfind(s, pattern)` | Find last occurrence |
| `str_count(s, pattern)` | Count occurrences |
| `str_contains(s, pattern)` | Check containment |
| `str_starts_with(s, prefix)` | Prefix test |
| `str_ends_with(s, suffix)` | Suffix test |
| `hamming_distance(a, b)` | Hamming distance |
| `str_compare(a, b)` | Lexicographic compare (-1/0/1) |
| `str_reverse(s)` | Reverse string |
| `to_upper(s)` | Uppercase |
| `to_lower(s)` | Lowercase |
| `trim(s)` | Trim whitespace |
| `str_replace(s, from, to)` | Replace first |
| `str_replace_all(s, from, to)` | Replace all |
| `repeat(s, n)` | Repeat n times |
| `word_count(s)` | Word count |
| `is_palindrome(s)` | Palindrome check |
| `str_len(s)` | Byte length |
| `token_count(s, sep)` | Token count |
| `find_byte(s, b)` | Find byte value |
| `count_byte(s, b)` | Count byte occurrences |

## License

MIT
