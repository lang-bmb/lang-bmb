# 229. Matrix Diagonal Sum

Given a square matrix mat, return the sum of the matrix diagonals. Only include elements of the primary diagonal (top-left to bottom-right) and secondary diagonal (top-right to bottom-left), but if the two diagonals share an element (i.e., n is odd), that element is counted only once. (external, LeetCode #1572)

Input: first line n, then n*n integers (row by row)
Output: the diagonal sum

Example:
Input:
3
1
2
3
4
5
6
7
8
9
Output:
25

Explanation: Primary diagonal: 1+5+9=15. Secondary: 3+5+7=15. Center 5 counted once. Total=25.
