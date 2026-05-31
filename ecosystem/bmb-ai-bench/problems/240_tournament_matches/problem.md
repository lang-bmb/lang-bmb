# 240. Count of Matches in Tournament

You are given an integer n, the number of teams in a tournament. In each round, if n is even, n/2 matches are played and n/2 teams advance. If n is odd, (n-1)/2 matches are played, (n-1)/2 teams advance, and one team gets a bye.

Return the number of matches played in the tournament until a winner is decided. (external, LeetCode #1688)

Input: a single integer n
Output: number of matches

Example:
Input:
7
Output:
6

Explanation: Round 1: 3 matches, 3 advance + 1 bye. Round 2: 1 match, 1 advance + 1 bye. Round 3: 1 match, winner. Total: 5 matches. Wait, actually for n=7: matches=n-1=6.
