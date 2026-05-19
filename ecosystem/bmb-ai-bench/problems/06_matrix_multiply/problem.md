# Matrix Multiplication

## Description

Multiply two square matrices of size n×n.

Matrices are stored as flat arrays in row-major order.

**Input** (stdin):
- First integer: `n`, the matrix dimension (1 <= n <= 50)
- Next n*n integers: matrix A elements (row by row)
- Next n*n integers: matrix B elements (row by row)

**Output** (stdout):
- Print the n*n elements of the result matrix C = A × B
- Print row by row, elements separated by spaces, each row on a new line

## Example

Input:
```
2 1 2 3 4 5 6 7 8
```

Output:
```
19 22
43 50
```

## Constraints

- 1 <= n <= 50
- All values fit in a 64-bit signed integer (no overflow in n <= 50)

## Category

Algorithm (linear algebra)

## BMB Notes
- Store both matrices as flat vecs in row-major order: `A[i*n+k]`, `B[k*n+j]`
- Triple nested loop (i, k, j): compute `C[i*n+j] += A[i*n+k] * B[k*n+j]`
- Initialize result vec C with zeros: `for _i in 0..(n*n) { let _p = vec_push(c, 0) }`
- Row output: space-separated on each line — use print/print_str pattern
```
let n: i64 = read_int();
let a = vec_new(); let b = vec_new(); let c = vec_new();
for _i in 0..(n*n) { let _p = vec_push(a, read_int()) };
for _i in 0..(n*n) { let _p = vec_push(b, read_int()) };
for _i in 0..(n*n) { let _p = vec_push(c, 0) };
for i in 0..n {
    for k in 0..n {
        for j in 0..n {
            let v = vec_get(c, i*n+j) + vec_get(a, i*n+k) * vec_get(b, k*n+j);
            let _s = vec_set(c, i*n+j, v)
        }
    }
};
for i in 0..n {
    let mut first: i64 = 1;
    for j in 0..n {
        if first == 0 { print_str(" ") } else { () };
        print(vec_get(c, i*n+j));
        first = 0
    };
    println_str("")
};
0
```
