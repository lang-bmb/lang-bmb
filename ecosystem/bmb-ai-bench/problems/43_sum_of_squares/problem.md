# Sum Of Squares

## Description

Compute the sum of squares 1^2 + 2^2 + ... + n^2 for multiple queries.

**Input** (stdin):
- First integer: `t`, the number of queries (1 <= t <= 100)
- For each query: a single integer `n` (1 <= n <= 10000)

**Output** (stdout):
- For each query, print 1^2 + 2^2 + ... + n^2 on its own line

## Example

Input:
```
3 1 3 5
```

Output:
```
1
14
55
```

(1=1; 1+4+9=14; 1+4+9+16+25=55)

## Constraints

- 1 <= t <= 100
- 1 <= n <= 10000
- Result fits in a 64-bit signed integer

## Category

Algorithm (arithmetic)
