# bmb-text — Fast String Processing

KMP search, substring operations, case conversion, and text analysis compiled from [BMB](https://github.com/iyulab/lang-bmb).

## Installation

```bash
pip install bmb-text
```

## Quick Start

```python
import bmb_text

bmb_text.kmp_search("hello world", "world")          # 6
bmb_text.str_replace_all("abcabc", "abc", "X")       # "XX"
bmb_text.to_upper("hello")                            # "HELLO"
bmb_text.trim("  hello  ")                            # "hello"
bmb_text.is_palindrome("racecar")                     # True
bmb_text.word_count("hello world")                    # 2
bmb_text.repeat("ab", 3)                              # "ababab"
bmb_text.hamming_distance("karolin", "kathrin")       # 3
```

## Full API (23 functions)

### Search
| Function | Description |
|----------|-------------|
| `kmp_search(text, pattern)` | KMP substring search O(n+m), returns index or -1 |
| `str_find(haystack, needle)` | Find first occurrence |
| `str_rfind(haystack, needle)` | Find last occurrence |
| `str_count(haystack, needle)` | Count non-overlapping occurrences |
| `str_contains(haystack, needle)` | Contains check |
| `str_starts_with(s, prefix)` | Prefix check |
| `str_ends_with(s, suffix)` | Suffix check |
| `find_byte(s, byte)` | Find byte value in string |
| `count_byte(s, byte)` | Count byte occurrences |

### Transform
| Function | Description |
|----------|-------------|
| `str_reverse(s)` | Reverse string |
| `str_replace(s, old, new)` | Replace first occurrence |
| `str_replace_all(s, old, new)` | Replace all occurrences |
| `to_upper(s)` | Convert to UPPERCASE |
| `to_lower(s)` | Convert to lowercase |
| `trim(s)` | Trim leading/trailing whitespace |
| `repeat(s, n)` | Repeat string n times |

### Analysis
| Function | Description |
|----------|-------------|
| `is_palindrome(s)` | Palindrome check |
| `hamming_distance(a, b)` | Hamming distance between equal-length strings |
| `word_count(s)` | Count whitespace-separated words |
| `token_count(s, delimiter)` | Count delimiter-separated tokens |
| `str_len(s)` | String length in bytes |
| `str_char_at(s, idx)` | Byte value at index (-1 if OOB) |
| `str_compare(a, b)` | Lexicographic comparison |

## How?

Written in [BMB](https://github.com/iyulab/lang-bmb) — compile-time contracts prove correctness, then generate code faster than hand-tuned C.

## License

MIT
