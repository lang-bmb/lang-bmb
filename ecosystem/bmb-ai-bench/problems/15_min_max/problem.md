# Min and Max

## Description

Find the minimum and maximum values in an array.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print two integers on one line separated by a space: min max

## Example

Input:
```
5 3 1 4 1 5
```

Output:
```
1 5
```

## Constraints

- 1 <= n <= 100000
- All values fit in a 64-bit signed integer

## Category

System (array processing)

## BMB Notes
- Use `min(a, b)` and `max(a, b)` builtins; output both on ONE line with `print_str(" ")` between
```
let n: i64 = read_int();
let first: i64 = read_int();
let mut mn: i64 = first; let mut mx: i64 = first;
for _i in 1..n {
    let v: i64 = read_int();
    mn = min(mn, v);
    mx = max(mx, v)
};
print(mn); print_str(" "); println(mx);
0
```
