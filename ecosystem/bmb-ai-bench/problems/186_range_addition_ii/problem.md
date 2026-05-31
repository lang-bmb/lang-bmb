# 186. Range Addition II

Given an m x n matrix M initialized with zeros, and an array of operations ops where ops[i] = [ai, bi] means M[x][y] should be incremented by one for all 0 <= x < ai and 0 <= y < bi. Count the number of maximum integers in the matrix after performing all operations. (external, LeetCode #598)

Input: first line "m n k", then k lines each "r c"
Output: count of maximum integers

Example:
Input:
3 3 2
2 2
3 3
Output: 4

Explanation: After ops, max is 2 at positions [0][0],[0][1],[1][0],[1][1].

Input:
3 3 0
Output: 9

Note: If no operations, all zeros, count = m*n.
