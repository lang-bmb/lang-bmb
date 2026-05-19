# Bubble Sort

## Description

Sort an array of integers using bubble sort with early termination.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 10000)
- Next `n` integers: the elements

**Output** (stdout):
- Print the sorted elements separated by spaces, followed by a newline

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

- 1 <= n <= 10000
- All values fit in a 64-bit signed integer

## Category

Algorithm (sorting)

## BMB Notes
- Use `vec_set` for swap; early termination if no swaps in a pass
- Space-separated output with print/print_str pattern
```
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
let mut swapped: i64 = 1;
while swapped == 1 {
    swapped = 0;
    for i in 0..(n-1) {
        if vec_get(v, i) > vec_get(v, i+1) {
            let tmp: i64 = vec_get(v, i);
            vec_set(v, i, vec_get(v, i+1));
            vec_set(v, i+1, tmp);
            swapped = 1
        }
    }
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
