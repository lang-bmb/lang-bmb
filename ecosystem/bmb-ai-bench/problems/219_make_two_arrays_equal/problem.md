# 219. Make Two Arrays Equal by Reversing Subarrays

You are given two integer arrays of equal length target and arr. In one step, you can select any non-empty subarray of arr and reverse it. You are allowed to make any number of steps. Return 1 if you can make arr equal to target, 0 otherwise. (external, LeetCode #1460)

Note: Two arrays are equal iff they have the same elements with the same frequencies (regardless of order, since we can reorder by reversal).

Input: first line n, then n integers for target (one per line), then n integers for arr (one per line)
Output: 1 if possible, 0 otherwise

Example:
Input:
3
1
2
3
3
2
1
Output:
1

Input:
3
1
2
3
3
2
2
Output:
0
