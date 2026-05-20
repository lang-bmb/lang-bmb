# Sieve Primes

## Description

Count the number of prime numbers up to n (**inclusive**) using the Sieve of Eratosthenes.

**Input** (stdin):
- A single integer `n` (0 <= n <= 10000000)

**Output** (stdout):
- Print the count of primes in [2, n] **inclusive**

## IMPORTANT: n is inclusive

The range is **[2, n]** — the number n itself is included if it is prime.
- n=2: primes are {2} → count=**1** (not 0)
- n=3: primes are {2,3} → count=**2** (not 1)
- n=10: primes are {2,3,5,7} → count=**4**

Your sieve loop must go up to and **including** n:
**CRITICAL**: Use `set i = i + 1` NOT `i = i + 1`. BMB requires `set` for all variable reassignment.
**CRITICAL**: BMB has NO `break` in while loops. Use a flag variable to exit early (see example below).

```
fn main() -> i64 = {
    let n: i64 = read_int();
    let sieve = vec_new();
    let mut i: i64 = 0;
    while i <= n {
        let _p = vec_push(sieve, 1);
        set i = i + 1
    };
    let _a = vec_set(sieve, 0, 0);
    if n >= 1 { let _b = vec_set(sieve, 1, 0) } else { () };
    let mut p: i64 = 2;
    while p * p <= n {
        if vec_get(sieve, p) == 1 {
            let mut m: i64 = p * p;
            while m <= n {
                let _c = vec_set(sieve, m, 0);
                set m = m + p
            }
        } else { () };
        set p = p + 1
    };
    let mut count: i64 = 0;
    let mut q: i64 = 2;
    while q <= n {
        if vec_get(sieve, q) == 1 { set count = count + 1 } else { () };
        set q = q + 1
    };
    println(count);
    0
};
```

## Example

Input:
```
10
```

Output:
```
4
```

(Primes up to 10: 2, 3, 5, 7 → count = 4)

Input:
```
2
```

Output:
```
1
```

(2 is prime, so count = 1)

## Constraints

- 0 <= n <= 10000000

## Category

Algorithm (prime sieve)
