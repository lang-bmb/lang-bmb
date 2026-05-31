# 295. Best Poker Hand

Given 5 cards (ranks 1-13 and suits), return the best hand: "Flush" (all same suit), "Three of a Kind" (≥3 same rank), "Pair" (≥2 same rank), or "High Card". (external, LeetCode #2347)

Input: 5 rank integers (one per line), then 5 suit characters (one per line, single char a-d)
Output: the best hand name

Example:
Input:
13
2
3
1
9
a
a
a
a
a
Output:
Flush

Example:
Input:
4
4
2
4
4
d
a
b
c
d
Output:
Three of a Kind

Example:
Input:
10
10
2
12
9
a
b
c
d
a
Output:
Pair
