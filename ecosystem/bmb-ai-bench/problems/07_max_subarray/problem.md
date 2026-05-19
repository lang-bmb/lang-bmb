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

## BMB Notes
- Initialize with first element (handles all-negative case)
- Kadane's: `cur = max(v, cur + v)` at each step; `best = max(best, cur)`
```
let n: i64 = read_int();
let first: i64 = read_int();
let mut cur: i64 = first;
let mut best: i64 = first;
for _i in 1..n {
    let v: i64 = read_int();
    cur = max(v, cur + v);
    best = max(best, cur)
};
println(best);
0
```
