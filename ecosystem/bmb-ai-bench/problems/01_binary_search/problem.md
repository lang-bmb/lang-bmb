# Binary Search

## Description

Implement binary search on a sorted array of integers.

**Input** (stdin):
- First integer: the target value to search for
- Second integer: `n`, the number of elements in the array (0 <= n <= 100000)
- Next `n` integers: the sorted array elements (in non-decreasing order)

**Output** (stdout):
- Print the 0-based index of the target if found
- Print `-1` if not found
- If duplicates exist, return any valid index

## Example

Input:
```
3 5 1 2 3 4 5
```

Output:
```
2
```

## Constraints

- Array is guaranteed to be sorted in non-decreasing order
- All values fit in a 64-bit signed integer
- 0 <= n <= 100000

## Category

Algorithm (search)

## BMB Notes
- Read target first, then n, then the array
- Standard lo/hi/mid; return mid when found, -1 if lo > hi
```
let target: i64 = read_int();
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
let mut lo: i64 = 0; let mut hi: i64 = n - 1; let mut ans: i64 = -1;
while lo <= hi {
    let mid: i64 = (lo + hi) / 2;
    if vec_get(v, mid) == target { lo = hi + 1; ans = mid }
    else { if vec_get(v, mid) < target { lo = mid + 1 } else { hi = mid - 1 } }
};
println(ans);
0
