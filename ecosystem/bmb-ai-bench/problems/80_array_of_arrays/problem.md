# Array of Arrays

Given a matrix with `rows` rows and `cols` columns, compute column sums.

## Input
- First integer: rows
- Second integer: cols
- Next rows*cols integers: matrix row-major

## Output
Column sums, space-separated on one line.

## Example
`2 3 1 2 3 4 5 6` -> [[1,2,3],[4,5,6]] -> col sums: 5 7 9

## BMB Notes
- No nested vec needed: store matrix flat in one vec with `row * cols + col` indexing
```
let rows: i64 = read_int();
let cols: i64 = read_int();
let v = vec_new();
for _i in 0..(rows * cols) { let _p = vec_push(v, read_int()) };
for j in 0..cols {
    let mut sum: i64 = 0;
    for i in 0..rows { sum = sum + vec_get(v, i * cols + j) };
    if j > 0 { let _s = print_str(" "); () } else { () };
    let _p = print(sum)
};
println_str("");
vec_free(v);
0
```
