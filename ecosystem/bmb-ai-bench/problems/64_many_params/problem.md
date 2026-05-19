# Many Params

Read t test cases, each with 12 integers (two groups of 6). Compute their dot product.

## Input
- First integer: **t** (number of test cases)
- Each test case: 12 integers **a1 a2 a3 a4 a5 a6 b1 b2 b3 b4 b5 b6** (all on one line)

## Output
a1*b1 + a2*b2 + ... + a6*b6 (one per line, t lines total)

## Example
`1 1 2 3 4 5 6 1 1 1 1 1 1` → t=1, a=[1,2,3,4,5,6], b=[1,1,1,1,1,1] → 1+2+3+4+5+6=21

## IMPORTANT: t is the query count

```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let a1: i64 = read_int();
    let a2: i64 = read_int();
    let a3: i64 = read_int();
    let a4: i64 = read_int();
    let a5: i64 = read_int();
    let a6: i64 = read_int();
    let b1: i64 = read_int();
    let b2: i64 = read_int();
    let b3: i64 = read_int();
    let b4: i64 = read_int();
    let b5: i64 = read_int();
    let b6: i64 = read_int();
    println(a1*b1 + a2*b2 + a3*b3 + a4*b4 + a5*b5 + a6*b6);
    set i = i + 1;
};
0
```
