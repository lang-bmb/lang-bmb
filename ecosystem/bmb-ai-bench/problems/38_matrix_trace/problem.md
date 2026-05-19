# Matrix Trace

## Description

Compute the trace of an n×n matrix (sum of diagonal elements).

**Input** (stdin):
- First integer: `n`, the matrix dimension (1 <= n <= 1000)
- Next `n*n` integers: the matrix elements in row-major order

**Output** (stdout):
- Print the trace (sum of elements at positions [0,0], [1,1], ..., [n-1,n-1])

## Example

Input:
```
3 1 2 3 4 5 6 7 8 9
```

Output:
```
15
```

(Diagonal elements: 1, 5, 9 → sum = 15)

## Constraints

- 1 <= n <= 1000
- All values fit in a 64-bit signed integer
- The trace fits in a 64-bit signed integer

## Category

Algorithm (matrix arithmetic)

## BMB Notes
- Store flat in row-major vec: diagonal at index `i*n+i`
```
let n: i64 = read_int();
let v = vec_new();
for _i in 0..(n*n) { vec_push(v, read_int()) };
let mut trace: i64 = 0;
for i in 0..n { trace = trace + vec_get(v, i*n+i) };
println(trace);
0
```
