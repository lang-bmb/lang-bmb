# Verified Binary Search

Binary search in a sorted array. Return 0-indexed position or -1 if not found.

## Input
- First integer: n
- Next n integers: sorted array (ascending)
- Next integer: q
- Next q integers: query targets

## Output
Position (0-indexed) or -1 for each query (one per line)

## Example
Array [1,2,3,4,5]: search(3)->2, search(6)->-1, search(1)->0

## BMB Notes
```
let n: i64 = read_int();
let arr = vec_new();
for _i in 0..n { vec_push(arr, read_int()) };
let q: i64 = read_int();
for _qi in 0..q {
    let target: i64 = read_int();
    let mut lo: i64 = 0; let mut hi: i64 = n - 1; let mut pos: i64 = -1;
    while lo <= hi {
        let mid: i64 = (lo + hi) / 2;
        if vec_get(arr, mid) == target { pos = mid; lo = hi + 1 }
        else if vec_get(arr, mid) < target { lo = mid + 1 }
        else { hi = mid - 1 }
    };
    println(pos)
};
0
```
