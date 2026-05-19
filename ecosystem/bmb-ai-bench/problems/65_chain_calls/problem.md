# Chain Calls

## Description

Apply a fixed function chain `add1(double(square(x)))` to each input value.

The three functions are:
- `square(x)` = x × x
- `double(x)` = x × 2
- `add1(x)` = x + 1

So the result for each x is: `x*x*2 + 1`

**Input** (stdin):
- First integer: `t`, number of test cases
- Next `t` integers: values of `x` (one per test case, space-separated on same line)

**Output** (stdout):
- For each x, print `add1(double(square(x)))` = `x*x*2 + 1` on its own line

## Example

Input:
```
3 0 1 2
```

Output:
```
1
3
9
```

Explanation:
- x=0: 0×0×2+1 = 1
- x=1: 1×1×2+1 = 3
- x=2: 2×2×2+1 = 9

## Constraints

- 1 ≤ t ≤ 100
- -1000 ≤ x ≤ 1000
- All values fit in a 64-bit signed integer
- Note: x can be negative (e.g., x=-1 gives (-1)²×2+1 = 3)

## Category

Edge cases / function composition

## BMB Notes
```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let x: i64 = read_int();
    let result: i64 = x * x * 2 + 1;
    println(result);
    set i = i + 1;
};
0
```
