# Longest Plateau

## Description

Find the length of the longest consecutive run of equal values in an array.

A "plateau" is a maximal consecutive subsequence of identical values. Find the longest such plateau.

**Input** (stdin):
- First integer: `t`, number of test cases
- For each test case:
  - Integer `n`: array length
  - Next `n` integers: array elements

**Output** (stdout):
- For each test case, print the length of the longest plateau on its own line

## Example

Input:
```
2 5 1 1 2 2 2 6 1 2 3 4 5 6
```

Output:
```
3
1
```

Explanation:
- Test 1: [1,1,2,2,2] — runs of 2 (length 2) and 2,2,2 (length 3) → longest=3
- Test 2: [1,2,3,4,5,6] — all different, each run has length 1 → longest=1

## Constraints

- 1 ≤ t ≤ 100
- 1 ≤ n ≤ 100,000
- -1,000,000,000 ≤ each element ≤ 1,000,000,000
- A single element array has plateau length 1

## Algorithm

Initialize `max_len = 1`, `cur_len = 1`. For i from 1 to n-1:
- If `a[i] == a[i-1]`: increment `cur_len`, update `max_len` if larger
- Else: reset `cur_len = 1`

## Category

Arrays / scanning
