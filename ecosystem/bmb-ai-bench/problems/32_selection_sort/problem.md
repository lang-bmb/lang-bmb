# Selection Sort

## Description

Sort an array of integers using selection sort.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 10000)
- Next `n` integers: the elements

**Output** (stdout):
- Print the sorted elements separated by spaces, followed by a newline

## Example

Input:
```
5 3 1 4 1 5
```

Output:
```
1 1 3 4 5
```

## Constraints

- 1 <= n <= 10000
- All values fit in a 64-bit signed integer

## Category

Algorithm (sorting)

## BMB Notes
- Use `vec_set` for swap: `let tmp = vec_get(v,i); vec_set(v,i,vec_get(v,min_idx)); vec_set(v,min_idx,tmp)`
- Space-separated output: print with `print_str(" ")` between elements, `println_str("")` at end
```
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
for i in 0..n {
    let mut min_idx: i64 = i;
    for j in (i+1)..n {
        if vec_get(v, j) < vec_get(v, min_idx) { min_idx = j }
    };
    let tmp: i64 = vec_get(v, i);
    vec_set(v, i, vec_get(v, min_idx));
    vec_set(v, min_idx, tmp)
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
