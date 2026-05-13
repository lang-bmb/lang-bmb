# Zigzag Print

Arrange n items into rows of w. Print odd rows left-to-right, even rows right-to-left.

## Input
- First integer: n
- Second integer: w (row width)
- Next n integers: items in order

## Output
Each row space-separated on its own line. Row 1 forward, row 2 backward, etc.

## Example
`6 3 1 2 3 4 5 6` -> row1 forward [1,2,3], row2 backward [6,5,4]
Output:
```
1 2 3
6 5 4
```
