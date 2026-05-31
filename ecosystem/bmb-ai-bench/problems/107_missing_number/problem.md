# Missing Number

## Description

Given an array of `n` distinct integers from the range `[0, n]`, find the one number that is missing.

Use the Gauss sum formula: expected sum = n*(n+1)/2, actual sum = sum of array elements.

**Input** (stdin):
- First integer: `n`, array size
- Next `n` integers: distinct values from [0, n] with exactly one missing

**Output** (stdout):
- Print the missing number

## Example

Input:
```
3 3 0 1
```

Output:
```
2
```

## Constraints

- 1 <= n <= 10000
- All values are distinct integers in [0, n]

## Category

Algorithm (math)

## BMB Notes
- Sum formula: missing = n*(n+1)/2 - sum(array)
- No array needed: read and accumulate in one pass
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let expected = n * (n + 1) / 2;
    let mut actual = 0;
    for _i in 0..n { actual = actual + read_int() };
    println(expected - actual);
    0
};
```
