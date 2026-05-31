# 223. Number of Students Doing Homework at a Given Time

Given n students with start[i] and end[i] times, and a query time, return the number of students doing homework at query time t (inclusive). Student is doing homework if start[i] <= t <= end[i]. (external, LeetCode #1450)

Input: first line n, then n start times (one per line), then n end times (one per line), then query time t
Output: count of students active at time t

Example:
Input:
3
4
6
10
5
8
12
4
Output:
1

Input:
1
4
4
4
Output:
1
