# Collatz Length

## Description

Compute the length of the Collatz sequence starting from n until it reaches 1.

The Collatz sequence: if n is even, next = n/2; if n is odd, next = 3*n+1. Length includes the starting number and 1.

**Input** (stdin):
- First integer: `t`, the number of queries (1 <= t <= 100)
- For each query: a single integer `n` (1 <= n <= 1000000)

**Output** (stdout):
- For each query, print the Collatz sequence length on its own line

## IMPORTANT: Reading Multiple Queries

The input always starts with `t` (the number of queries). You must:
1. Read `t` first
2. Loop `t` times
3. In each iteration, read ONE integer `n` and compute+print its Collatz length

**DO NOT** treat the first integer as the value to compute. It is the query count.

Reading pattern:
```
let t = read_int();   // number of queries
let i = 0;
while i < t {
    let n = read_int();   // value for THIS query
    let length = collatz_len(n);
    let _p = println(length);
    i = i + 1
};
```

## Example

Input:
```
3 1 6 27
```

Output:
```
1
9
112
```

(t=3 queries: n=1→length 1, n=6→length 9, n=27→length 112)
(Sequence from 6: 6,3,10,5,16,8,4,2,1 → length 9)

## Constraints

- 1 <= t <= 100
- 1 <= n <= 1000000

## Category

Algorithm (number theory)
