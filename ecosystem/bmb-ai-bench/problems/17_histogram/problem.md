# Histogram

## Description

Given an array of integers in range [0, k-1], count the frequency of each value and print the histogram.

**Input** (stdin):
- First integer: `k`, the number of buckets (1 <= k <= 100)
- Second integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the elements (each in [0, k-1])

**Output** (stdout):
- Print `k` lines, one per bucket
- Each line: `i count` (bucket index and its count)

## Example

Input:
```
3 7 0 1 2 0 1 0 2
```

Output:
```
0 3
1 2
2 2
```

## Constraints

- 1 <= k <= 100
- 0 <= n <= 100000
- All elements in [0, k-1]

## Category

System (counting/aggregation)

## BMB Notes

**CRITICAL**: Do NOT use `format("{} {}", ...)` — the `{}` placeholders are NOT replaced in BMB. Use `print(i); print_str(" "); println(count)` instead.

```
fn main() -> i64 = {
    let k: i64 = read_int();
    let n: i64 = read_int();
    let counts = vec_new();
    for _i in 0..k { let _p = vec_push(counts, 0) };
    for _i in 0..n {
        let v: i64 = read_int();
        let _s = vec_set(counts, v, vec_get(counts, v) + 1)
    };
    for i in 0..k {
        print(i);
        print_str(" ");
        println(vec_get(counts, i))
    };
    0
};
```
