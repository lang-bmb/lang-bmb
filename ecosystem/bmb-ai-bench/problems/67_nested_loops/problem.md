# Nested Loops

## Description

Count the number of triplets (i, j, k) such that `i + j + k = target`, where each of i, j, k is in the range `[0, n)`.

**Input** (stdin):
- First integer: `t`, number of test cases
- For each test case: two integers `n` and `target` (space-separated on same line)

**Output** (stdout):
- For each test case, print the count of valid triplets on its own line

## Example

Input:
```
2 2 1 3 2
```

Output:
```
3
6
```

Explanation:
- n=2, target=1: i,j,k ∈ {0,1}. Triplets summing to 1: (0,0,1),(0,1,0),(1,0,0) = 3
- n=3, target=2: i,j,k ∈ {0,1,2}. Triplets summing to 2: (0,0,2),(0,1,1),(0,2,0),(1,0,1),(1,1,0),(2,0,0) = 6

## Additional Examples

- n=3, target=3 → 7 triplets: (0,1,2),(0,2,1),(1,0,2),(1,1,1),(1,2,0),(2,0,1),(2,1,0)
- n=3, target=6 → 1 triplet: (2,2,2)
- n=3, target=0 → 1 triplet: (0,0,0)

## Constraints

- 1 ≤ t ≤ 50
- 1 ≤ n ≤ 100
- 0 ≤ target ≤ 3*(n-1)
- All values fit in a 64-bit signed integer

## Category

Algorithm / nested loops / counting
