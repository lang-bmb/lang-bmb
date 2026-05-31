# 133. Number Complement

LeetCode #476

## Problem

Given a positive integer, return its complement. The complement flips all bits up to the highest set bit.

## Input

- One line: the positive integer

## Output

The complement value

## Examples

```
Input: 5
Output: 2   (101 → 010)
```

```
Input: 1
Output: 0   (1 → 0)
```

## BMB Notes

Build a mask of all 1s up to the highest bit, then XOR with the number:

```bmb
fn highest_bit_mask(n: i64, mask: i64) -> i64
= if mask >= n { mask }
    else { highest_bit_mask(n, mask * 2 + 1) };

println(n bxor highest_bit_mask(n, 1));
```
