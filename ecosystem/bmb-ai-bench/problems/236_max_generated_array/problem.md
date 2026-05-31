# 236. Get Maximum in Generated Array

You are given an integer n. A 0-indexed integer array nums of length n+1 is generated in the following way:
- nums[0] = 0
- nums[1] = 1 (when n >= 1)
- nums[2 * i] = nums[i] when 2 <= 2*i <= n
- nums[2 * i + 1] = nums[i] + nums[i + 1] when 2 <= 2*i + 1 <= n

Return the maximum integer in the array nums. (external, LeetCode #1646)

Input: a single integer n
Output: maximum value in the generated array

Example:
Input:
7
Output:
3

Explanation: nums = [0,1,1,2,1,3,2,3], max is 3.
