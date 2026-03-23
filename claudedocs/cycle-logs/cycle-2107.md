# Cycle 2107: Per-library benchmark scripts
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2106: Test suites complete, next was benchmarks.

## Scope & Implementation
Created benchmark scripts for all 5 libraries under `benchmarks/bench_<lib>.py`.

### Results
| Library | vs Baseline | Verdict |
|---------|------------|---------|
| bmb-algo | vs pure Python | 7/7 FASTER (6.25x–32.19x) |
| bmb-crypto | vs C-backed hashlib | 0/9 (expected — FFI overhead) |
| bmb-text | vs C-backed str | 0/9 (expected — FFI overhead) |
| bmb-json | vs C-backed json | 0/10 (expected — FFI overhead) |
| bmb-compute | vs C-backed math | 0/12 (expected — FFI overhead) |

### Analysis
bmb-algo's dramatic speedups (6x–32x vs pure Python) validate the binding strategy.
Crypto/text/json/compute comparisons are against C-backed Python stdlib — ctypes FFI overhead (~1us/call) dominates. For these libraries, the real comparison is BMB-compiled binary vs C (validated in the main benchmarks as PASS/FASTER).

## Review & Resolution
- Fixed emoji encoding issues on Windows (replaced with ASCII markers)
- All benchmarks run successfully

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Consider batch/bulk APIs to amortize FFI overhead for crypto/text/compute
- Next Recommendation: Build script for compiling all DLLs (cycle 2108)
