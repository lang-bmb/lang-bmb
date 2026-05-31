# 138. Factorial Trailing Zeros

LeetCode #172

## Problem

Given `n`, return the number of trailing zeros in `n!`.

## Input

- One line: `n`

## Output

Number of trailing zeros

## Examples

```
Input: 5
Output: 1   (5! = 120)
```

```
Input: 25
Output: 6
```

## BMB Notes

Count factors of 5 (each contributes one trailing zero): sum of floor(n/5^k).

```bmb
fn trailing_zeros(n: i64, acc: i64) -> i64
= if n < 5 { acc }
    else { trailing_zeros(n / 5, acc + n / 5) };
```
