# 175. Minimum Moves to Equal Array Elements

Given an integer array of size n, find the minimum number of moves required to make all array elements equal, where a move is incrementing n-1 elements by 1. (external, LeetCode #453)

Note: Incrementing n-1 elements by 1 is equivalent to decrementing 1 element by 1. So the answer is sum - n * min.

Input:
n (array size)
n integers, one per line

Output: minimum number of moves

Example:
Input:
3
1
2
3
Output: 3
(moves: [1,2,3] -> [2,3,3] -> [3,4,3] -> [3,3,4] -> ... min=3 steps: sum=6, min=1, 6-3*1=3)

Input:
3
1
1
1
Output: 0
