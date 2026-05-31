# 140. First Unique Character in a String

LeetCode #387

## Problem

Given a string, return the index of the first non-repeating character. Return -1 if none exists.

## Input

- One line: the string

## Output

Index of first unique character, or -1

## Examples

```
Input: leetcode
Output: 0   ('l' appears once, at index 0)
```

```
Input: loveleetcode
Output: 2   ('v' appears once, at index 2)
```

```
Input: aabb
Output: -1
```

## BMB Notes

Count occurrences of each character, find first with count == 1.

```bmb
fn first_unique(s: String, i: i64) -> i64
= if i >= s.len() { 0 - 1 }
    else if count_char(s, s.char_code_at(i), 0, 0) == 1 { i }
    else { first_unique(s, i + 1) };
```
