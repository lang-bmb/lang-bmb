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

## BMB Notes
- Track current run length and best; when value changes, reset current to 1
- Read all into vec first, then scan; handle n=1 edge case (best=1)
```
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
let mut best: i64 = 1; let mut cur: i64 = 1; let mut i: i64 = 1;
while i < n {
    if vec_get(v, i) == vec_get(v, i - 1) {
        set cur = cur + 1;
        if cur > best { set best = cur } else { () }
    } else { set cur = 1 };
    set i = i + 1
};
println(best);
0
