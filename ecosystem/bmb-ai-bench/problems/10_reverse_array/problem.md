# Reverse Array

## Description

Reverse an array of integers in-place.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the reversed array elements separated by spaces, followed by a newline
- If n = 0, print an empty line

## Example

Input:
```
5 1 2 3 4 5
```

Output:
```
5 4 3 2 1
```

## Constraints

- 0 <= n <= 100000
- All values fit in a 64-bit signed integer

## Category

Algorithm (array manipulation)

## BMB Notes
- Read into vec, then print from n-1 down to 0; space-separated with print/print_str
```
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
let mut first: i64 = 1;
let mut i: i64 = n - 1;
while i >= 0 {
    if first == 0 { print_str(" ") } else { () };
    print(vec_get(v, i));
    first = 0;
    i = i - 1
};
println_str("");
0
```
