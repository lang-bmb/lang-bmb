# Matrix Multiplication

## Description

Multiply two square matrices of size n×n.

Matrices are stored as flat arrays in row-major order.

**Input** (stdin):
- First integer: `n`, the matrix dimension (1 <= n <= 50)
- Next n*n integers: matrix A elements (row by row)
- Next n*n integers: matrix B elements (row by row)

**Output** (stdout):
- Print the n*n elements of the result matrix C = A × B
- Print row by row, elements separated by spaces, each row on a new line

## Example

Input:
```
2 1 2 3 4 5 6 7 8
```

Output:
```
19 22
43 50
```

## Constraints

- 1 <= n <= 50
- All values fit in a 64-bit signed integer (no overflow in n <= 50)

## Category

Algorithm (linear algebra)
