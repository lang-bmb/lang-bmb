# 136. Sum of Unique Elements

LeetCode #1748

## Problem

Given an integer array, return the sum of all unique elements (elements that appear exactly once).

## Input

- Line 1: n (number of elements)
- Next n lines: one integer per line

## Output

Sum of unique elements

## Examples

```
Input:
4
1
2
3
2
Output: 4   (1 and 3 are unique: 1+3=4)
```

## BMB Notes

Count occurrences of each element, include in sum only if count == 1.
