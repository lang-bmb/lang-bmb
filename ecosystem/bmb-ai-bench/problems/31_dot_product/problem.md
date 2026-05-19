# Dot Product

## Description

Compute the dot product of two integer vectors.

**Input** (stdin):
- First integer: `n`, the length of both vectors (1 <= n <= 100000)
- Next `n` integers: elements of vector A
- Next `n` integers: elements of vector B

**Output** (stdout):
- Print the dot product (a single integer)

## Example

Input:
```
3 1 2 3 4 5 6
```

Output:
```
32
```

(1×4 + 2×5 + 3×6 = 4 + 10 + 18 = 32)

## Constraints

- 1 <= n <= 100000
- All values fit in a 64-bit signed integer
- The dot product fits in a 64-bit signed integer

## Category

Algorithm (vector arithmetic)

## BMB Notes
- Read ALL n elements of A first, then ALL n elements of B (never interleave)
```
let n: i64 = read_int();
let a = vec_new();
for _i in 0..n { vec_push(a, read_int()) };
let b = vec_new();
for _i in 0..n { vec_push(b, read_int()) };
let mut sum: i64 = 0;
for i in 0..n { sum = sum + vec_get(a, i) * vec_get(b, i) };
println(sum);
0
```
