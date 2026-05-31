# Find the Duplicate Number

## Description

Given an array of n+1 integers where each integer is in the range [1, n], find the one duplicate number. The array contains exactly one duplicate (which may appear more than twice).

Use the sum formula: sum(array) - n*(n+1)/2 gives the excess, but for multiple copies use XOR or just sum (assuming exactly one element duplicated).

For simplicity: exactly one element appears twice, all others appear once.

**Input** (stdin):
- First integer: `n` (1 <= n <= 10000)
- Next `n+1` integers: values in [1, n] with exactly one duplicate

**Output** (stdout):
- Print the duplicate number

## Example

Input:
```
4 1 3 4 2 2
```

Output:
```
2
```

## Constraints

- 1 <= n <= 10000
- Exactly one element appears twice, all others once
- All values in [1, n]

## Category

Algorithm (math/bit)

## BMB Notes
- Sum formula: sum(array) - n*(n+1)/2 = duplicate number
- No extra space needed
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let expected = n * (n + 1) / 2;
    let mut actual = 0;
    for _i in 0..n+1 { actual = actual + read_int() };
    println(actual - expected);
    0
};
```
