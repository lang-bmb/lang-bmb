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
let n = read_int();
// Step 1: Read all n elements
let v = vec_new();
let i = 0;
while i < n {
    let val = read_int();
    let _p = vec_push(v, val);
    i = i + 1
};
// Step 2: Read q AFTER the array
let q = read_int();
// Step 3: Process q queries
let qi = 0;
while qi < q {
    let l = read_int();
    let r = read_int();
    // answer = prefix[r+1] - prefix[l]
    ...
    qi = qi + 1
};
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
