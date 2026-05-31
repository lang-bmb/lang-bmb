# 135. Is Ugly Number

LeetCode #263

## Problem

An ugly number is a positive integer whose prime factors are limited to 2, 3, and 5. Determine whether `n` is an ugly number.

## Input

- One line: the integer `n`

## Output

`true` or `false`

## Examples

```
Input: 6
Output: true   (6 = 2 × 3)
```

```
Input: 14
Output: false  (14 = 2 × 7, 7 is not allowed)
```

## BMB Notes

Repeatedly divide by 2, 3, 5. If the result is 1, it's ugly.

```bmb
fn remove_factor(n: i64, f: i64) -> i64
= if n % f != 0 { n }
    else { remove_factor(n / f, f) };
```
