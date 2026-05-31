# Plus One

## Description

Given a large integer represented as an array of digits, increment the integer by one and output the result digit by digit.

**Input** (stdin):
- First line: n (number of digits)
- Second line: n space-separated digits (0-9), most significant first

**Output** (stdout):
- One digit per line (may have one more digit than input if carry)

## Example

Input:
```
3
1 2 3
```

Output:
```
1
2
4
```

## Constraints

- 1 <= n <= 10000
- Each digit is 0-9

## Category

Array / Simulation

## BMB Notes
- Process from the last digit, handle carry recursively
- `vec_set(arr, i, val)` to update digit in place
- If carry remains after all digits, print 1 then zeros

```
fn add_carry(arr: i64, n: i64, i: i64) -> () = {
    if i < 0 {
        println(1);
        for j in 0..n { println(vec_get(arr, j)) }
    } else {
        let d = vec_get(arr, i) + 1;
        if d < 10 { vec_set(arr, i, d); for j in 0..n { println(vec_get(arr, j)) } }
        else { vec_set(arr, i, 0); add_carry(arr, n, i - 1) }
    }
};
```
