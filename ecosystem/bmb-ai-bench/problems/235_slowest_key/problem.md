# 235. Slowest Key

A newly designed keypad has n keys. You are given a string keysPressed of length n where keysPressed[i] was the i-th key pressed, and releaseTimes[i] was the time the i-th key was released. The time to press the i-th key is releaseTimes[i] - releaseTimes[i-1] for i > 0, and releaseTimes[0] for i = 0.

Return the key that had the longest press duration. If multiple keys have the same duration, return the lexicographically largest key. (external, LeetCode #1629)

Input: first line n, then n integers (release times), then the string of keys pressed
Output: single character

Example:
Input:
3
9
29
49
cbcd
Output:
c

Explanation: c pressed 9s, b pressed 20s, c again pressed 20s, d pressed 20s. c and b and d all pressed 20s, but 'd' > 'c' > 'b', so return 'd'.

Wait, example shows 'c' but 'd' should win. Let me correct: b=20,c=20,d=20 → all equal → return lexicographically largest = 'd'.
Actually the original example: releaseTimes=[9,29,49,50], keysPressed="cbcd", durations: c=9, b=20, c=20, d=1. longest is 20 (b and c), return 'c' (larger).
