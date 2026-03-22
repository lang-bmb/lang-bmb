# bmb-algo — Blazing Fast Algorithms

> 90x faster than Python on knapsack. 181x faster on N-Queens.

High-performance algorithms compiled from [BMB](https://github.com/iyulab/lang-bmb), a language where compile-time contracts eliminate runtime overhead.

## Benchmarks (vs Pure Python)

| Algorithm | bmb-algo | Python | Speedup |
|-----------|----------|--------|---------|
| knapsack(100 items) | 18 us | 1,664 us | **90.7x** |
| nqueens(8) | 50 us | 9,136 us | **181.6x** |
| prime_count(10k) | 10 us | 252 us | **25.6x** |
| fibonacci(50) | 0.2 us | 0.8 us | **3.4x** |

## Quick Start

```python
import bmb_algo

# Dynamic Programming
bmb_algo.knapsack([2, 3, 4], [3, 4, 5], 7)         # 9
bmb_algo.edit_distance("kitten", "sitting")          # 3
bmb_algo.coin_change([1, 5, 11], 15)                 # 3

# Graph
bmb_algo.dijkstra([[0,4,-1],[-1,0,2],[-1,-1,0]], 0) # [0, 4, 6]
bmb_algo.topological_sort(adj_matrix)                 # [0, 1, 2, 3]

# Sort
bmb_algo.merge_sort([5, 3, 1, 4, 2])                # [1, 2, 3, 4, 5]

# Number Theory
bmb_algo.nqueens(8)         # 92 solutions
bmb_algo.modpow(2, 10, 1000)  # 24
bmb_algo.prime_count(100)   # 25 primes
```

## 34 Algorithms

**DP**: knapsack, lcs, edit_distance, coin_change, lis, max_subarray
**Graph**: floyd_warshall, dijkstra, bfs_count, topological_sort
**Sort**: quicksort, merge_sort, heap_sort, counting_sort
**Search**: binary_search
**Number Theory**: gcd, lcm, fibonacci, prime_count, nqueens, modpow
**Utility**: djb2_hash, power_set_size, matrix_multiply, matrix_transpose, is_sorted, array_reverse, array_rotate, unique_count, prefix_sum, array_sum, array_min, array_max, bit_popcount

## How?

Written in [BMB](https://github.com/iyulab/lang-bmb) — compile-time contracts prove correctness, then generate code faster than hand-tuned C.
