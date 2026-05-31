# Counting Bits

## Description

Given an integer `n`, for every integer `i` in the range `[0, n]`, print the number of 1's in the binary representation of `i`. Print one number per line.

**Input** (stdin):
- One integer `n` (0 <= n <= 100000)

**Output** (stdout):
- n+1 lines, the i-th line contains the count of set bits in i

## Example

Input:
```
5
```

Output:
```
0
1
1
2
1
2
```

## Constraints

- 0 <= n <= 100000

## Category

Bit Manipulation / Dynamic Programming

## BMB Notes
- Use Brian Kernighan's algorithm: `x band (x-1)` clears the lowest set bit
- `band` is BMB's bitwise AND operator
- Count iterations until x becomes 0

```
fn popcount(x: i64, acc: i64) -> i64 =
    if x == 0 { acc }
    else { popcount(x band (x - 1), acc + 1) };

fn main() -> i64 = {
    let n = read_int();
    for i in 0..n + 1 { println(popcount(i, 0)) };
    0
};
```
