# 200. Divisor Game

Alice and Bob take turns playing a game. Alice starts first. Initially, there is a number n on the chalkboard. On each player's turn, that player makes a move consisting of: choosing any x with 0 < x < n and n % x == 0, and replacing n with n - x. A player who cannot make a move loses. Return 1 if Alice wins, 0 if Bob wins. (external, LeetCode #1025)

Note: Alice wins if and only if n is even.

Input: a single integer n (1 <= n <= 1000)
Output: 1 if Alice wins, 0 if Bob wins

Example:
Input: 2
Output: 1

Input: 3
Output: 0

Input: 4
Output: 1
