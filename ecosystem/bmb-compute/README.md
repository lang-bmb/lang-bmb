# bmb-compute — Numeric Computation

Math, statistics, random numbers, and vector operations compiled from [BMB](https://github.com/iyulab/lang-bmb).

## Installation

```bash
pip install bmb-compute
```

## Quick Start

```python
import bmb_compute

# Math
bmb_compute.sqrt(144)           # 12
bmb_compute.factorial(10)       # 3628800
bmb_compute.ipow(2, 20)         # 1048576
bmb_compute.clamp(15, 1, 10)    # 10

# Statistics
bmb_compute.sum([10, 20, 30])          # 60
bmb_compute.mean_scaled([10, 20, 30])  # 20000 (= 20.000)
bmb_compute.min_val([5, 3, 8, 1])      # 1
bmb_compute.range_val([10, 50])        # 40

# Vector
bmb_compute.dot_product([1, 2, 3], [4, 5, 6])  # 32
bmb_compute.dist_squared([0, 0], [3, 4])        # 25

# Utility
bmb_compute.is_power_of_two(8)      # True
bmb_compute.next_power_of_two(5)    # 8
```

## Full API (25 functions)

### Math
| Function | Description |
|----------|-------------|
| `abs(x)` | Absolute value |
| `min(a, b)` / `max(a, b)` | Minimum / maximum |
| `clamp(x, lo, hi)` | Clamp to range |
| `sign(x)` | Sign (-1, 0, 1) |
| `ipow(base, exp)` | Integer power |
| `sqrt(n)` | Integer square root |
| `factorial(n)` | Factorial (up to 20!) |

### Statistics
| Function | Description |
|----------|-------------|
| `sum(arr)` | Sum of elements |
| `mean_scaled(arr)` | Mean x 1000 |
| `min_val(arr)` / `max_val(arr)` | Min/max of array |
| `range_val(arr)` | Range (max - min) |
| `variance_scaled(arr)` | Variance x 1000000 |

### Random (XorShift64*)
| Function | Description |
|----------|-------------|
| `rand_seed(seed)` | Initialize PRNG |
| `rand_next(state)` | Next state |
| `rand_pos(state)` | Positive random value |
| `rand_range(state, max)` | Random in [0, max) |

### Vector
| Function | Description |
|----------|-------------|
| `dot_product(a, b)` | Dot product |
| `dist_squared(a, b)` | Euclidean distance squared |
| `weighted_sum(values, weights)` | Weighted sum |
| `lerp_scaled(a, b, t)` | Linear interpolation (t: 0-1000) |

### Utility
| Function | Description |
|----------|-------------|
| `is_power_of_two(n)` | Power of two check |
| `next_power_of_two(n)` | Next power of two >= n |

## How?

Written in [BMB](https://github.com/iyulab/lang-bmb) — compile-time contracts prove correctness, then generate code faster than hand-tuned C.

## License

MIT
