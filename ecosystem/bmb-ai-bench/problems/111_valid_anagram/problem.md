# Valid Anagram

## Description

Given two strings `s` and `t`, return `"true"` if `t` is an anagram of `s`, `"false"` otherwise.

An anagram uses all the characters of the original string exactly once (in any order).

**Input** (stdin):
- First line: string `s` (lowercase letters only, length 1-10000)
- Second line: string `t` (lowercase letters only)

**Output** (stdout):
- `"true"` if anagram, `"false"` otherwise

## Example

Input:
```
anagram
nagaram
```

Output:
```
true
```

## Constraints

- 1 <= s.len(), t.len() <= 10000
- s and t consist of lowercase English letters only

## Category

String (frequency count)

## BMB Notes
- Compare character frequencies for all 26 letters
- `char_code_at(s, i)` returns ASCII code of s[i]; 'a' = 97
- Count each character in both strings, check if counts match

```
fn count_char(s: String, c: i64, i: i64, acc: i64) -> i64 =
    if i >= s.len() { acc }
    else if s.char_code_at(i) == c { count_char(s, c, i + 1, acc + 1) }
    else { count_char(s, c, i + 1, acc) };

fn anagram_check(s: String, t: String, i: i64) -> bool =
    if i >= 26 { true }
    else if count_char(s, 97 + i, 0, 0) != count_char(t, 97 + i, 0, 0) { false }
    else { anagram_check(s, t, i + 1) };

fn main() -> i64 = {
    let s = read_line();
    let t = read_line();
    if s.len() != t.len() { println_str("false") }
    else if anagram_check(s, t, 0) { println_str("true") }
    else { println_str("false") };
    0
};
```
