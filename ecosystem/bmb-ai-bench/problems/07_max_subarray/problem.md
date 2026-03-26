# Maximum Subarray Sum

## Description

Find the maximum sum of a contiguous subarray using Kadane's algorithm.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Next `n` integers: the array elements (may be negative)

**Output** (stdout):
- Print the maximum subarray sum

## Example

Input:
```
9 -2 1 -3 4 -1 2 1 -5 4
```

Output:
```
6
```

(The subarray [4, -1, 2, 1] has the maximum sum 6)

## Constraints

- 1 <= n <= 100000
- All values fit in a 64-bit signed integer
- Array may contain all negative numbers (answer is the largest single element)

## Category

Algorithm (dynamic programming)
