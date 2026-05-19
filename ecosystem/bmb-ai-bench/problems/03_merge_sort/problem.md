# Merge Sort

## Description

Implement merge sort to sort an array of integers in non-decreasing order.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the sorted elements separated by spaces, followed by a newline
- If n = 0, print an empty line

## Example

Input:
```
5 5 3 1 4 2
```

Output:
```
1 2 3 4 5
```

## Constraints

- 0 <= n <= 100000
- All values fit in a 64-bit signed integer
- Negative values are possible

## Category

Algorithm (sorting)

## BMB Notes
- Implement merge sort using a flat vec; use a temp vec for merging
- Space-separated output with print/print_str pattern
- BMB vec supports: vec_new(), vec_push(v,x), vec_get(v,i), vec_set(v,i,x), vec_len(v)
```
// Bottom-up iterative merge sort (avoids deep recursion)
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { let _p = vec_push(v, read_int()) };
let tmp = vec_new();
for _i in 0..n { let _p = vec_push(tmp, 0) };
let mut width: i64 = 1;
while width < n {
    let mut lo: i64 = 0;
    while lo < n {
        let mid: i64 = if lo + width < n { lo + width } else { n };
        let hi: i64 = if lo + width * 2 < n { lo + width * 2 } else { n };
        // merge v[lo..mid] and v[mid..hi] into tmp
        let mut i: i64 = lo; let mut j: i64 = mid; let mut k: i64 = lo;
        while i < mid && j < hi {
            if vec_get(v, i) <= vec_get(v, j) {
                let _s = vec_set(tmp, k, vec_get(v, i)); i = i + 1
            } else {
                let _s = vec_set(tmp, k, vec_get(v, j)); j = j + 1
            };
            k = k + 1
        };
        while i < mid { let _s = vec_set(tmp, k, vec_get(v, i)); i = i + 1; k = k + 1 };
        while j < hi  { let _s = vec_set(tmp, k, vec_get(v, j)); j = j + 1; k = k + 1 };
        // copy tmp[lo..hi] back to v
        for idx in lo..hi { let _s = vec_set(v, idx, vec_get(tmp, idx)) };
        lo = lo + width * 2
    };
    width = width * 2
};
let mut first: i64 = 1;
for i in 0..n {
    if first == 0 { print_str(" ") } else { () };
    print(vec_get(v, i));
    first = 0
};
println_str("");
0
```
