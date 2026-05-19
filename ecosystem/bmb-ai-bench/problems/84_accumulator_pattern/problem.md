# Accumulator Pattern

Apply an accumulation function to an array.

## Input
- First integer: **t** (number of test cases)
- Each test case: op n v1..vn
  - op=1 (sum): sum of array
  - op=2 (product): product of array
  - op=3 (min): minimum value
  - op=4 (max): maximum value

## Output
Result per test case (one per line).

## Example
`1 1 3 1 2 3` → t=1, op=1(sum), n=3, values=[1,2,3] → 6
`1 2 4 1 2 3 4` → t=1, op=2(product), n=4, values=[1,2,3,4] → 24
`1 3 5 3 1 4 1 5` → t=1, op=3(min), n=5, values=[3,1,4,1,5] → 1

## IMPORTANT: Reading Multiple Queries

The first integer is **t** (test case count), NOT the op or array size.

```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let op: i64 = read_int();
    let n: i64 = read_int();
    // read n values and compute
    let mut j: i64 = 0;
    let mut acc: i64 = /* initial value depends on op */;
    while j < n {
        let v: i64 = read_int();
        // apply accumulator
        set j = j + 1;
    };
    println(acc);
    set i = i + 1;
};
0
```

## Initial Values

- op=1 (sum): initial acc = 0
- op=2 (product): initial acc = 1
- op=3 (min): initial acc = first element (read first, then loop n-1 times) or i64_max
- op=4 (max): initial acc = first element (read first, then loop n-1 times) or i64_min

Use `i64_min(a, b)` and `i64_max(a, b)` for min/max comparison.
