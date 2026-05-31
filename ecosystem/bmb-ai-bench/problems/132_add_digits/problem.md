# 132. Add Digits

LeetCode #258

## Problem

Given an integer, repeatedly add its digits until the result has only one digit. Return that single digit.

## Input

- One line: the integer

## Output

The digital root

## Examples

```
Input: 38
Output: 2   (3+8=11, 1+1=2)
```

```
Input: 0
Output: 0
```

## BMB Notes

Mathematical shortcut (digital root formula):
- If n == 0: return 0
- Otherwise: return 1 + (n - 1) % 9

```bmb
if n == 0 { println(0) }
else { println(1 + (n - 1) % 9) };
```
