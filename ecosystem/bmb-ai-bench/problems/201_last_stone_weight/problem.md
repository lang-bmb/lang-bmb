# 201. Last Stone Weight

We have a collection of stones, each stone has a positive integer weight. Each turn, we choose the two heaviest stones and smash them together. Suppose the stones have weights x and y with x <= y. The result of this smash is: if x == y, both stones are destroyed; if x != y, x is destroyed and y has new weight y-x. Return the weight of the last remaining stone, or 0 if there are no stones left. (external, LeetCode #1046)

Input: first line n, then n integers (one per line)
Output: weight of last stone, or 0

Example:
Input:
6
2
7
4
1
8
1
Output: 1

Input:
1
1
Output: 1
