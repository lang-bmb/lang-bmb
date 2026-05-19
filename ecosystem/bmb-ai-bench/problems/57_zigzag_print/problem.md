# Zigzag Print

Arrange n items into rows of w. Print odd rows left-to-right, even rows right-to-left.

## Input
- First integer: n
- Second integer: w (row width)
- Next n integers: items in order

## Output
Each row space-separated on its own line. Row 1 forward, row 2 backward, etc.

## Example
`6 3 1 2 3 4 5 6` -> row1 forward [1,2,3], row2 backward [6,5,4]
Output:
```
1 2 3
6 5 4
```

## BMB Notes
- Use `print(x)` (no newline) + `print_str(" ")` for space-separated; `println_str("")` for newline
- Row direction: `row % 2 == 0` → forward, `row % 2 == 1` → backward
```
// forward row: j = i..actual_end
let mut j: i64 = i;
while j < actual_end {
    if j > i { let _s = print_str(" "); () } else { () };
    let _p = print(vec_get(v, j)); j = j + 1
};
println_str("");
// backward row: j = actual_end-1 downto i
let mut j: i64 = actual_end - 1;
let mut first: i64 = 1;
while j >= i {
    if first == 0 { let _s = print_str(" "); () } else { () };
    let _p = print(vec_get(v, j)); first = 0; j = j - 1
};
println_str("");
```
