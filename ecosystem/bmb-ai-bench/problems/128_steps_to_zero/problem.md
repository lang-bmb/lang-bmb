# 128. Reverse Bits

LeetCode #190

## Problem

Reverse the bits of a 32-bit unsigned integer.

## Input

- One line: a non-negative integer

## Output

The integer with all 32 bits reversed

## Examples

```
Input: 43261596
Output: 964176192
(binary: 00000010100101000001111010011100 → 00111001011110000010100101000000)
```

```
Input: 1
Output: 2147483648
(binary: 00000000000000000000000000000001 → 10000000000000000000000000000000)
```

## BMB Notes

Shift and OR bit by bit for 32 iterations:

```bmb
fn reverse_bits(n: i64, result: i64, count: i64) -> i64
= if count >= 32 { result }
    else {
        let bit = n band 1;
        reverse_bits(n >>> 1, (result << 1) bor bit, count + 1)
    };
```

Note: Use `>>>` (logical right shift) to avoid sign extension.
