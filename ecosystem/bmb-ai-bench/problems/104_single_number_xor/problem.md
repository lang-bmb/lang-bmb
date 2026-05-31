# Single Number (XOR)

## Description

Given an array of integers where every element appears exactly twice except for one element which appears exactly once. Find that single element.

Use XOR: XOR of a number with itself is 0, XOR with 0 is the number itself.

**Input** (stdin):
- First integer: `n`, array size (odd, 1 <= n <= 10001)
- Next `n` integers: array elements

**Output** (stdout):
- Print the single element

## Example

Input:
```
5 4 1 2 1 2
```

Output:
```
4
```

## Constraints

- 1 <= n <= 10001 (odd)
- Exactly one element appears once, all others appear twice

## Category

Algorithm (bit manipulation)

## BMB Notes
- XOR all elements: duplicates cancel out, leaving the unique element
- BMB uses `a bxor b` for XOR (not `^`)
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let mut result: i64 = 0;
    for _i in 0..n {
        let x = read_int();
        result = result bxor x
    };
    println(result);
    0
};
```
