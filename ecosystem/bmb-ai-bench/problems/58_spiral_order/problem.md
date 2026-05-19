# Spiral Order

Given n^2 integers forming an n x n matrix (row-major), output elements in clockwise spiral order.

## Input
- First integer: n
- Next n*n integers: matrix row-major

## Output
All elements in spiral order, space-separated on one line.

## Example
`3 1 2 3 4 5 6 7 8 9` -> matrix 3x3, spiral: `1 2 3 6 9 8 7 4 5`

## BMB Notes
- Store matrix flat in one vec: `v[row * n + col]`
- Use boundary shrinking: top/bottom/left/right pointers
- Output: space-separated on one line — use `print(x)` + `print_str(" ")` + `println_str("")`
```
let v = vec_new();
for _i in 0..(n * n) { let _p = vec_push(v, read_int()) };
let mut top: i64 = 0; let mut bottom: i64 = n - 1;
let mut left: i64 = 0; let mut right: i64 = n - 1;
let mut first: i64 = 1;
while top <= bottom && left <= right {
    // right along top, down right, left along bottom, up left
    // use: if first == 0 { print_str(" ") }; print(val); first = 0
}
```
