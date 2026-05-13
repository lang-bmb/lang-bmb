# Knapsack 01

0/1 Knapsack: maximize total value without exceeding capacity.

## Input
- First integer: n (number of items)
- Second integer: W (capacity)
- Next 2n integers: weight1 value1 weight2 value2 ... (pairs)

## Output
Maximum achievable value.

## Example
`3 50 10 60 20 100 30 120` -> items (w=10,v=60),(w=20,v=100),(w=30,v=120), cap=50
Best: items 1+2, weight=30, value=220
