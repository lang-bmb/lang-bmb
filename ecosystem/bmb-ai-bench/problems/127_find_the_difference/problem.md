# 127. Number of Steps to Reduce to Zero

LeetCode #1342

## Problem

Given a non-negative integer `n`, return the number of steps to reduce it to zero. Each step: if even, divide by 2; if odd, subtract 1.

## Input

- One line: the integer `n`

## Output

Number of steps

## Examples

```
Input: 14
Output: 6
(14→7→6→3→2→1→0)
```

```
Input: 8
Output: 4
(8→4→2→1→0)
```

## BMB Notes

Simple recursion:

```bmb
fn steps(n: i64, count: i64) -> i64
= if n == 0 { count }
    else if n % 2 == 0 { steps(n / 2, count + 1) }
    else { steps(n - 1, count + 1) };
```
