# Two Sum II (Sorted Array)

## Description

Given a 1-indexed sorted array of integers and a target sum, find two numbers such that they add up to the target. Return their 1-based indices.

**Input** (stdin):
- First line: `n target` (array size and target)
- Second line: `n` space-separated sorted integers

**Output** (stdout):
- Two lines: index1, then index2 (1-based, index1 < index2)

## Example

Input:
```
4 9
2 7 11 15
```

Output:
```
1
2
```

## Constraints

- 2 <= n <= 100000
- Exactly one solution exists

## Category

Two Pointers

## BMB Notes
- Use two-pointer approach: lo=0, hi=n-1
- `vec_new()`, `vec_push(v, x)`, `vec_get(v, i)` for array
- `read_int()` reads next integer (works across newlines)

```
fn find_pair(arr: i64, lo: i64, hi: i64, target: i64) -> () = {
    if lo < hi {
        let sum = vec_get(arr, lo) + vec_get(arr, hi);
        if sum == target { println(lo + 1); println(hi + 1) }
        else if sum < target { find_pair(arr, lo + 1, hi, target) }
        else { find_pair(arr, lo, hi - 1, target) }
    }
};
fn main() -> i64 = {
    let n = read_int();
    let target = read_int();
    let arr = vec_new();
    for _i in 0..n { vec_push(arr, read_int()) };
    find_pair(arr, 0, n - 1, target);
    0
};
```
