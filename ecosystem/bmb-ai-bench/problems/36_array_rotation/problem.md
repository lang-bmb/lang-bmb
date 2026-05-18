# Array Rotation

## Description

Rotate an array left by k positions.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Second integer: `k`, the number of left rotation steps (0 <= k < n)
- Next `n` integers: the elements

**Output** (stdout):
- Print the rotated array elements separated by spaces, followed by a newline

## Example

Input:
```
5 2 1 2 3 4 5
```

Output:
```
3 4 5 1 2
```

(Rotate left by 2: [1,2,3,4,5] → [3,4,5,1,2])

## Constraints

- 1 <= n <= 100000
- 0 <= k < n
- All values fit in a 64-bit signed integer

## Category

Algorithm (array manipulation)
