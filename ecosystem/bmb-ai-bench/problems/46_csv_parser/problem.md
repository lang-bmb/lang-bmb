# CSV Parser

Read t test cases. Each test case has n (number of fields) followed by n integers. For each case output the field count and sum separated by a space.

## Input
- First integer: t
- Each test case: n followed by n integers on one line

## Output
For each test case: `n sum` (one per line)

## Example
Input: `1 3 1 2 3` -> n=3, values=[1,2,3] -> output: `3 6`

## BMB Notes
- Read t test cases; each: read n, then sum n values; output `n sum` per line
```
let t: i64 = read_int();
for _i in 0..t {
    let n: i64 = read_int();
    let mut sum: i64 = 0;
    for _j in 0..n { set sum = sum + read_int() };
    print(n); print_str(" "); println(sum)
};
0

```
