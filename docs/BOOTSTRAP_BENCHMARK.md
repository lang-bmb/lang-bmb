# BMB Bootstrap + Benchmark Cycle

This document describes the Bootstrap + Benchmark cycle system for ensuring compiler correctness and preventing performance regressions.

## Overview

The system provides:

1. **3-Stage Bootstrap Verification** - Ensures the compiler can compile itself correctly
2. **Tiered Benchmark Suite** - Comprehensive performance testing across multiple categories
3. **Regression Detection** - Automatic detection of performance regressions with configurable thresholds
4. **CI/CD Integration** - GitHub Actions workflows for automated verification

## Quick Start

### Local Development

```bash
# Quick check (fast feedback loop)
./scripts/quick-check.sh

# Full verification before PR
./scripts/full-cycle.sh
```

### Scripts

| Script | Purpose | Typical Time |
|--------|---------|--------------|
| `quick-check.sh` | Fast local verification | ~2 min |
| `full-cycle.sh` | Complete pre-PR verification | ~15 min |
| `bootstrap.sh` | 3-stage bootstrap only | ~5 min |
| `benchmark.sh` | Benchmark suite only | ~10 min |
| `compare.py` | Compare benchmark results | <1 sec |
| `perf-analyze.sh` | Detailed performance analysis | Varies |

## 3-Stage Bootstrap

The bootstrap process verifies compiler correctness through self-compilation:

```
Stage 0 (Rust)  →  Stage 1 (BMB₁)  →  Stage 2 (BMB₂)  →  Stage 3 (BMB₃)
    ↓                   ↓                  ↓                   ↓
 Rust BMB         Compiles          Compiles            Compiles
 compiler         bootstrap          bootstrap           bootstrap
                     ↓                   ↓                   ↓
                 BMB₁ binary        BMB₂ LLVM IR       BMB₃ LLVM IR
                                         ↓                   ↓
                                    Must be identical (fixed point)
```

### Running Bootstrap

```bash
# Full 3-stage bootstrap
./scripts/bootstrap.sh

# Stage 1 only (fast check)
./scripts/bootstrap.sh --stage1-only

# JSON output for CI
./scripts/bootstrap.sh --json

# Verbose output
./scripts/bootstrap.sh --verbose
```

## Benchmark Tiers

| Tier | Name | Benchmarks | Threshold | Blocking |
|------|------|------------|-----------|----------|
| 0 | Bootstrap | lexer, parser, types, etc. | 5% | No |
| 1 | Core Compute | fibonacci, mandelbrot, sieve, etc. | **2%** | **Yes** |
| 2 | Contract Features | bounds_check, null_check, etc. | 5% | No |
| 3 | Real World | json_parse, sorting, etc. | 5% | No |

### Running Benchmarks

```bash
# All benchmarks
./scripts/benchmark.sh --tier all

# Specific tier
./scripts/benchmark.sh --tier 1

# More runs for stability
./scripts/benchmark.sh --runs 10

# JSON output
./scripts/benchmark.sh --json --output results.json

# List available benchmarks
./scripts/benchmark.sh --list
```

## Regression Detection

The `compare.py` script compares benchmark results:

```bash
# Compare baseline vs current
python3 scripts/compare.py baseline.json current.json

# Strict mode (fail on Tier 1 regression)
python3 scripts/compare.py baseline.json current.json --tier1-strict

# Custom thresholds
python3 scripts/compare.py baseline.json current.json \
  --tier1-threshold 2 \
  --threshold 5

# CI mode (GitHub Actions annotations)
python3 scripts/compare.py baseline.json current.json --ci

# JSON output
python3 scripts/compare.py baseline.json current.json --json
```

### Regression Policy

| Tier | Threshold | Action |
|------|-----------|--------|
| Tier 0 | 5% | Warning |
| **Tier 1** | **2%** | **Build Failure** |
| Tier 2 | 5% | Warning |
| Tier 3 | 5% | Warning |

## CI Workflows

### `bootstrap-benchmark.yml` (Primary)

Runs on every PR and push to main:

1. **test** - Build and run tests (all platforms)
2. **bootstrap** - 3-stage bootstrap verification
3. **benchmark** - Run benchmark suite and compare
4. **performance-gate** - Check 2% regression threshold

### `benchmark-baseline.yml`

Updates baseline when main branch changes.

### `nightly-bench.yml`

Comprehensive nightly benchmark run with:
- 20 runs per benchmark (higher precision)
- Memory profiling
- Trend analysis
- Automatic issue creation on regression

## Performance Analysis

For detailed analysis of specific benchmarks:

```bash
# Basic analysis
./scripts/perf-analyze.sh fibonacci

# With CPU profiling (Linux)
./scripts/perf-analyze.sh fibonacci --profile

# With memory profiling
./scripts/perf-analyze.sh fibonacci --memory

# Compare LLVM IR
./scripts/perf-analyze.sh fibonacci --compare-ir

# Generate assembly
./scripts/perf-analyze.sh fibonacci --asm

# All analysis
./scripts/perf-analyze.sh fibonacci --profile --memory --compare-ir --asm
```

## Environment Setup

### Prerequisites

- Rust (stable)
- LLVM 21+ (for native compilation)
- GCC (for C baselines)
- Python 3.x
- bc (for calculations)

### Setup Script

```bash
# Verify environment
source scripts/ci/setup-env.sh --verify

# Install dependencies (Linux)
source scripts/ci/setup-env.sh --install-deps

# Install LLVM
source scripts/ci/setup-env.sh --install-llvm
```

## File Structure

```
scripts/
├── bootstrap.sh              # 3-stage bootstrap
├── benchmark.sh              # Benchmark runner
├── compare.py                # Regression detection
├── quick-check.sh            # Fast local check
├── full-cycle.sh             # Complete verification
├── perf-analyze.sh           # Performance analysis
└── ci/
    ├── setup-env.sh          # Environment setup
    └── report-results.sh     # CI report generation

.github/workflows/
├── bootstrap-benchmark.yml   # Primary CI workflow
├── benchmark-baseline.yml    # Baseline updates
└── nightly-bench.yml         # Nightly runs
```

## Development Workflow

### Before Making Changes

```bash
# Create baseline
./scripts/benchmark.sh --output .baseline.json
```

### During Development

```bash
# Quick verification after changes
./scripts/quick-check.sh
```

### Before Creating PR

```bash
# Full verification
./scripts/full-cycle.sh --baseline .baseline.json

# If passed, create PR
# CI will run automatically
```

### Investigating Regressions

```bash
# Identify which benchmark regressed
python3 scripts/compare.py .baseline.json current.json

# Analyze specific benchmark
./scripts/perf-analyze.sh <benchmark-name> --profile --compare-ir
```

## Troubleshooting

### Bootstrap Fails at Stage 1

```bash
# Check BMB compiler
cargo build --release --features llvm

# Verify bootstrap source
./target/release/bmb check bootstrap/compiler.bmb
```

### Fixed Point Not Reached

The compiler produces different output when self-compiling. This usually indicates:
- Non-deterministic code generation
- Timestamp or path embedding
- Different optimization decisions

```bash
# Compare Stage 2 and Stage 3 IR
diff target/bootstrap/bmb-stage2.ll target/bootstrap/bmb-stage3.ll
```

### Benchmark Build Failures

```bash
# Check individual benchmark
./target/release/bmb build ecosystem/benchmark-bmb/benches/compute/<name>/bmb/main.bmb
```

## References

- Ken Thompson, "Reflections on Trusting Trust" (1984)
- BMB Performance Gates in ROADMAP.md
- CLAUDE.md for development principles
