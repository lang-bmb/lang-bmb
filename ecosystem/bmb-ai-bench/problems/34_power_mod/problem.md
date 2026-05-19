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

```
let n: i64 = read_int();
let mut i: i64 = 0;
while i < n {
    let a: i64 = read_int();
    let b: i64 = read_int();
    let m: i64 = read_int();
    println(pow_mod(a, b, m));
    i = i + 1
};
```

## Algorithm: Fast Exponentiation (Required)

Since b can be up to 10^18, you MUST use fast modular exponentiation (binary exponentiation):

```
fn pow_mod(base: i64, exp: i64, m: i64) -> i64 = {
    let mut result = 1;
    let mut b = base % m;
    let mut e = exp;
    while e > 0 {
        if (e band 1) == 1 { result = (result * b) % m };
        b = (b * b) % m;
        e = e / 2
    };
    result
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
