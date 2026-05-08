# BMB — Bare-Metal-Banter

A contract-verified systems programming language that compiles to native code via LLVM.

[![Version](https://img.shields.io/badge/version-0.98-blue.svg)](docs/ROADMAP.md)
[![Bootstrap](https://img.shields.io/badge/bootstrap-3--stage%20fixed%20point-green.svg)](docs/BOOTSTRAP_BENCHMARK.md)
[![Tests](https://img.shields.io/badge/tests-6201%20passed-brightgreen.svg)](bmb/src)
[![Golden Tests](https://img.shields.io/badge/golden%20tests-2814%2F2815-brightgreen.svg)](tests/bootstrap)

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

BMB targets parity with C and Rust. All claims are measured, not assumed.

**Tier 1 benchmarks vs Clang -O3** (16 workloads, v0.98 strict gate):

| Category | Result |
|----------|--------|
| Within 5% of Clang -O3 | 16/16 |
| Within measurement noise (≤2%) | 10/16 |
| All within 10% | 16/16 |

Representative results (BMB / Clang -O3 ratio, lower is faster):

| Benchmark | Ratio | Reading |
|-----------|-------|---------|
| fasta | 0.94x | Within run-to-run noise of Clang |
| gcd | 0.97x | Within run-to-run noise of Clang |
| binary_trees | 0.99x | Equivalent to Clang |
| spectral_norm | 1.00x | Equivalent to Clang |
| mandelbrot | 1.01x | Equivalent to Clang (identical IR) |
| sieve | 1.04x | Within 5% gate |

These are not claims of "beating C" — typical run-to-run variance is 1–3%, so a 0.94x ratio is statistically equivalent to parity. The honest claim is: **BMB matches Clang -O3 because both target the same LLVM backend, and BMB generates IR of comparable quality.**

Where BMB still trails (sieve, json_parse) we report it and investigate. Where the gap is LLVM-inherent (e.g., GCC outperforms both Clang and BMB on a workload), we document it as a backend limit, not a BMB limit.

See [Benchmark Details](docs/BENCHMARK.md) for full methodology, raw numbers, and noise analysis.

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

## Memory Model

BMB has **no garbage collector and no reference counting**. Memory is managed at compile time via Rust-style ownership, with raw pointers available for systems work.

| Mechanism | Syntax | Use |
|-----------|--------|-----|
| Owned value | `T`, `own T` | Single owner; freed at scope exit |
| Immutable borrow | `&T` | Multiple readers, no writers |
| Mutable borrow | `&mut T` | Single writer, no readers (XOR rule) |
| Raw pointer | `*const T`, `*mut T` | FFI, intrusive data structures, manual memory |
| Nullable | `T?` | Compile-time tracked; no null deref at runtime |

Key properties:

- **Drop semantics**: When an owned value goes out of scope, its destructor (if any) runs deterministically. No finalizer threads, no GC pauses.
- **No hidden allocations**: `let x = expr;` never allocates on the heap. Heap allocation requires explicit `Box`, `Vec`, etc.
- **Borrow checker enforces XOR**: Either many `&T` or one `&mut T`, never both. Same rule as Rust; same compile-time guarantees against data races and aliasing bugs.
- **Raw pointers are unchecked**: `*T` exists for FFI and low-level work. They do not participate in borrow checking — the programmer is responsible.

See [OWNERSHIP](docs/tutorials/OWNERSHIP.md) for the full tutorial and [SPECIFICATION §3](docs/SPECIFICATION.md) for the formal rules.

---

## Verification Model

Contracts are checked by an SMT solver (Z3) at compile time. The honest part of "compile-time proofs" is what happens when the solver cannot decide:

| SMT outcome | Compiler behavior |
|-------------|-------------------|
| `proved` | Compiles; runtime check elided |
| `disproved` (counterexample) | Compile error with witness |
| `unknown` / `timeout` | **Compile error** (default) |

There is **no runtime fallback**. If the prover times out, the build fails — silently passing unchecked contracts would defeat the entire model.

Two escape hatches exist for genuinely undecidable conditions:

```bmb
// Skip verification, document the proof obligation in prose
@trust "monotonicity follows from sorted invariant; see lemma 4.2"
fn binary_search(arr: &[i64], target: i64) -> i64? = { ... };
```

```toml
# bmb.toml — relax timeout policy globally (not recommended)
[smt]
timeout_ms = 5000
timeout_action = "error"  # error | trust_with_warning
```

`@trust` requires a written reason — it shifts the proof obligation from the solver to a reviewer, but does not hide it. `trust_with_warning` exists for incremental verification of legacy code.

See [VERIFICATION](docs/VERIFICATION.md) for the full policy, decidable fragment boundaries, and comparison with Dafny / F* / SPARK / Kani.

---

## Self-Hosting

BMB compiles itself. The bootstrap compiler (`bootstrap/compiler.bmb`, 32K LOC) achieves a **3-stage fixed point**:

```
Rust compiler → Stage 1 (BMB₁)
BMB₁ compiles bootstrap → Stage 2 (BMB₂)
BMB₂ compiles bootstrap → Stage 3 (BMB₃)
Verified: Stage 2 IR == Stage 3 IR ✅
```

---

## Quick Start

```bash
bmb run examples/hello.bmb        # run
bmb check examples/simple.bmb     # type check
bmb verify examples/contracts.bmb # prove contracts (requires Z3)
bmb build examples/hello.bmb -o hello  # compile native (requires LLVM)
```

## Building BMB

```bash
git clone https://github.com/iyulab/lang-bmb.git
cd lang-bmb
cargo build --release --features llvm --target x86_64-pc-windows-gnu  # Windows
cargo build --release --features llvm                                 # Linux/macOS
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
- **Platforms**: Windows x64 primary. Linux/macOS via Bindings CI.

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
| [Verification Model](docs/VERIFICATION.md) | SMT policy, timeout handling, escape hatches |
| [Comparison](docs/COMPARISON.md) | BMB vs Rust+Kani / Dafny / F* / SPARK / Vale |
| [Ownership](docs/tutorials/OWNERSHIP.md) | Memory model tutorial |
| [Build from Source](docs/BUILD_FROM_SOURCE.md) | Build instructions |
| [Benchmark](docs/BENCHMARK.md) | Performance methodology and results |
| [SIMD Performance Guide](docs/SIMD_PERF.md) | When to write manual SIMD vs trust auto-vec |
| [Contributing](docs/CONTRIBUTING.md) | How to contribute |
| [Target Users](docs/TARGET_USERS.md) | Who BMB is for |
| [Roadmap](docs/ROADMAP.md) | Development roadmap |

---

## Status

BMB is an **experimental language** in active development (v0.98). The compiler self-hosts via a 3-Stage Fixed Point bootstrap, all 16 Tier 1 benchmarks reach parity with Clang -O3 (within 5%), and SIMD is a first-class type system. The ecosystem is young and the community is small.

If you're interested in contract-verified systems programming, formal methods, or AI-assisted code generation — we'd love your feedback. See [VERIFICATION](docs/VERIFICATION.md) for the verification model and [COMPARISON](docs/COMPARISON.md) for how BMB relates to Dafny, F*, SPARK, and Rust+Kani.

---

## License

MIT

---

<p align="center">
  <b>Performance > Everything</b><br>
  <sub>The verbose contracts and explicit syntax that make BMB tedious for humans are tractable for AI.<br>
  Everything BMB asks the programmer to write enables a runtime check to be erased at compile time.</sub>
</p>
