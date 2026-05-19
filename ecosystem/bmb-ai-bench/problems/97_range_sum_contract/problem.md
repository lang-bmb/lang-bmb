# Range Sum Contract

Answer inclusive range sum queries on an array.

## Input
- First integer: n (array length)
- Next n integers: the array
- Next integer: q (queries)
- Next 2q integers: l1 r1 l2 r2 ... (0-indexed inclusive ranges)

## Output
Sum of arr[l..r] for each query, one per line.

## Example
Array [1,2,3,4,5]: query [0,4]->15; [1,3]->9; [2,2]->3

## BMB Notes
- Build prefix sum array; `sum(l,r) = prefix[r+1] - prefix[l]`
```
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
let prefix = vec_new();
vec_push(prefix, 0);
for i in 0..n { vec_push(prefix, vec_get(prefix, i) + vec_get(v, i)) };
let q: i64 = read_int();
for _qi in 0..q {
    let l: i64 = read_int();
    let r: i64 = read_int();
    println(vec_get(prefix, r+1) - vec_get(prefix, l))
};
0
```
