# bmb-text — Fast String Processing

KMP search, substring operations, case conversion, and more — compiled from [BMB](https://github.com/iyulab/lang-bmb).

## Functions (20)

### Search
- `kmp_search(text, pattern)` — KMP substring search O(n+m)
- `str_find(haystack, needle)` — Find first occurrence
- `str_rfind(haystack, needle)` — Find last occurrence
- `str_count(haystack, needle)` — Count non-overlapping occurrences
- `str_contains(haystack, needle)` — Contains check
- `str_starts_with(s, prefix)` — Prefix check
- `str_ends_with(s, suffix)` — Suffix check
- `find_byte(s, byte)` — Find byte in string
- `count_byte(s, byte)` — Count byte occurrences

### Transform
- `str_reverse(s)` — Reverse string
- `str_replace(s, old, new)` — Replace first occurrence
- `str_replace_all(s, old, new)` — Replace all occurrences
- `to_upper(s)` — Convert to UPPERCASE
- `to_lower(s)` — Convert to lowercase
- `trim(s)` — Trim whitespace
- `repeat(s, n)` — Repeat string n times

### Analysis
- `is_palindrome(s)` — Palindrome check
- `hamming_distance(a, b)` — Hamming distance
- `word_count(s)` — Count words
- `token_count(s, delimiter)` — Count tokens

## Quick Start

```python
import bmb_text

bmb_text.kmp_search("hello world", "world")  # 6
bmb_text.str_replace_all("abcabc", "abc", "X")  # "XX"
bmb_text.to_upper("hello")  # "HELLO"
bmb_text.trim("  hello  ")  # "hello"
bmb_text.is_palindrome("racecar")  # True
bmb_text.word_count("hello world")  # 2
```

Powered by [BMB](https://github.com/iyulab/lang-bmb) — *Performance > Everything*.
