# 231. Crawler Log Folder

The Leetcode file system keeps a log each time some user performs a change folder operation. The operations are:
- "../" : Move to the parent folder of the current folder. (If you are already in the main folder, remain in the same folder.)
- "./" : Remain in the same folder.
- "x/" : Move to the child folder named x (a string of lowercase English letters).

You are given a list of strings logs where logs[i] is the operation performed by the user at the ith step. Return the minimum number of operations needed to go back to the main folder after the change folder operations. (external, LeetCode #1598)

Input: first line n, then n strings (one per line)
Output: minimum steps to return to main folder

Example:
Input:
3
d1/
d2/
../
Output:
1

Explanation: After d1/ and d2/ we are 2 levels deep, after ../ we are 1 level deep. Need 1 more ../ to reach main.
