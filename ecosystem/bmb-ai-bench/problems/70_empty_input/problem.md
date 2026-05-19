# Empty Input

Given an array (possibly empty), output its sum and count.

## Input
- First integer: n (may be 0)
- Next n integers: the array

## Output (2 lines)
1. Sum (0 if empty)
2. Count (= n)

## Example
`0` -> 0 and 0; `3 1 2 3` -> 6 and 3

## BMB Notes

**CRITICAL**: Must print BOTH sum AND n (two lines). Missing `println(n)` gives only one line of output.
**CRITICAL**: Use `set sum = sum + read_int()` NOT `sum = sum + read_int()`. Always need `set`.

```
fn main() -> i64 = {
    let n: i64 = read_int();
    let mut sum: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        set sum = sum + read_int();
        set i = i + 1
    };
    println(sum);
    println(n);
    0
};
```
