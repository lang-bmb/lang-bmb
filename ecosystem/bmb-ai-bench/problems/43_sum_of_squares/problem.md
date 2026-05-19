# Sum Of Squares

## Description

Compute the sum of squares 1^2 + 2^2 + ... + n^2 for multiple queries.

**Input** (stdin):
- First integer: `t`, the number of queries (1 <= t <= 100)
- For each query: a single integer `n` (1 <= n <= 10000)

**Output** (stdout):
- For each query, print 1^2 + 2^2 + ... + n^2 on its own line

## IMPORTANT: Reading Multiple Queries

**CRITICAL**: Use `set` for ALL variable updates. `qi = qi + 1` does NOT work — must be `set qi = qi + 1`.
**CRITICAL**: Declare mutable variables with `let mut`. `let qi = 0` is immutable and cannot be incremented.

```
fn main() -> i64 = {
    let t: i64 = read_int();
    let mut qi: i64 = 0;
    while qi < t {
        let n: i64 = read_int();
        let mut sum: i64 = 0;
        let mut i: i64 = 1;
        while i <= n {
            set sum = sum + i * i;
            set i = i + 1
        };
        println(sum);
        set qi = qi + 1
    };
    0
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
