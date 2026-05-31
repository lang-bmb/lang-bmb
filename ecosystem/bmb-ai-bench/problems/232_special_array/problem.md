# 232. Special Array With X Elements Greater Than or Equal X

You are given an array nums of non-negative integers. nums is considered special if there exists a number x such that there are exactly x numbers in nums that are greater than or equal to x.

Notice that x does not have to be an element in nums.

Return x if the array is special, otherwise, return -1. It can be proven that if nums is special, the value for x is unique. (external, LeetCode #1608)

Input: first line n, then n integers
Output: x or -1

Example:
Input:
4
3
5
0
3
Output:
2

Explanation: There are 2 numbers (3 and 3) that are >= 2. And 2 is not a number in the array, still x=2 works.
