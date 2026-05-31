# 122. Reverse String

LeetCode #344

## Problem

Given a string, return the reversed string.

## Input

- One line: the string to reverse

## Output

The reversed string

## Examples

```
Input: hello
Output: olleh
```

```
Input: Hannah
Output: hannaH
```

## BMB Notes

Use the built-in `str_reverse` function:

```bmb
fn main() -> i64 = {
    let s = read_line();
    println_str(str_reverse(s));
    0
};
```
