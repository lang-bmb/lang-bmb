# Counting Sort

## Description

Sort an array of non-negative integers using counting sort.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Second integer: `max_val`, the maximum possible value (0 <= max_val <= 1000000)
- Next `n` integers: the elements (each in [0, max_val])

**Output** (stdout):
- Print the sorted elements separated by spaces, followed by a newline

## Example

Input:
```
5 5 3 1 4 1 5
```

Output:
```
1 1 3 4 5
```

## Constraints

- 1 <= n <= 100000
- 0 <= max_val <= 1000000
- All elements are in [0, max_val]

## Category

Algorithm (sorting)

## BMB Notes
- Read n and max_val first, then read n elements
- Create a count vec of size max_val+1, initialized to zeros
- Increment `counts[val]` for each input element
- Output: for each value 0..=max_val, print value `counts[v]` times, space-separated
```
let n: i64 = read_int();
let max_val: i64 = read_int();
let counts = vec_new();
for _i in 0..(max_val+1) { let _p = vec_push(counts, 0) };
for _i in 0..n {
    let v: i64 = read_int();
    let _s = vec_set(counts, v, vec_get(counts, v) + 1)
};
let mut first: i64 = 1;
for v in 0..(max_val+1) {
    let cnt: i64 = vec_get(counts, v);
    for _j in 0..cnt {
        if first == 0 { print_str(" ") } else { () };
        print(v);
        first = 0
    }
};
println_str("");
0
```
