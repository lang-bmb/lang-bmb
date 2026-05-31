# 123. Excel Sheet Column Number

LeetCode #171

## Problem

Given a column title (e.g., "A", "B", "AA"), return its corresponding column number. A→1, B→2, ..., Z→26, AA→27, AB→28, ...

## Input

- One line: the column label (uppercase letters)

## Output

The column number as an integer

## Examples

```
Input: A
Output: 1
```

```
Input: AA
Output: 27
```

```
Input: ZY
Output: 701
```

## BMB Notes

Convert using base-26 positional notation. Each character `C` contributes `(ord(C) - 64)` to the value.

```bmb
fn col_num(s: String, i: i64, acc: i64) -> i64
= if i >= s.len() { acc }
    else { col_num(s, i + 1, acc * 26 + s.char_code_at(i) - 64) };
```
