# BMB — Bare-Metal-Banter

A contract-verified systems programming language that compiles to native code via LLVM.

[![Version](https://img.shields.io/badge/version-0.93.32-blue.svg)](docs/ROADMAP.md)
[![Bootstrap](https://img.shields.io/badge/bootstrap-3--stage%20fixed%20point-green.svg)](docs/BOOTSTRAP_BENCHMARK.md)
[![Tests](https://img.shields.io/badge/tests-5234%20passed-brightgreen.svg)](bmb/src)
[![Golden Tests](https://img.shields.io/badge/golden%20tests-69%2F69-brightgreen.svg)](tests/bootstrap)

---

## Hello, BMB

```bmb
fn main() -> i64 = {
    println("Hello, BMB!");
    0
};
```

```bash
bmb run hello.bmb
# Hello, BMB!
```

---

## What Makes BMB Different

BMB pursues **maximum performance through compile-time proofs**. Safety is not a separate goal — it's a natural consequence of proving everything at compile time.

```bmb
fn get(arr: &[i64], idx: i64) -> i64
  pre idx >= 0
  pre idx < arr.len()
= arr[idx];
```

The `pre` conditions are verified at compile time by an SMT solver (Z3). At runtime, they generate **zero overhead** — no bounds checks, no null checks, nothing. The proof happens before execution.

| Approach | Safety Method | Runtime Cost |
|----------|--------------|-------------|
| C | None (programmer responsibility) | 0% |
| Rust | Ownership + borrow checker | 0% (most cases) |
| Go/Java | Runtime checks (GC, bounds) | >0% |
| **BMB** | Compile-time contract proofs | **0%** |

---

## Performance

BMB targets C/Rust-level performance. All claims are measured, not assumed.

**Tier 1 benchmarks vs Clang -O3** (67 benchmarks, v0.93):

| Category | Result |
|----------|--------|
| FASTER than C | 5 benchmarks |
| PASS (within 2%) | 5 benchmarks |
| All within 10% | 67/67 benchmarks |

Representative results:

| Benchmark | BMB/Clang | Notes |
|-----------|-----------|-------|
| fasta | 0.94x | BMB faster |
| gcd | 0.97x | BMB faster |
| binary_trees | 0.99x | BMB faster |
| spectral_norm | 1.00x | Parity |
| mandelbrot | 1.01x | Identical IR |
| sieve | 1.07x | Residual gap (under investigation) |

BMB and Clang both use the LLVM backend. When BMB generates equivalent IR, the performance is identical. Remaining gaps are in BMB's IR generation, not in LLVM.

See [Benchmark Details](docs/BENCHMARK.md) for full results with methodology.

---

## Language Features

### Contracts

```bmb
fn binary_search(arr: &[i64], target: i64) -> i64
  pre is_sorted(arr)
  post ret == -1 || (ret >= 0 && ret < arr.len())
  post ret != -1 implies arr[ret] == target
= {
    // implementation
};
```

Contracts are checked by Z3 at compile time. Pass an unsorted array? **Compile error.**

### Explicit Overflow Semantics

```bmb
let a = x + y;      // requires contract proving no overflow
let b = x +% y;     // wrapping (mod 2^n)
let c = x +| y;     // saturating (clamp to bounds)
let d = x +? y;     // checked (returns T?)
```

### Pure Functions

```bmb
pure fn square(x: i64) -> i64 = x * x;
```

Compiler-guaranteed: no side effects, deterministic. Enables memoization, reordering, parallelization.

### Refinement Types

```bmb
type NonZero = i64 where self != 0;
type Percentage = f64 where self >= 0.0 and self <= 100.0;
```

### Concurrency Primitives

```bmb
let handle = thread_spawn(|| compute());
let result = thread_join(handle);

let m = mutex_new(0);
let ch = channel_new();
```

Thread, Mutex, Channel, RwLock, Barrier, async/await, ThreadPool, Scoped Threads — all built-in.

---

## Self-Hosting

BMB compiles itself. The bootstrap compiler (`bootstrap/compiler.bmb`, 32K LOC) achieves a **3-stage fixed point**:

```
Rust compiler → Stage 1 (BMB₁)
BMB₁ compiles bootstrap → Stage 2 (BMB₂)
BMB₂ compiles bootstrap → Stage 3 (BMB₃)
Verified: Stage 2 IR == Stage 3 IR ✅
```

A [golden binary](golden/) enables building BMB without Rust.

---

## Quick Start

```bash
bmb run examples/hello.bmb        # run
bmb check examples/simple.bmb     # type check
bmb verify examples/contracts.bmb # prove contracts (requires Z3)
bmb build examples/hello.bmb -o hello  # compile native (requires LLVM)
```

## Building BMB

### Option 1: Golden Binary (No Rust Required)

```bash
git clone https://github.com/iyulab/lang-bmb.git
cd lang-bmb
./scripts/golden-bootstrap.sh        # builds bmb-stage1
```

**Requirements**: LLVM 21+ only (`opt`, `clang` or `llc` + `gcc`)

### Option 2: From Source with Rust

```bash
cargo build --release --features llvm --target x86_64-pc-windows-gnu
```

**Requirements**: Rust 1.75+, LLVM 21+

See [Building from Source](docs/BUILD_FROM_SOURCE.md) for details.

---

## When to Use BMB

| Use Case | BMB Fit | Notes |
|----------|---------|-------|
| Performance-critical numeric computation | Good | C-level performance with compile-time safety |
| Safety-critical systems (avionics, medical) | Good | Contract verification eliminates runtime checks |
| AI-generated code pipelines | Experimental | Explicit syntax suits code generation |
| General application development | Not yet | Ecosystem still growing |
| Rapid prototyping | No | Use Python/TypeScript instead |

### Current Limitations

- **Ecosystem**: ~14 packages. No large standard library yet.
- **Community**: Early stage. Contributions welcome.
- **Tooling**: VS Code extension available. LSP basic.
- **Platforms**: Windows x64 primary. Linux/macOS golden binaries planned.

---

## Design Philosophy

BMB's direction is opposite to Rust:

| Language | Primary Goal | Method | Consequence |
|----------|-------------|--------|------------|
| Rust | Memory Safety | Ownership + Borrow Checker | Good performance |
| **BMB** | **Performance** | **Compile-time proofs** | **Safety guaranteed** |

BMB was designed with AI code generation in mind. The verbose, explicit syntax that makes languages hard for humans makes them precise for AI. But this is a hypothesis under validation — BMB is an experimental language exploring this design space.

---

## Ecosystem

| Tool | Purpose |
|------|---------|
| [gotgan](ecosystem/gotgan) | Package manager |
| [vscode-bmb](ecosystem/vscode-bmb) | VS Code extension |
| [tree-sitter-bmb](ecosystem/tree-sitter-bmb) | Syntax highlighting |
| [playground](ecosystem/playground) | Online editor |
| [benchmark-bmb](ecosystem/benchmark-bmb) | Performance test suite |

---

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](docs/tutorials/GETTING_STARTED.md) | Tutorial |
| [Language Reference](docs/LANGUAGE_REFERENCE.md) | Complete feature guide |
| [Specification](docs/SPECIFICATION.md) | Formal language definition |
| [Architecture](docs/ARCHITECTURE.md) | Compiler internals |
| [Build from Source](docs/BUILD_FROM_SOURCE.md) | Build instructions |
| [Benchmark](docs/BENCHMARK.md) | Performance methodology and results |
| [Contributing](docs/CONTRIBUTING.md) | How to contribute |
| [Target Users](docs/TARGET_USERS.md) | Who BMB is for |
| [Roadmap](docs/ROADMAP.md) | Development roadmap |

---

## Status

BMB is an **experimental language** in active development (v0.93). The compiler works, benchmarks are competitive with C, and the bootstrap is self-verifying. However, the ecosystem is young and the community is small.

If you're interested in contract-verified systems programming, formal methods, or AI-assisted code generation — we'd love your feedback.

---

## License

MIT

---

<p align="center">
  <b>Performance > Everything</b><br>
  <sub>Safety is not a goal — it's a consequence of pursuing maximum performance through compile-time proofs.</sub>
</p>
