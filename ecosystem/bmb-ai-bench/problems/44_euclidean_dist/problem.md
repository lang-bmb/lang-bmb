# Euclidean Distance Squared

## Description

Compute the squared Euclidean distance between two n-dimensional integer vectors.

**Input** (stdin):
- First integer: `n`, the vector dimension (1 <= n <= 100000)
- Next `n` integers: ALL elements of vector A (read all n before reading B)
- Next `n` integers: ALL elements of vector B

**Output** (stdout):
- Print the squared Euclidean distance: sum of (a[i] - b[i])^2 for i in [0..n)

## IMPORTANT: Reading Order

Read ALL n elements of vector A first, THEN read ALL n elements of vector B.
Do NOT interleave reading of A and B elements.

```
let n: i64 = read_int();
let a = vec_new();
let mut i: i64 = 0;
while i < n { vec_push(a, read_int()); i = i + 1 };
let b = vec_new();
let mut j: i64 = 0;
while j < n { vec_push(b, read_int()); j = j + 1 };
let mut dist: i64 = 0;
let mut k: i64 = 0;
while k < n {
    let diff: i64 = vec_get(a, k) - vec_get(b, k);
    dist = dist + diff * diff;
    k = k + 1
};
println(dist);
0
```

## Example

Input:
```
3 1 2 3 4 5 6
```

Output:
```
27
```

(n=3, A=[1,2,3], B=[4,5,6]: (1-4)²+(2-5)²+(3-6)² = 9+9+9 = 27)

## Constraints

- 1 <= n <= 100000
- All values fit in a 64-bit signed integer
- The result fits in a 64-bit signed integer

## Category

Algorithm (vector arithmetic)
