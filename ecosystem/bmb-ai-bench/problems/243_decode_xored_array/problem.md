# 243. Decode XORed Array

There is a hidden integer array arr that consists of n + 1 non-negative integers. It was encoded into another integer array encoded of length n, such that encoded[i] = arr[i] XOR arr[i+1]. You are also given an integer first, that is the first element of arr, i.e. arr[0] = first. Given the encoded array and first, return the original array arr. (external, LeetCode #1720)

Input: first line is n, second line is first, then n integers (one per line)
Output: space-separated array elements

Example:
Input:
1
7
13
Output:
7 10

Example:
Input:
3
1
2
4
1
Output:
1 3 7 6
