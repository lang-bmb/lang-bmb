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

**CRITICAL**: Use `set` for ALL variable updates. `first = 0` does NOT work — it must be `set first = 0`.
**CRITICAL**: Use different variable names in different loops. Do NOT reuse `v` for both input reading and output printing.

```
fn main() -> i64 = {
    let n: i64 = read_int();
    let max_val: i64 = read_int();
    let counts = vec_new();
    let mut i: i64 = 0;
    while i <= max_val {
        let _p = vec_push(counts, 0);
        set i = i + 1
    };
    let mut j: i64 = 0;
    while j < n {
        let val: i64 = read_int();
        let _s = vec_set(counts, val, vec_get(counts, val) + 1);
        set j = j + 1
    };
    let mut first: i64 = 1;
    let mut vi: i64 = 0;
    while vi <= max_val {
        let cnt: i64 = vec_get(counts, vi);
        let mut k: i64 = 0;
        while k < cnt {
            if first == 0 { print_str(" ") } else { () };
            print(vi);
            set first = 0;
            set k = k + 1
        };
        set vi = vi + 1
    };
    println_str("");
    0
};
```
