# Palindrome Number

## Description

Given t queries, for each query determine if a given integer is a palindrome. An integer is a palindrome when it reads the same forward and backward. Negative numbers are never palindromes.

**Input** (stdin):
- First integer: `t`, number of test cases (1 <= t <= 100)
- Next `t` integers: each integer `n` to check (-2^31 <= n <= 2^31 - 1)

**Output** (stdout):
- For each test case: "yes" if palindrome, "no" otherwise

## Example

Input:
```
3 121 -121 10
```

Output:
```
yes
no
no
```

## Constraints

- 1 <= t <= 100
- Negative numbers are NOT palindromes
- 0 is a palindrome

## Category

Algorithm (math)

## BMB Notes
- Reverse digits: `while x > 0 { rev = rev * 10 + x % 10; x = x / 10 }`
- Negative numbers: immediately return "no"
- Compare reversed with original
```
fn is_palindrome(n: i64) -> i64 = {
    if n < 0 { 0 }
    else {
        let mut x = n; let mut rev = 0;
        while x > 0 { rev = rev * 10 + x % 10; x = x / 10 };
        if rev == n { 1 } else { 0 }
    }
};
fn main() -> i64 = {
    let t: i64 = read_int();
    for _i in 0..t {
        let n: i64 = read_int();
        if is_palindrome(n) == 1 { println_str("yes") }
        else { println_str("no") }
    };
    0
};
```
