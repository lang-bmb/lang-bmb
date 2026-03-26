# Matrix Transpose

## Description

Transpose a matrix (rows become columns, columns become rows).

**Input** (stdin):
- First two integers: `rows cols` (dimensions)
- Next `rows * cols` integers: matrix elements in row-major order

**Output** (stdout):
- Print the transposed matrix (cols rows), row by row
- Elements separated by spaces, one row per line

## Example

Input:
```
2 3 1 2 3 4 5 6
```

Output:
```
1 4
2 5
3 6
```

## Constraints

- 1 <= rows, cols <= 100
- All values fit in a 64-bit signed integer

## Category

System (matrix operation)
