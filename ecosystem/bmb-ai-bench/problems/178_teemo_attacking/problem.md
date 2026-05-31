# 178. Teemo Attacking

Our hero Teemo attacks Ashe with a poison attack every time a certain number of seconds. When Teemo attacks at time t, the poison lasts from t to t + duration - 1 seconds. Return the total number of seconds during which Ashe is poisoned. (external, LeetCode #495)

Input:
n (number of attacks)
duration
n attack times, one per line (sorted ascending)

Output: total poisoned seconds

Example:
Input:
2
2
1
4
Output: 4
(poisoned [1,2] and [4,5] = 4 seconds)

Input:
3
5
1
2
3
Output: 7
(attacks at 1,2,3: [1..5] union [2..6] union [3..7] = [1..7] = 7 seconds)
