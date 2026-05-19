# Positive Factorial

## Description

Compute factorial for multiple queries. Use a helper function `factorial(n: i64) -> i64` with contracts. The contract goes on the **helper function**, not on `main`.

**Input** (stdin):
- First integer: `n`, number of queries (1 <= n <= 20)
- Next `n` integers: values to compute factorial for (0 <= x <= 20)

**Output** (stdout):
- For each value, print x! on its own line

## Contract Requirement

The `factorial` helper function must have:
```
fn factorial(n: i64) -> i64
    pre n >= 0 and n <= 20
    post ret >= 1
= ...
```

Do NOT put contracts on `fn main()` — main has no parameters so contract variables would be undefined.

## Example

Input:
```
3 0 5 10
```

Output:
```
1
120
3628800
```

## Constraints

- 0 <= x <= 20 (20! fits in i64)

## Category

Contract (range + postcondition)

## BMB Notes
- Put the contract on the `factorial` helper, NOT on `main`
- Iterative factorial; `pre n >= 0 and n <= 20  post ret >= 1`
```
fn factorial(n: i64) -> i64
    pre n >= 0 and n <= 20
    post ret >= 1
= {
    let mut result: i64 = 1; let mut i: i64 = 2;
    while i <= n { set result = result * i; set i = i + 1 };
    result
};
fn main() -> i64 = {
    let q: i64 = read_int();
    for _i in 0..q { println(factorial(read_int())) };
    0
};

```
