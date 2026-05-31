# 222. Maximum Nesting Depth of the Parentheses

A string is a valid parentheses string (VPS) if it is a sequence of "(", ")" pairs properly nested. The depth of a VPS is the maximum number of opening parentheses before any character. Given a VPS s, return its depth. (external, LeetCode #1614)

Input: a single line containing the string s (only digits, '+', '-', '*', '/', '(', ')')
Output: the maximum nesting depth

Example:
Input:
(1+(2*3)+((8)/4))+1
Output:
3

Input:
(1)+((2))+(((3)))
Output:
3
