# 241. Reformat Phone Number

You are given a phone number as a string number. number consists of digits, spaces, and/or dashes. You would like to reformat the phone number in a certain manner. Firstly, remove all spaces and dashes. Then, group the digits from left to right into blocks of length 3 until there are 4 or fewer digits. The final digits are then grouped as follows:
- 2 digits: a single block of length 2.
- 3 digits: a single block of length 3.
- 4 digits: two blocks of length 2 each.

The blocks are then joined by dashes. Return the phone number after formatting. (external, LeetCode #1694)

Input: a single string (phone number)
Output: reformatted phone number

Example:
Input:
1-23-45 6
Output:
123-456

Example:
Input:
123 4-567
Output:
123-45-67
