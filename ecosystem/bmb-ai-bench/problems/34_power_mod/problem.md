# Power Mod

## Description

Compute modular exponentiation: a^b mod m for multiple queries using fast exponentiation.

**Input** (stdin):
- First integer: `n`, the number of queries (1 <= n <= 100)
- For each query: three integers `a b m` (all on the same token stream)

**Output** (stdout):
- For each query, print a^b mod m on its own line

## IMPORTANT: Reading Order

Read `n` FIRST. Then loop `n` times, and in each iteration read exactly THREE integers: `a`, `b`, `m`.

**CRITICAL**: Use `set` for ALL variable updates. `i = i + 1`, `result = ...`, `b = ...`, `e = ...` do NOT work without `set`.

```
fn pow_mod(base: i64, exp: i64, m: i64) -> i64 = {
    let mut result: i64 = 1;
    let mut b: i64 = base % m;
    let mut e: i64 = exp;
    while e > 0 {
        if (e band 1) == 1 { set result = (result * b) % m } else { () };
        set b = (b * b) % m;
        set e = e / 2
    };
    result
};

fn main() -> i64 = {
    let n: i64 = read_int();
    let mut i: i64 = 0;
    while i < n {
        let a: i64 = read_int();
        let bv: i64 = read_int();
        let m: i64 = read_int();
        println(pow_mod(a, bv, m));
        set i = i + 1
    };
    0
};
```

## Example

Input:
```
3 2 10 1000 3 3 7 5 1 1000
```

Output:
```
24
6
5
```

(n=3 queries: 2^10 mod 1000=24, 3^3 mod 7=6, 5^1 mod 1000=5)

## Constraints

- 1 <= n <= 100
- 1 <= a, m <= 1000000007
- 0 <= b <= 10^18

## Category

Algorithm (modular arithmetic)
