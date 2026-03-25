# Sorted Insert

## Description

Insert a value into a sorted array while maintaining sorted order. Use contracts to verify the result remains sorted.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 10000)
- Next `n` integers: a sorted array (non-decreasing)
- Last integer: `val`, the value to insert

**Output** (stdout):
- Print the resulting sorted array, space-separated

## Contract Requirement

The insertion function should have contracts verifying:
```
pre the input array is sorted (at the insertion point)
post the output contains n+1 elements
```

## Example

Input:
```
5 1 3 5 7 9 4
```

Output:
```
1 3 4 5 7 9
```

## Constraints

- 0 <= n <= 10000
- Input array is guaranteed sorted
- All values fit in a 64-bit signed integer

## Category

Contract (sorted invariant)
