# 212. Count Good Triplets

Given an array of integers arr, and three integers a, b and c. Return the number of good triplets. A triplet (arr[i], arr[j], arr[k]) is good if: 0 <= i < j < k < arr.length, |arr[i] - arr[j]| <= a, |arr[j] - arr[k]| <= b, |arr[i] - arr[k]| <= c. (external, LeetCode #1534)

Input: first line n, then n integers (one per line), then a, b, c (each on separate line)
Output: count of good triplets

Example:
Input:
3
0
0
0
0
0
0
Output: 1

Input:
4
1
1
2
2
0
0
1
Output: 0
