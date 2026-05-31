# 129. Happy Number

LeetCode #202

## Problem

Determine whether a number is "happy": repeatedly replace with the sum of squares of its digits. If the process eventually reaches 1, it's happy; if it loops endlessly, it's not.

## Input

- One line: the integer `n`

## Output

`true` or `false`

## Examples

```
Input: 19
Output: true
(19→82→68→100→1)
```

```
Input: 2
Output: false
(2→4→16→37→58→89→145→42→20→4→... cycles)
```

## BMB Notes

Use a step limit to detect cycles (100 steps is sufficient for all valid inputs):

```bmb
fn digit_sq_sum(n: i64) -> i64
= if n == 0 { 0 }
    else { let d = n % 10; d * d + digit_sq_sum(n / 10) };

fn is_happy(n: i64, steps: i64) -> bool
= if n == 1 { true }
    else if steps <= 0 { false }
    else { is_happy(digit_sq_sum(n), steps - 1) };
```
