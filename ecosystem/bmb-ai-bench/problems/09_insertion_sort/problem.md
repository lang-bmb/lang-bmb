# Insertion Sort

## Description

Sort an array of integers using insertion sort.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 10000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the sorted elements separated by spaces, followed by a newline
- If n = 0, print an empty line

## Example

Input:
```
5 5 3 1 4 2
```

Output:
```
1 2 3 4 5
```

## Constraints

- 0 <= n <= 10000
- All values fit in a 64-bit signed integer

## Category

Algorithm (sorting)

## BMB Notes
- Use `vec_set` to shift elements; `vec_get` to read
- Space-separated output with print/print_str pattern
```
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
for i in 1..n {
    let key: i64 = vec_get(v, i);
    let mut j: i64 = i - 1;
    while j >= 0 && vec_get(v, j) > key {
        vec_set(v, j+1, vec_get(v, j));
        j = j - 1
    };
    vec_set(v, j+1, key)
};
let mut first: i64 = 1;
for i in 0..n {
    if first == 0 { print_str(" ") } else { () };
    print(vec_get(v, i));
    first = 0
};
println_str("");
0
```
