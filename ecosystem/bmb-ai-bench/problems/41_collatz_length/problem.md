# Collatz Length

## Description

Compute the length of the Collatz sequence starting from n until it reaches 1.

The Collatz sequence: if n is even, next = n/2; if n is odd, next = 3*n+1. Length includes the starting number and 1.

**Input** (stdin):
- First integer: `t`, the number of queries (1 <= t <= 100)
- For each query: a single integer `n` (1 <= n <= 1000000)

**Output** (stdout):
- For each query, print the Collatz sequence length on its own line

## Example

Input:
```
3 1 6 27
```

Output:
```
1
9
112
```

(Sequence from 6: 6,3,10,5,16,8,4,2,1 → length 9)

## Constraints

- 1 <= t <= 100
- 1 <= n <= 1000000

## Category

Algorithm (number theory)
