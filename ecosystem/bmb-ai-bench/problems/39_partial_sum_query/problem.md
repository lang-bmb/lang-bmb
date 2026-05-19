# Partial Sum Query

## Description

Answer range sum queries on an integer array using prefix sums.

**Input** (stdin):
- First integer: `n`, the array length (1 <= n <= 100000)
- Next `n` integers: the array elements
- Next integer: `q`, the number of queries (1 <= q <= 100000)
- For each query: two integers `l r` (0-indexed, inclusive range [l..r])

**Output** (stdout):
- For each query, print the sum of elements in the range [l..r] (inclusive) on its own line

## IMPORTANT: Reading Order

Read in this EXACT sequence: n → n elements → q → q×(l,r). Do NOT read q or any query integers before reading all n array elements.

```
let n: i64 = read_int();
let v = vec_new();
let mut i: i64 = 0;
while i < n { vec_push(v, read_int()); i = i + 1 };
// Build prefix sums
let prefix = vec_new();
vec_push(prefix, 0);
let mut pi: i64 = 0;
while pi < n { vec_push(prefix, vec_get(prefix, pi) + vec_get(v, pi)); pi = pi + 1 };
let q: i64 = read_int();
let mut qi: i64 = 0;
while qi < q {
    let l: i64 = read_int();
    let r: i64 = read_int();
    println(vec_get(prefix, r+1) - vec_get(prefix, l));
    qi = qi + 1
};
0
```

## Algorithm: Prefix Sums

Build a prefix sum array (size n+1) where `prefix[0]=0` and `prefix[i] = prefix[i-1] + elements[i-1]`.

Then `sum(l, r) = prefix[r+1] - prefix[l]`.

## Example

Input:
```
5 1 2 3 4 5 3 0 4 1 3 2 2
```

Output:
```
15
9
3
```

(n=5, array=[1,2,3,4,5], q=3 queries: sum[0..4]=15, sum[1..3]=2+3+4=9, sum[2..2]=3)

## Constraints

- 1 <= n, q <= 100000
- 0 <= l <= r < n
- All values fit in a 64-bit signed integer

## Category

Algorithm (prefix sums)
