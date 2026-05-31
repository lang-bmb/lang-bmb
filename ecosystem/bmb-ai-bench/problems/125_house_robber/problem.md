# 125. House Robber

LeetCode #198

## Problem

Given an array of non-negative integers representing the amount of money in each house, find the maximum amount you can rob without robbing two adjacent houses.

## Input

- First line: `n` (number of houses)
- Next `n` lines: one integer per line (money in each house)

## Output

The maximum amount of money

## Examples

```
Input:
3
1
2
3
Output: 4   (rob house 1 and 3: 1+3=4)
```

```
Input:
4
2
7
9
3
Output: 11  (rob house 1 and 3: 2+9=11)
```

## BMB Notes

DP with two variables tracking the best solution up to i-2 and i-1:

```bmb
fn rob(v: i64, n: i64, i: i64, prev2: i64, prev1: i64) -> i64
= if i >= n { prev1 }
    else {
        let cur = vec_get(v, i);
        let with_cur = prev2 + cur;
        let new_val = if with_cur > prev1 { with_cur } else { prev1 };
        rob(v, n, i + 1, prev1, new_val)
    };
```
