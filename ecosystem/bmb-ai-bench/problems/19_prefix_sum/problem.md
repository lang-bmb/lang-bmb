# Prefix Sum Queries

## Description

Build a prefix sum array, then answer range sum queries.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Next `n` integers: the array elements
- Next integer: `q`, the number of queries (1 <= q <= 100000)
- For each query: two integers `l r` (0-indexed, inclusive)

**Output** (stdout):
- For each query, print the sum of elements from index l to r (inclusive)

## Example

Input:
```
5 1 2 3 4 5 3 0 2 1 3 0 4
```

Output:
```
6
9
15
```

## Constraints

- 1 <= n <= 100000
- 1 <= q <= 100000
- 0 <= l <= r < n
- All partial sums fit in a 64-bit signed integer

## Category

System (data structure / precomputation)

## BMB Notes
- Build prefix array: `prefix[i+1] = prefix[i] + arr[i]`; answer `sum(l,r) = prefix[r+1] - prefix[l]`
```
let n: i64 = read_int();
let arr = vec_new();
for _i in 0..n { vec_push(arr, read_int()) };
let prefix = vec_new();
vec_push(prefix, 0);
for i in 0..n { vec_push(prefix, vec_get(prefix, i) + vec_get(arr, i)) };
let q: i64 = read_int();
for _qi in 0..q {
    let l: i64 = read_int(); let r: i64 = read_int();
    println(vec_get(prefix, r+1) - vec_get(prefix, l))
};
0
```
