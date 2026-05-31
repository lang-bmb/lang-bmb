# FizzBuzz

## Description

The classic FizzBuzz problem. For each number from 1 to n:
- If divisible by both 3 and 5, print "FizzBuzz"
- If divisible by 3 only, print "Fizz"
- If divisible by 5 only, print "Buzz"
- Otherwise, print the number

**Input** (stdin):
- One integer `n` (1 <= n <= 1000)

**Output** (stdout):
- n lines, one per number

## Example

Input:
```
15
```

Output:
```
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
```

## Constraints

- 1 <= n <= 1000

## Category

Algorithm (basic)

## BMB Notes
- BMB: use `print_str("Fizz")` for strings; `println(i)` for integers
- Check 15 first (both divisors), then 3, then 5, else number
- Use `for i in 1..n+1 { ... }` for 1-indexed loop
```
let n: i64 = read_int();
for i in 1..n+1 {
    if i % 15 == 0 { let _p = println_str("FizzBuzz") }
    else if i % 3 == 0 { let _p = println_str("Fizz") }
    else if i % 5 == 0 { let _p = println_str("Buzz") }
    else { let _p = println(i) }
};
0
```
