# 204. Minimum Cost to Move Chips to Same Position

We have n chips at various positions. Move all chips to one position. Moving a chip 2 steps costs 0. Moving a chip 1 step costs 1. Return the minimum cost to move all chips to the same position. (external, LeetCode #1217)

Note: chips at even positions can be moved to any even position for free, and similarly for odd positions. The cost is min(count of chips at odd positions, count of chips at even positions).

Input: first line n, then n integers (chip positions, one per line)
Output: minimum cost

Example:
Input:
3
1
2
3
Output: 1

Input:
2
2
2
Output: 0

Input:
3
1
1000000000
1000000000
Output: 1
