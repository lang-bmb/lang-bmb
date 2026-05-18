# Euclidean Distance Squared

## Description

Compute the squared Euclidean distance between two n-dimensional integer vectors.

**Input** (stdin):
- First integer: `n`, the vector dimension (1 <= n <= 100000)
- Next `n` integers: elements of vector A
- Next `n` integers: elements of vector B

**Output** (stdout):
- Print the squared Euclidean distance: sum of (a[i] - b[i])^2 for i in [0..n)

## Example

Input:
```
3 1 2 3 4 5 6
```

Output:
```
27
```

((1-4)^2 + (2-5)^2 + (3-6)^2 = 9 + 9 + 9 = 27)

## Constraints

- 1 <= n <= 100000
- All values fit in a 64-bit signed integer
- The result fits in a 64-bit signed integer

## Category

Algorithm (vector arithmetic)
