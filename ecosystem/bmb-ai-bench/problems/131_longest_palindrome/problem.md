# 131. Longest Palindrome

LeetCode #409

## Problem

Given a string of letters, return the length of the longest palindrome that can be built using those characters.

## Input

- One line: the string

## Output

Length of the longest palindrome

## Examples

```
Input: abccccdd
Output: 7   (e.g., "dccaccd")
```

```
Input: a
Output: 1
```

## BMB Notes

Count character pairs: each pair contributes 2 to the length. If any character has an odd count, add 1 (center of palindrome).

```bmb
fn sum_pairs(s: String, c: i64, acc: i64, has_odd: i64) -> i64
= if c > 122 {
        if has_odd > 0 { acc + 1 } else { acc }
    }
    else {
        let cnt = count_char(s, c, 0, 0);
        sum_pairs(s, c + 1, acc + (cnt / 2) * 2, if cnt % 2 == 1 { 1 } else { has_odd })
    };
```
