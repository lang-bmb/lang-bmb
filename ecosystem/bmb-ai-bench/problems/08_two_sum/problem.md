# Two Sum

## Description

Given an array of integers and a target sum, find two distinct indices whose elements sum to the target. Use a brute-force O(n²) approach.

**Input** (stdin):
- First integer: `target`, the target sum
- Second integer: `n`, the number of elements (2 <= n <= 1000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the two 0-based indices (smaller index first), separated by a space
- A valid pair is guaranteed to exist

## Example

Input:
```
9 4 2 7 11 15
```

Output:
```
0 1
```

(arr[0] + arr[1] = 2 + 7 = 9)

## Constraints

- 2 <= n <= 1000
- Exactly one valid pair exists
- All values fit in a 64-bit signed integer

## Category

Algorithm (search)
