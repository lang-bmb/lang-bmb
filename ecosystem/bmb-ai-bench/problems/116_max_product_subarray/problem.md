# Maximum Product Subarray

## Description

Given an integer array, find the contiguous subarray with the largest product and return that product.

**Input** (stdin):
- First line: n
- Second line: n space-separated integers

**Output** (stdout):
- Maximum product as one integer

## Example

Input:
```
3
2 3 -2 4
```

Output:
```
6
```

(Subarray [2,3] has product 6)

## Constraints

- 1 <= n <= 10000
- -10 <= arr[i] <= 10

## Category

Dynamic Programming

## BMB Notes
- Track both max and min products (negative * negative = large positive)
- At each step: new_max = max(x, cur_max*x, cur_min*x)
- At each step: new_min = min(x, cur_max*x, cur_min*x)

```
fn max_i64(a: i64, b: i64) -> i64 = if a > b { a } else { b };
fn min_i64(a: i64, b: i64) -> i64 = if a < b { a } else { b };
fn scan(arr: i64, n: i64, i: i64, cur_max: i64, cur_min: i64, best: i64) -> i64 =
    if i >= n { best }
    else {
        let x = vec_get(arr, i);
        let new_max = max_i64(x, max_i64(cur_max * x, cur_min * x));
        let new_min = min_i64(x, min_i64(cur_max * x, cur_min * x));
        scan(arr, n, i + 1, new_max, new_min, max_i64(best, new_max))
    };
```
