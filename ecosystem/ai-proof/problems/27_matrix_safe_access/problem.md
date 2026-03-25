# Matrix Safe Access

## Description

Access elements of a 2D matrix with row and column bounds contracts.

**Input** (stdin):
- First two integers: `rows cols`
- Next `rows * cols` integers: matrix elements (row-major)
- Next integer: `q`, number of queries
- For each query: two integers `r c` (0-indexed)

**Output** (stdout):
- For each query, print the element at (r, c)

## Contract Requirement

```
pre r >= 0 and r < rows and c >= 0 and c < cols
```

## Example

Input:
```
3 3 1 2 3 4 5 6 7 8 9 3 0 0 1 1 2 2
```

Output:
```
1
5
9
```

## Constraints

- 1 <= rows, cols <= 100
- 0 <= r < rows, 0 <= c < cols (guaranteed by test data)
- 1 <= q <= 1000

## Category

Contract (2D bounds verification)
