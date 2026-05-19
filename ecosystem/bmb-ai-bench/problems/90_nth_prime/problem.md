# Nth Prime

Find the n-th prime (1-indexed: 1st prime=2, 2nd=3, 3rd=5, ...).

## Input
- First integer: t
- Each test case: one positive integer n

## Output
The n-th prime, one per line.

## Examples
n=1->2; n=2->3; n=5->11

## BMB Notes
- Use trial division (no Sieve needed for typical constraints)
```
fn is_prime(n: i64) -> bool = {
    if n < 2 { false }
    else {
        let mut d: i64 = 2;
        let mut ok: bool = true;
        while d * d <= n {
            if n % d == 0 { ok = false } else { () };
            d = d + 1
        };
        ok
    }
};

fn nth_prime(n: i64) -> i64 = {
    let mut count: i64 = 0;
    let mut num: i64 = 1;
    while count < n {
        num = num + 1;
        if is_prime(num) { count = count + 1 } else { () }
    };
    num
};

fn main() -> i64 = {
    let t: i64 = read_int();
    let mut i: i64 = 0;
    while i < t {
        let n: i64 = read_int();
        println(nth_prime(n));
        set i = i + 1;
    };
    0
};
```
