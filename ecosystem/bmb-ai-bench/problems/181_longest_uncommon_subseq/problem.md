# 181. Longest Uncommon Subsequence I

Given two strings a and b, return the length of the longest uncommon subsequence between a and b. If the longest uncommon subsequence doesn't exist, return -1. (external, LeetCode #521)

An uncommon subsequence is a string that is a subsequence of one string but not a subsequence of the other. The longest uncommon subsequence is the longest string that satisfies this rule.

Key insight: If a != b, the answer is max(len(a), len(b)). If a == b, there is no uncommon subsequence, return -1.

Input:
a
b

Output: length of longest uncommon subsequence, or -1

Example:
Input:
aba
cdc
Output: 3

Input:
aaa
bbb
Output: 3

Input:
aaa
aaa
Output: -1
