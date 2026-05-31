# 255. Replace All Digits with Characters

You are given a 0-indexed string s that has lowercase English letters in its even indices and digits in its odd indices. There is a function shift(c, x), where c is a character and x is a digit, that returns the xth character after c. Replace every digit in s with shift(s[i-1], s[i]) and return s after replacing all digits. (external, LeetCode #1844)

Input: a single string (alternating letter-digit)
Output: replaced string

Example:
Input:
a1c1e1
Output:
abcdef

Example:
Input:
a1b2c3d4e5
Output:
abbdcfdhej
