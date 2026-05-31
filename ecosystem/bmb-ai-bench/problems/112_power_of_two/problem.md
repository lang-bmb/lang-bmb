# Power of Two

## Description

Given an integer `n`, return `"true"` if it is a power of two, `"false"` otherwise.

An integer is a power of two if there exists integer `x` such that `n == 2^x`.

**Input** (stdin):
- One integer `n` (-2^31 <= n <= 2^31)

**Output** (stdout):
- `"true"` or `"false"`

## Example

Input:
```
16
```

Output:
```
true
```

## Constraints

- -2^31 <= n <= 2^31

## Category

Math / Bit Manipulation

## BMB Notes
- Key insight: n > 0 and (n AND (n-1)) == 0 checks for power of 2
- Powers of 2 have exactly one bit set; n & (n-1) clears the lowest set bit
- BMB uses `band` for bitwise AND: `n band (n-1)`

```
fn main() -> i64 = {
    let n = read_int();
    if n > 0 and (n band (n - 1)) == 0 { println_str("true") }
    else { println_str("false") };
    0
};
```
