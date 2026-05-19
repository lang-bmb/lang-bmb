# Binary Exponentiation

## Description

Compute a^b for multiple queries using binary exponentiation (fast power).

**Input** (stdin):
- First integer: `n`, the number of queries (1 <= n <= 100)
- For each query: two integers `a b`

**Output** (stdout):
- For each query, print a^b on its own line

## Example

Input:
```
3 2 10 3 3 5 1
```

Output:
```
1024
27
5
```

## Constraints

- 1 <= n <= 100
- 1 <= a <= 1000
- 0 <= b <= 30

## Category

Algorithm (fast exponentiation)

## BMB Notes
- b ≤ 30, result fits in i64 — no modulus needed
- Use `e band 1` to test the low bit; shift right by `e = e / 2`
**CRITICAL**: Use `set` for ALL variable updates inside loops. `result = result * b` does NOT work — must be `set result = result * b`.

```
fn fast_pow(base: i64, exp: i64) -> i64 = {
    let mut result: i64 = 1;
    let mut b: i64 = base;
    let mut e: i64 = exp;
    while e > 0 {
        if (e band 1) == 1 { set result = result * b } else { () };
        set b = b * b;
        set e = e / 2
    };
    result
};

fn main() -> i64 = {
    let n: i64 = read_int();
    let mut i: i64 = 0;
    while i < n {
        let a: i64 = read_int();
        let b: i64 = read_int();
        println(fast_pow(a, b));
        set i = i + 1
    };
    0
};
```
