# Safe Matrix Mul

Multiply two n x n matrices.

## Input
- First integer: n
- Next 2*n*n integers: matrix A (row-major) then matrix B (row-major)

## Output
C = A*B, each row space-separated on its own line.

## Example
`2 1 2 3 4 5 6 7 8` -> A=[[1,2],[3,4]], B=[[5,6],[7,8]] -> C=[[19,22],[43,50]]

## BMB Notes
- Flat vec (row-major): `A[i*n+k]`, `B[k*n+j]`, result `C[i*n+j]`
- Initialize C to zeros; triple loop (i,k,j) for matrix multiply
- Space-separated per row with `println_str("")` after each row
```
let n: i64 = read_int();
let a = vec_new(); let b = vec_new(); let c = vec_new();
for _i in 0..(n*n) { vec_push(a, read_int()) };
for _i in 0..(n*n) { vec_push(b, read_int()) };
for _i in 0..(n*n) { vec_push(c, 0) };
for i in 0..n {
    for k in 0..n {
        for j in 0..n {
            let v: i64 = vec_get(c, i*n+j) + vec_get(a, i*n+k) * vec_get(b, k*n+j);
            vec_set(c, i*n+j, v)
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
