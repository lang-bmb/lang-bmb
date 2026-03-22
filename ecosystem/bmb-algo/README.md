# bmb-algo

High-performance algorithms powered by [BMB](https://github.com/iyulab/lang-bmb) — a language that beats C and Rust.

## Benchmarks

| Algorithm | bmb-algo | C (clang -O3) | Rust (--release) | vs C |
|-----------|----------|---------------|-----------------|------|
| knapsack | 163ms | 1121ms | 244ms | **6.8x faster** |
| lcs | 130ms | 230ms | 290ms | **1.8x faster** |
| floyd_warshall | 426ms | 593ms | 779ms | **1.4x faster** |
| spectral_norm | 104ms | 122ms | 123ms | **1.2x faster** |
| n_body | 79ms | 100ms | 94ms | **1.3x faster** |

## Install

```bash
pip install bmb-algo
```

## Usage

```python
import bmb_algo

# Dynamic Programming
result = bmb_algo.knapsack(weights=[2, 3, 4], values=[3, 4, 5], capacity=7)
# 9

distance = bmb_algo.edit_distance("kitten", "sitting")
# 3

length = bmb_algo.lcs("ABCBDAB", "BDCAB")
# 4

coins = bmb_algo.coin_change(coins=[1, 5, 11], amount=15)
# 3

sub = bmb_algo.max_subarray([-2, 1, -3, 4, -1, 2, 1, -5, 4])
# 6

length = bmb_algo.lis([10, 9, 2, 5, 3, 7, 101, 18])
# 4

# Graph Algorithms
dist = bmb_algo.floyd_warshall([[0, 3, 999], [2, 0, 999], [999, 7, 0]])
# [[0, 3, 999], [2, 0, 999], [9, 7, 0]]

shortest = bmb_algo.dijkstra([[0, 4, -1], [-1, 0, 2], [-1, -1, 0]], source=0)
# [0, 4, 6]
```

## How is this possible?

BMB is an AI-native systems language where compile-time contracts (`pre`/`post` conditions)
eliminate runtime overhead. The compiler *proves* correctness, then generates code faster
than hand-tuned C.

```bmb
@export
pub fn bmb_knapsack(weights: i64, values: i64, n: i64, capacity: i64) -> i64
  pre n > 0 and capacity >= 0
  post ret >= 0
= { ... };
```

The `pre` condition is verified at compile time. No runtime bounds checks, no safety overhead.
The result: **zero-cost safety + maximum performance**.

## Error Handling

bmb-algo uses BMB's FFI safety layer. Invalid inputs raise Python exceptions instead of crashing:

```python
try:
    bmb_algo.knapsack([], [], capacity=-1)
except RuntimeError as e:
    print(f"BMB error: {e}")
```

## License

MIT

## Links

- [BMB Language](https://github.com/iyulab/lang-bmb)
- [Benchmark Methodology](https://github.com/iyulab/lang-bmb/tree/main/ecosystem/benchmark-bmb)
