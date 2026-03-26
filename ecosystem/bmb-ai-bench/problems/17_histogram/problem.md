# Histogram

## Description

Given an array of integers in range [0, k-1], count the frequency of each value and print the histogram.

**Input** (stdin):
- First integer: `k`, the number of buckets (1 <= k <= 100)
- Second integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the elements (each in [0, k-1])

**Output** (stdout):
- Print `k` lines, one per bucket
- Each line: `i count` (bucket index and its count)

## Example

Input:
```
3 7 0 1 2 0 1 0 2
```

Output:
```
0 3
1 2
2 2
```

## Constraints

- 1 <= k <= 100
- 0 <= n <= 100000
- All elements in [0, k-1]

## Category

System (counting/aggregation)
