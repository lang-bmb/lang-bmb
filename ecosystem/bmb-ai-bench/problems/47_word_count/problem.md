# Longest Consecutive Run

## Description

Given an array of integers, find the length of the longest consecutive run of equal values.

A "run" is a maximal sequence of adjacent equal elements.

**Input** (stdin):
- First integer: `n`, number of elements
- Next `n` integers: the array elements (space-separated)

**Output** (stdout):
- Print the length of the longest consecutive run of equal values

## Example

Input:
```
7 1 1 2 2 2 1 1
```

Output:
```
3
```

Explanation: The runs are [1,1], [2,2,2], [1,1] with lengths 2, 3, 2. The longest is 3.

## Additional Examples

- `5 1 1 2 2 2` → 3 (the run [2,2,2])
- `5 1 2 3 4 5` → 1 (all elements different)
- `4 1 1 1 1` → 4 (one run of all elements)
- `6 1 2 1 2 1 2` → 1 (alternating, max run = 1)
- `6 1 2 2 3 3 3` → 3 (the run [3,3,3])

## Constraints

- 1 ≤ n ≤ 10000
- -10^9 ≤ array[i] ≤ 10^9
- All values fit in a 64-bit signed integer

## Category

Array / sequential scan
