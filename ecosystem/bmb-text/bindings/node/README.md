# bmb-text

High-performance string processing powered by [BMB](https://github.com/iyulab/lang-bmb) — Node.js FFI bindings.

## Installation

```bash
npm install bmb-text koffi
```

Requires the `bmb_text` native shared library (`.dll`/`.so`/`.dylib`) to be present.
See the [main README](../../README.md) for build instructions.

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
