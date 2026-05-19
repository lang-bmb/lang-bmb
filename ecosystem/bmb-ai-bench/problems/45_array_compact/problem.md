# Array Compact

## Description

Remove all zeros from an array and print the remaining elements.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Next `n` integers: the elements

**Output** (stdout):
- First line: count of non-zero elements
- Second line (if count > 0): the non-zero elements separated by spaces

## Example

Input:
```
6 1 0 2 0 3 0
```

Output:
```
3
1 2 3
```

## Constraints

- 1 <= n <= 100000
- All values fit in a 64-bit signed integer
- If all elements are zero, print `0` and no second line

## Category

Algorithm (array filtering)

## BMB Notes
- Collect non-zero elements into a result vec using vec_push
- Print count first, then space-separated elements (if count > 0)
- `println_str("")` for the empty line when all zeros
```
let n: i64 = read_int();
let result = vec_new();
for _i in 0..n {
    let v: i64 = read_int();
    if v != 0 { let _p = vec_push(result, v) } else { () }
};
let cnt: i64 = vec_len(result);
println(cnt);
if cnt > 0 {
    let mut first: i64 = 1;
    for i in 0..cnt {
        if first == 0 { print_str(" ") } else { () };
        print(vec_get(result, i));
        first = 0
    };
    println_str("")
} else { () };
0
```
