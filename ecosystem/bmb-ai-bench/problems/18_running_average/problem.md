# Running Average

## Description

Compute the running (cumulative) average of a sequence of integers. After reading each number, print the average of all numbers read so far, truncated to an integer (floor division).

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 100000)
- Next `n` integers: the elements

**Output** (stdout):
- Print `n` lines, each containing the running average (truncated to integer)

## Example

Input:
```
4 10 20 30 40
```

Output:
```
10
15
20
25
```

## Constraints

- 1 <= n <= 100000
- All partial sums fit in a 64-bit signed integer

## Category

System (streaming computation)

## BMB Notes
- Maintain a running sum; after each read, print sum / count (integer division = truncate)
```
let n: i64 = read_int();
let mut sum: i64 = 0;
let mut i: i64 = 1;
while i <= n {
    let v: i64 = read_int();
    set sum = sum + v;
    println(sum / i);
    set i = i + 1
};
0
