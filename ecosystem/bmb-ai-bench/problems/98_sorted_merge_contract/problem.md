# Sorted Merge Contract

Merge two sorted arrays into one sorted array.

## Input
- First integer: n1
- Next n1 integers: first sorted array
- Next integer: n2
- Next n2 integers: second sorted array

## Output
Merged sorted array, space-separated on one line.

## Example
`3 1 3 5 3 2 4 6` -> merge [1,3,5] and [2,4,6] -> `1 2 3 4 5 6`

## BMB Notes
- Two-pointer merge into result vec, then print space-separated
```
let n1: i64 = read_int();
let a = vec_new();
for _i in 0..n1 { vec_push(a, read_int()) };
let n2: i64 = read_int();
let b = vec_new();
for _i in 0..n2 { vec_push(b, read_int()) };
let result = vec_new();
let mut i: i64 = 0; let mut j: i64 = 0;
while i < n1 && j < n2 {
    if vec_get(a, i) <= vec_get(b, j) { vec_push(result, vec_get(a, i)); i = i + 1 }
    else { vec_push(result, vec_get(b, j)); j = j + 1 }
};
while i < n1 { vec_push(result, vec_get(a, i)); i = i + 1 };
while j < n2 { vec_push(result, vec_get(b, j)); j = j + 1 };
let total: i64 = vec_len(result);
let mut first: i64 = 1;
for idx in 0..total {
    if first == 0 { print_str(" ") } else { () };
    print(vec_get(result, idx));
    first = 0
};
println_str("");
0
```
