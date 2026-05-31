# 221. Decompress Run-Length Encoded List

A run-length encoded list is a list of pairs [freq, val] meaning val is repeated freq times. Given the encoded list (2n numbers), return the decompressed list. (external, LeetCode #1313)

Input: first line n (number of pairs), then 2n integers (freq and val alternating, one per line)
Output: decompressed list elements (one per line)

Example:
Input:
2
1
2
3
4
Output:
2
4
4
4

Input:
3
1
1
2
2
3
3
Output:
1
2
2
3
3
3
