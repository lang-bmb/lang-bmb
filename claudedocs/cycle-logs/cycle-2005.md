# Cycle 2005-2008: Competitive benchmarks + Python quality
Date: 2026-03-22

## Inherited -> Addressed
- Cycle 2001: No actionable carry-forward

## Scope & Implementation

### Python Package __init__.py
- Created __init__.py for all 4 libraries (bmb-algo, bmb-crypto, bmb-text, bmb-json)
- Enables `from bmb_algo import *` pattern

### Competitive Benchmark Script
- `ecosystem/benchmark_bindings.py`: 15 benchmarks across 4 libraries
- Compares BMB vs Pure Python, hashlib, str methods, json module

### Benchmark Results

**bmb-algo vs Pure Python** (massive wins on compute-heavy algorithms):
| Benchmark | BMB | Python | Speedup |
|-----------|-----|--------|---------|
| knapsack(100) | 18 us | 1664 us | **90.7x** |
| nqueens(8) | 50 us | 9136 us | **181.6x** |
| prime_count(10k) | 10 us | 252 us | **25.6x** |
| fibonacci(50) | 0.2 us | 0.8 us | **3.4x** |

**bmb-crypto/text/json vs C-extensions**: Slower (0.1-0.4x) due to FFI overhead.
Python's hashlib/str/json are all C code; BMB must cross ctypes boundary per call.
This is expected — BMB's value is in compute-heavy algorithms, not competing with C-extensions.

### Files created
- `ecosystem/bmb-algo/bindings/python/__init__.py`
- `ecosystem/bmb-crypto/bindings/python/__init__.py`
- `ecosystem/bmb-text/bindings/python/__init__.py`
- `ecosystem/bmb-json/bindings/python/__init__.py`
- `ecosystem/benchmark_bindings.py`

## Review & Resolution
- All benchmarks run successfully
- Results align with BMB's design philosophy: compute-bound algorithms dominate

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: FFI overhead makes per-call benchmarks unfavorable for small data
- Next Recommendation: bmb-compute library for numeric computation
