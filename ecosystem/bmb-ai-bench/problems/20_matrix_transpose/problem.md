# Matrix Transpose

## Description

Transpose a matrix (rows become columns, columns become rows).

**Input** (stdin):
- First two integers: `rows cols` (dimensions)
- Next `rows * cols` integers: matrix elements in row-major order

**Output** (stdout):
- Print the transposed matrix (cols rows), row by row
- Elements separated by spaces, one row per line

## Example

Input:
```
2 3 1 2 3 4 5 6
```

Output:
```
1 4
2 5
3 6
```

## Constraints

- 1 <= rows, cols <= 100
- All values fit in a 64-bit signed integer

## Category

System (matrix operation)

## BMB Notes
- Store the matrix flat: `v[row * cols + col]`
- Transposed output: for each output row `c` (0..cols), print row `r` (0..rows): `v[r * cols + c]`
- Space-separated per row with `println_str("")` after each row
```
let rows: i64 = read_int();
let cols: i64 = read_int();
let v = vec_new();
for _i in 0..(rows * cols) { vec_push(v, read_int()) };
for c in 0..cols {
    let mut first: i64 = 1;
    for r in 0..rows {
        if first == 0 { print_str(" ") } else { () };
        print(vec_get(v, r * cols + c));
        first = 0
    };
    println_str("")
};
0
```
