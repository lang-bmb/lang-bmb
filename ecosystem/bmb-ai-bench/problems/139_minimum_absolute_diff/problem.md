# 139. Minimum Absolute Difference

LeetCode #1200

## Problem

Given an array of integers, find the minimum absolute difference between any two elements.

## Input

- Line 1: n
- Next n lines: one integer per line

## Output

Minimum absolute difference

## Examples

```
Input:
4
4
2
1
3
Output: 1
```

## BMB Notes

Sort the array, then find minimum difference between adjacent elements.

```bmb
let _sort = vec_sort(v);
// min of |v[i+1] - v[i]| for all i
```
