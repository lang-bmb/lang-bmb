# Array Rotation

## Description

Rotate an array left by k positions.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Second integer: `k`, the number of left rotation steps (0 <= k < n)
- Next `n` integers: the elements

**Output** (stdout):
- Print the rotated array elements separated by spaces, followed by a newline

## Example

Input:
```
5 2 1 2 3 4 5
```

Output:
```
3 4 5 1 2
```

(Rotate left by 2: [1,2,3,4,5] → [3,4,5,1,2])

## Constraints

- 1 <= n <= 100000
- 0 <= k < n
- All values fit in a 64-bit signed integer

## Category

Algorithm (array manipulation)

## BMB Notes
- No need to physically rotate; just print elements from index k to n-1, then 0 to k-1
- Space-separated output with print/print_str pattern
```
let n: i64 = read_int();
let k: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
let mut first: i64 = 1;
for i in k..n {
    if first == 0 { print_str(" ") } else { () };
    print(vec_get(v, i));
    first = 0
};
for i in 0..k {
    if first == 0 { print_str(" ") } else { () };
    print(vec_get(v, i));
    first = 0
};
println_str("");
0
```
