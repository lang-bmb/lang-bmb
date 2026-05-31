# 176. Construct the Rectangle

For a web developer, it is very important to know the acceptable dimensions of a web page. Given a specific rectangular web page's area, your job is to design a rectangular web page that meets the following requirements:

1. The area of the rectangular web page is equal to the given target area.
2. The width W should not be greater than the length L, i.e., L >= W.
3. The difference between length and width should be as small as possible.

Return the dimensions as "L W". (external, LeetCode #492)

Input: area (integer)
Output: L W (space separated, L >= W, L*W=area, minimize L-W)

Example:
Input: 4
Output: 2 2

Input: 37
Output: 37 1

Input: 122122
Output: 427 286
