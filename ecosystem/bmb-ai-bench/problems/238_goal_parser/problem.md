# 238. Goal Parser Interpretation

You own a Goal Parser that can interpret a string command. The command consists of "G", "()" and/or "(al)" in some order. Interpret the command as follows:
- "G" -> "G"
- "()" -> "o"
- "(al)" -> "al"

Return the Goal Parser's interpretation of command. (external, LeetCode #1678)

Input: a single string
Output: interpreted string

Example:
Input:
G()(al)
Output:
Goal
