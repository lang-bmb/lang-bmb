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
```
// Sieve array of size n+1 (indices 0..n inclusive)
let sieve = vec_new();
let mut i: i64 = 0;
while i <= n {   // <= n, NOT < n
    vec_push(sieve, 1);  // 1 = prime
    i = i + 1
};
// Mark 0 and 1 as non-prime
let _a = vec_set(sieve, 0, 0);
let _b = vec_set(sieve, 1, 0);
// Sieve
...
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
