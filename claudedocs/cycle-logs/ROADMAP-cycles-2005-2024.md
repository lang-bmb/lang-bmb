# Roadmap: Cycles 2005-2024 — Benchmarks + bmb-compute + quality
Date: 2026-03-22

## Phase 1: Competitive benchmarks + Python quality (2005-2008)
- Benchmark scripts: bmb-algo vs scipy, bmb-crypto vs hashlib at scale
- Python __init__.py for all libraries (proper package imports)
- Error handling: BmbError exception class
- Edge case tests

## Phase 2: bmb-compute library (2009-2012)
- Numeric computation from benchmark-bmb: spectral_norm, matrix chain, fibonacci large
- Integration from gotgan-packages: bmb-math, bmb-matrix, bmb-statistics

## Phase 3: More gotgan integration (2013-2016)
- bmb-rand: PRNG (XorShift64*)
- bmb-bitset: bit operations
- bmb-statistics: mean, median, stddev, variance

## Phase 4: Quality pass (2017-2020)
- Edge case testing for all functions
- Memory safety review
- Python API consistency

## Phase 5: Final automation (2021-2024)
- PowerShell build script (Windows native)
- Cross-library benchmark report
- ROADMAP final update
