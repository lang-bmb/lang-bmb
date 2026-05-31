# 121. Contains Duplicate

LeetCode #217

## Problem

Given an integer array, return `true` if any value appears at least twice, and `false` if every element is distinct.

## Input

- First line: `n` (number of elements)
- Next `n` lines: one integer per line

## Output

`true` or `false`

## Examples

```
Input:
4
1
2
3
1

Output:
true
```

```
Input:
3
1
2
3

Output:
false
```

## BMB Notes

Read `n` integers into a vec, then check for duplicates with O(n²) nested scan.

```bmb
fn has_dup_from(v: i64, target: i64, i: i64, n: i64) -> bool
= if i >= n { false }
    else if vec_get(v, i) == target { true }
    else { has_dup_from(v, target, i + 1, n) };

fn has_dup(v: i64, i: i64, n: i64) -> bool
= if i >= n { false }
    else if has_dup_from(v, vec_get(v, i), i + 1, n) { true }
    else { has_dup(v, i + 1, n) };
```
