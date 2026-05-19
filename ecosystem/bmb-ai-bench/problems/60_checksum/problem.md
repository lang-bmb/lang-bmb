# Checksum

Sum n integers and take the result modulo 256.

## Input
- First integer: n
- Next n integers: the values

## Output
Sum mod 256 (value in [0, 255])

## Example
- `3 1 2 3` -> 6
- `2 128 128` -> 0 (256 mod 256)
- `3 100 100 100` -> 44 (300 mod 256)

## BMB Notes
- Read n values and sum them; print sum % 256
```
let n: i64 = read_int();
let mut sum: i64 = 0;
for _i in 0..n { set sum = sum + read_int() };
println(sum % 256);
0

```
