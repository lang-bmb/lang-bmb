# bmb-algo — Node.js Bindings

High-performance algorithms compiled from [BMB](https://github.com/iyulab/lang-bmb), a contract-verified systems language. Node.js bindings via [koffi](https://koffi.dev/) FFI — no native build required.

## Status

**PoC** (Track T Node bindings, v0.1.0) — 24 functions implemented.

## Requirements

- Node.js ≥ 18
- `bmb_algo.dll` (Windows) / `libbmb_algo.so` (Linux) / `libbmb_algo.dylib` (macOS) — built from `ecosystem/bmb-algo/`

## Quick Start

```bash
# In ecosystem/bmb-algo/bindings/node/
npm install

# Build the DLL first (if not already built)
cd ../../..  # to lang-bmb root
python ecosystem/build_all.py --lib algo
cd ecosystem/bmb-algo/bindings/node
```

```javascript
const algo = require('bmb-algo');

// Number theory
algo.fibonacci(10)           // 55
algo.gcd(12, 8)              // 4
algo.prime_count(100)        // 25
algo.is_prime(17)            // true

// Dynamic Programming
algo.knapsack([2,3,4], [3,4,5], 7)       // 9
algo.lcs('abcde', 'ace')                  // 3
algo.edit_distance('kitten', 'sitting')   // 3
algo.max_subarray([-2,1,-3,4,-1,2,1])    // 6
algo.coin_change([1,5,10,25], 36)        // 3

// Arrays
algo.array_sum([1,2,3,4,5])        // 15
algo.binary_search([1,3,5,7], 5)   // 2
algo.is_sorted([1,2,3])            // true
```

## API

All functions return JavaScript `number` (int64 values ≤ 2^53). Boolean predicates (`is_prime`, `is_sorted`, `is_palindrome_num`) return `boolean`.

### Number Theory
| Function | Signature | Description |
|----------|-----------|-------------|
| `gcd` | `(a, b) → number` | Greatest common divisor |
| `lcm` | `(a, b) → number` | Least common multiple |
| `fibonacci` | `(n) → number` | n-th Fibonacci (0-indexed) |
| `prime_count` | `(n) → number` | Primes ≤ n |
| `modpow` | `(base, exp, mod) → number` | Modular exponentiation |
| `nqueens` | `(n) → number` | N-Queens solution count |
| `is_prime` | `(n) → boolean` | Primality test |

### Dynamic Programming
| Function | Signature | Description |
|----------|-----------|-------------|
| `knapsack` | `(weights[], values[], capacity) → number` | 0/1 Knapsack |
| `lcs` | `(a, b) → number` | Longest Common Subsequence |
| `edit_distance` | `(a, b) → number` | Levenshtein distance |
| `max_subarray` | `(arr[]) → number` | Maximum subarray sum |
| `coin_change` | `(coins[], amount) → number` | Minimum coins (-1 if impossible) |
| `lis` | `(arr[]) → number` | Longest Increasing Subsequence |

### Arrays
| Function | Signature | Description |
|----------|-----------|-------------|
| `array_sum` | `(arr[]) → number` | Sum of elements |
| `array_min` | `(arr[]) → number` | Minimum element |
| `array_max` | `(arr[]) → number` | Maximum element |
| `binary_search` | `(sortedArr[], target) → number` | Binary search (index or -1) |
| `is_sorted` | `(arr[]) → boolean` | Non-decreasing order check |

### Utility
| Function | Signature | Description |
|----------|-----------|-------------|
| `djb2_hash` | `(s) → number` | DJB2 string hash |
| `is_palindrome_num` | `(n) → boolean` | Palindrome number test |
| `digit_sum` | `(n) → number` | Sum of decimal digits |
| `bit_popcount` | `(n) → number` | Set bit count |

## Running Tests

```bash
npm test
# → 21/21 PASS
```

## FFI Architecture

```
JavaScript (Node.js)
    ↓ koffi FFI (no native build)
bmb_algo.dll / libbmb_algo.so
    ↓ BMB runtime
BMB compiled algorithms (LLVM IR → native)
```

Mirrors the Python binding (`ctypes`) approach for API and type convention parity.
