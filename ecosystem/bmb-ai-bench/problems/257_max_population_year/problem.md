# 257. Maximum Population Year

You are given a 2D integer array logs where logs[i] = [birth_i, death_i] indicates the birth and death years of the ith person. The population of some year x is the number of people alive during that year. The ith person is alive during years [birth_i, death_i - 1]. Return the earliest year with the maximum population. (external, LeetCode #1854)

Input: first line is n, then n pairs of birth/death years (one per line each)
Output: earliest year with maximum population

Example:
Input:
2
1993
1999
2000
2010
Output:
1993

Example:
Input:
2
1950
1961
1960
1971
Output:
1960
