# 126. Intersection of Two Arrays II

LeetCode #350

## Problem

Given two integer arrays, return their intersection (including duplicates). Each element in the result appears as many times as it appears in both arrays. Output sorted.

## Input

- Line 1: n1 (size of first array)
- Next n1 lines: elements of first array
- Line n1+2: n2 (size of second array)
- Next n2 lines: elements of second array

## Output

Intersection elements, one per line, sorted ascending

## Examples

```
Input:
4
1
2
2
1
2
2
2
Output:
2
2
```

```
Input:
3
4
9
5
3
9
4
9
Output:
4
9
```

## BMB Notes

Count occurrences: for each element in arr1, include it if its occurrence index < its count in arr2.

```bmb
fn count_in_vec(v: i64, n: i64, target: i64, i: i64, acc: i64) -> i64
= if i >= n { acc }
    else if vec_get(v, i) == target { count_in_vec(v, n, target, i + 1, acc + 1) }
    else { count_in_vec(v, n, target, i + 1, acc) };
```
