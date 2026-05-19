# Text Wrap

Arrange n integers into rows of w items each, print sum per row.

## Input
- First integer: n
- Second integer: w (row width)
- Next n integers: the items

## Output
Sum of each row (one per line). Last row may be shorter.

## Example
`6 3 1 2 3 4 5 6` -> rows [1,2,3]->6, [4,5,6]->15

## BMB Notes
```
let n: i64 = read_int();
let w: i64 = read_int();
let mut i: i64 = 0;
while i < n {
    let mut sum: i64 = 0;
    let mut j: i64 = 0;
    while j < w && i + j < n {
        sum = sum + read_int();
        j = j + 1
    };
    println(sum);
    i = i + w
};
0
```
