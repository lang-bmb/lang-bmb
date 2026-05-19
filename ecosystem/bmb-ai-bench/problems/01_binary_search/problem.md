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
- If duplicates exist, return the index found by the first mid-point comparison

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
- CRITICAL: When target found at mid, `set ans = mid` and `set lo = hi + 1` to EXIT immediately — do NOT change `hi` (changing hi searches leftmost/rightmost, wrong!)
- Use overflow-safe midpoint: `lo + (hi - lo) / 2`
- CRITICAL: Use `set` for ALL variable updates in BMB. `ans = mid` does NOT work — must be `set ans = mid`.
```
fn main() -> i64 = {
    let target: i64 = read_int();
    let n: i64 = read_int();
    let v = vec_new();
    for _i in 0..n {
        let val: i64 = read_int();
        let _p = vec_push(v, val)
    };
    let mut lo: i64 = 0;
    let mut hi: i64 = n - 1;
    let mut ans: i64 = -1;
    while lo <= hi {
        let mid: i64 = lo + (hi - lo) / 2;
        let val: i64 = vec_get(v, mid);
        if val == target {
            set ans = mid;
            set lo = hi + 1
        } else {
            if val < target { set lo = mid + 1 } else { set hi = mid - 1 }
        }
    };
    let _p = println(ans);
    0
};

```
