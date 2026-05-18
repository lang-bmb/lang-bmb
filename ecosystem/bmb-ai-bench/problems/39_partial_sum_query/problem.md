# Partial Sum Query

## Description

Answer range sum queries on an integer array using prefix sums.

**Input** (stdin):
- First integer: `n`, the array length (1 <= n <= 100000)
- Next `n` integers: the array elements
- Next integer: `q`, the number of queries (1 <= q <= 100000)
- For each query: two integers `l r` (0-indexed, inclusive range [l..r])

**Output** (stdout):
- For each query, print the sum of elements in the range [l..r] (inclusive) on its own line

## Example

Input:
```
5 1 2 3 4 5 3 0 4 1 3 2 2
```

Output:
```
15
9
3
```

(Array [1,2,3,4,5]: sum[0..4]=15, sum[1..3]=2+3+4=9, sum[2..2]=3)

## Constraints

- 1 <= n, q <= 100000
- 0 <= l <= r < n
- All values fit in a 64-bit signed integer

## Category

Algorithm (prefix sums)
