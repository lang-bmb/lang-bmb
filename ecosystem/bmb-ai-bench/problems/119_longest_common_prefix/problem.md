# Longest Common Prefix

## Description

Given n strings, find the longest common prefix shared by all strings.

**Input** (stdin):
- First line: n (number of strings)
- Next n lines: one string per line (lowercase letters)

**Output** (stdout):
- The longest common prefix (or empty line if none)

## Example

Input:
```
3
flower
flow
flight
```

Output:
```
fl
```

## Constraints

- 1 <= n <= 100
- 1 <= string length <= 1000

## Category

String

## BMB Notes
- Compare each string against the first string
- Track minimum prefix length seen so far
- Use `s.char_code_at(i)` for character comparison
- Use `svec_new()`, `svec_push(v, s)`, `svec_get(v, i)` for string vector
- Use `read_line()` to read each string

```
fn lcp_two(a: String, b: String, i: i64) -> i64 =
    if i >= a.len() or i >= b.len() { i }
    else if a.char_code_at(i) != b.char_code_at(i) { i }
    else { lcp_two(a, b, i + 1) };

fn find_lcp(strs: i64, n: i64, first: String, i: i64, plen: i64) -> i64 =
    if i >= n or plen == 0 { plen }
    else {
        let s = svec_get(strs, i);
        let min_plen = lcp_two(first, s, 0);
        find_lcp(strs, n, first, i + 1, if min_plen < plen { min_plen } else { plen })
    };
```
