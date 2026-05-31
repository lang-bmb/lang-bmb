# 130. Palindrome Number

LeetCode #9

## Problem

Given an integer, return `true` if it is a palindrome (reads the same forwards and backwards). Negative numbers are not palindromes.

## Input

- One line: the integer

## Output

`true` or `false`

## Examples

```
Input: 121
Output: true
```

```
Input: -121
Output: false
```

```
Input: 10
Output: false
```

## BMB Notes

Reverse the number and compare:

```bmb
fn reverse_num(n: i64, acc: i64) -> i64
= if n == 0 { acc }
    else { reverse_num(n / 10, acc * 10 + n % 10) };
```

Negative numbers are always false.
