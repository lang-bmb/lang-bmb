# Topological Sort

Output the lexicographically smallest topological ordering of a DAG.

## Input
- First integer: n (nodes 0..n-1)
- Second integer: m (number of directed edges)
- Next 2m integers: from1 to1 from2 to2 ...

## Output
Nodes in topological order, space-separated on one line. Use lexicographically smallest (BFS/Kahn with min-heap).

## Example
`4 4 0 1 0 2 1 3 2 3` -> `0 1 2 3`
`3 0` -> `0 1 2` (no edges, any valid order; output smallest: 0 1 2)
