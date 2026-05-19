# Range Clamp

## Description

Clamp each element of an array to a given range [lo, hi] using contracts.

**IMPORTANT**: Name your clamp function `clamp_val` (not `clamp` — that name is reserved in BMB's standard library).

**Input** (stdin):
- First two integers: `lo hi` (lo <= hi)
- Third integer: `n`, the number of elements (1 <= n <= 100000)
- Next `n` integers: the elements

**Output** (stdout):
- Print the clamped elements separated by spaces

## Contract Requirement

The `clamp_val` function must have:
```
pre lo <= hi
post ret >= lo and ret <= hi
```

## Example

Input:
```
0 10 5 -3 5 15 0 7
```

Output:
```
0 5 10 0 7
```

## Constraints

- lo <= hi
- 1 <= n <= 100000
- All values fit in a 64-bit signed integer

## BMB Notes

**CRITICAL**: Name the function `clamp_val` (NOT `clamp`). `clamp` is a reserved built-in in BMB and causes a linker error.

```
fn clamp_val(x: i64, lo: i64, hi: i64) -> i64
    pre lo <= hi
    post ret >= lo and ret <= hi
= if x < lo { lo } else if x > hi { hi } else { x };

fn main() -> i64 = {
    let lo: i64 = read_int();
    let hi: i64 = read_int();
    let n: i64 = read_int();
    let mut first: i64 = 1;
    let mut i: i64 = 0;
    while i < n {
        let x: i64 = read_int();
        if first == 0 { print_str(" ") } else { () };
        print(clamp_val(x, lo, hi));
        set first = 0;
        set i = i + 1
    };
    println_str("");
    0
};
```

## Category

Contract (range verification)
