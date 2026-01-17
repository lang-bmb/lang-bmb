# BMB - Bare-Metal-Banter

A contract-verified systems programming language.

**Hard to write. Hard to get wrong. And that's what AI prefers.**

## Why BMB?

Traditional languages prioritize human convenience—readable syntax, flexible conventions, implicit behaviors. The cost: ambiguity, guesswork, runtime surprises.

BMB takes a different approach. Contracts are mandatory. Specifications are explicit. Invariants are verified at compile time.

```bmb
fn binary_search(arr: &[i64], target: i64) -> i64
  pre is_sorted(arr)
  post ret == -1 || (0 <= ret && ret < len(arr))
  post ret != -1 implies arr[ret] == target
= {
    var lo = 0;
    var hi = len(arr) - 1;
    while lo <= hi
      invariant 0 <= lo && hi < len(arr)
    {
        let mid = lo + (hi - lo) / 2;
        if arr[mid] == target { mid }
        else if arr[mid] < target { lo = mid + 1; }
        else { hi = mid - 1; }
    };
    -1
};
```

## Priorities

| Priority | Principle |
|----------|-----------|
| **P0** | **Performance** — No syntax that constrains optimization. Target: exceed C/Rust. |
| **P0** | **Correctness** — If it can be verified at compile time, it must be. |

## Quick Start

```bash
# Build
cargo build --release

# Run
bmb run examples/hello.bmb

# Type check
bmb check examples/simple.bmb

# Contract verification (requires Z3)
bmb verify examples/contracts.bmb

# Native compile (requires LLVM)
bmb build examples/hello.bmb -o hello

# REPL
bmb repl
```

## Current Status: v0.46 (Independence)

| Category | Status |
|----------|--------|
| Language Core | ✅ Complete |
| Type System | ✅ Complete |
| Contract System | ✅ Complete |
| Bootstrap Compiler | ✅ 30K LOC |
| Test Suite | ✅ 1,753+ tests |
| Documentation | ✅ Complete |
| CI/CD | ✅ Complete |
| **Performance** | ✅ 0.89x-0.99x vs C |
| **Self-Compile** | ✅ 0.56s |
| **v1.0.0-beta** | 🎯 Target |

## Features

### Completed

- **Types**: i8-i128, u8-u128, f64, bool, char, String
- **Generics**: `<T>`, `<K, V>`, bounds, where clauses
- **Contracts**: `pre`, `post`, `invariant`, `where`, `pure`, `@trust`
- **Control Flow**: if-else, match, while, for-in, loop
- **Operators**: Arithmetic, overflow-safe (`+%`, `+|`, `+?`), bitwise (`band`, `bor`), shift (`<<`, `>>`)
- **Collections**: Vec, Box, HashMap (stdlib)
- **Tooling**: Package manager (gotgan), VS Code, formatter, LSP

### In Progress

- 3-Stage self-hosting verification (WSL)
- Performance Gate #3.2, #3.3 (Benchmarks Game, Contract optimization)
- Ecosystem packages (14+ target, 12 complete)

## Project Structure

```
lang-bmb/
├── bmb/           # Rust compiler (being replaced)
├── bootstrap/     # Self-hosted BMB compiler (30K LOC)
├── stdlib/        # Standard library
├── examples/      # Example programs
├── ecosystem/     # Tools & extensions
│   ├── gotgan/           # Package manager
│   ├── vscode-bmb/       # VS Code extension
│   ├── tree-sitter-bmb/  # Syntax highlighting
│   ├── playground/       # Online editor
│   └── benchmark-bmb/    # Performance suite
└── docs/          # Documentation
```

## Requirements

| Requirement | Purpose | Required |
|-------------|---------|----------|
| Rust 1.70+ | Build compiler | Yes (until v0.45) |
| LLVM 21+ | Native codegen | Optional |
| Z3 | Contract verification | Optional |

## Documentation

| Document | Description |
|----------|-------------|
| [SPECIFICATION.md](docs/SPECIFICATION.md) | Language specification |
| [LANGUAGE_REFERENCE.md](docs/LANGUAGE_REFERENCE.md) | Complete reference |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Compiler internals |
| [ROADMAP.md](docs/ROADMAP.md) | Development roadmap |
| [API_STABILITY.md](docs/API_STABILITY.md) | API guarantees |
| [BENCHMARK_COMPARISON.md](docs/BENCHMARK_COMPARISON.md) | C/Rust/BMB performance |
| [tutorials/](docs/tutorials/) | Getting started guides |

## Roadmap to v1.0.0-beta

```
v0.46 (Current) ─── Independence (3-Stage verification)
     │
v0.47 ─────────── Performance Gates (C parity verified)
     │
v0.48 ─────────── Ecosystem (14+ core packages)
     │
v0.49 ─────────── Samples & scenarios
     │
v0.50 ─────────── Final verification
     │
v1.0.0-beta ───── Complete programming language ★
```

See [ROADMAP.md](docs/ROADMAP.md) for detailed phases.

## Performance

BMB matches or exceeds C/Rust performance on compute-intensive workloads:

```
                     C         Rust      BMB       Winner
─────────────────────────────────────────────────────────
fibonacci(45)        1.65s     1.66s     1.63s     ★ BMB (0.99x)
fibonacci(40)        177ms     180ms     150ms     ★ BMB (0.85x)
mandelbrot           42ms      42ms      39ms      ★ BMB (0.93x)
spectral_norm        44ms      44ms      39ms      ★ BMB (0.89x)
self-compile         -         -         0.56s     ✅ (30K LOC)
```

See [BENCHMARK_COMPARISON.md](docs/BENCHMARK_COMPARISON.md) for detailed methodology and results.

## License

MIT
