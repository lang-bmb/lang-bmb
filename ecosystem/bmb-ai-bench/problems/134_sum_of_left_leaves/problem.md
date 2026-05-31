# 134. Arrange Coins

LeetCode #441

## Problem

Given `n` coins, arrange them in rows (row k has k coins). Return the number of complete rows.

## Input

- One line: the integer `n`

## Output

Maximum complete rows

## Examples

```
Input: 5
Output: 2   (row1=1, row2=2, row3=2 incomplete)
```

```
Input: 8
Output: 3   (1+2+3=6, row4=2 incomplete)
```

## BMB Notes

Binary search: find largest k where k*(k+1)/2 <= n.

```bmb
fn max_rows(n: i64, lo: i64, hi: i64) -> i64
= if lo > hi { lo - 1 }
    else {
        let mid = lo + (hi - lo) / 2;
        let sum = mid * (mid + 1) / 2;
        if sum == n { mid }
        else if sum < n { max_rows(n, mid + 1, hi) }
        else { max_rows(n, lo, mid - 1) }
    };
```
