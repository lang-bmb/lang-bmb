# Sum Of Squares

## Description

Compute the sum of squares 1^2 + 2^2 + ... + n^2 for multiple queries.

**Input** (stdin):
- First integer: `t`, the number of queries (1 <= t <= 100)
- For each query: a single integer `n` (1 <= n <= 10000)

**Output** (stdout):
- For each query, print 1^2 + 2^2 + ... + n^2 on its own line

## IMPORTANT: Reading Multiple Queries

The FIRST integer is `t` (number of queries). Loop `t` times, reading one `n` per iteration.

```
let t = read_int();   // number of queries — read FIRST
let qi = 0;
while qi < t {
    let n = read_int();   // this query's n
    // compute sum of squares 1^2 + ... + n^2
    let sum = 0;
    let i = 1;
    while i <= n {
        sum = sum + i * i;
        i = i + 1
    };
    let _p = println(sum);
    qi = qi + 1
};
```

## Example

Input:
```
3 1 3 5
```

Output:
```
1
14
55
```

(t=3 queries: n=1→1, n=3→1+4+9=14, n=5→1+4+9+16+25=55)

## Constraints

- 1 <= t <= 100
- 1 <= n <= 10000
- Result fits in a 64-bit signed integer

## Category

Algorithm (arithmetic)
