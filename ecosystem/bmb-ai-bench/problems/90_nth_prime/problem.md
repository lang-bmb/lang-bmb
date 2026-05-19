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

**CRITICAL**: Use `set` for ALL variable updates. `d = d + 1` does NOT work — must be `set d = d + 1`.
**CRITICAL**: Use `i64` (1/0) instead of `bool` for is_prime return type to avoid type errors.

```
fn is_prime(n: i64) -> i64 = {
    if n < 2 { 0 }
    else {
        let mut d: i64 = 2;
        let mut ok: i64 = 1;
        while d * d <= n {
            if n % d == 0 { set ok = 0 } else { () };
            set d = d + 1
        };
        ok
    }
};

fn nth_prime(n: i64) -> i64 = {
    let mut count: i64 = 0;
    let mut num: i64 = 1;
    while count < n {
        set num = num + 1;
        if is_prime(num) == 1 { set count = count + 1 } else { () }
    };
    num
};

fn main() -> i64 = {
    let t: i64 = read_int();
    let mut i: i64 = 0;
    while i < t {
        let n: i64 = read_int();
        println(nth_prime(n));
        set i = i + 1
    };
    0
};
```
