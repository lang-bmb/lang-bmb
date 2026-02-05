# BMB — Bare-Metal-Banter

> **Banter for AI. Bare-metal for humans.**

A contract-verified systems programming language.

[![Version](https://img.shields.io/badge/version-0.60.251-blue.svg)](docs/ROADMAP.md)
[![Bootstrap](https://img.shields.io/badge/bootstrap-3--stage%20fixed%20point-green.svg)](docs/BOOTSTRAP_BENCHMARK.md)
[![Performance](https://img.shields.io/badge/vs%20C-≤1.10x-brightgreen.svg)](docs/BENCHMARK.md)

---

## Why BMB?

Every language faces this trade-off:

```
Runtime Overhead ←――――――――――→ Developer Effort
```

To eliminate runtime overhead, you need exhaustive type annotations, formal proofs, and explicit everything. **For humans, this is unsustainable.**

AI changes the equation. LLMs can write verbose, formally-specified code without complaint.

But AI isn't infinite—context limits, hallucination, and verification needs prevent direct machine code generation.

**BMB is the lowest abstraction level that AI can efficiently produce.**

- Lower than BMB → Context explosion, unverifiable, hallucination
- Higher than BMB → Runtime overhead unavoidable

This position could not exist before AI.

---

## The Trade-off

Most languages optimize for humans. BMB doesn't.

| You Give Up | You Get |
|-------------|---------|
| More type annotations | More aggressive optimization |
| Contracts required | Runtime checks eliminated |
| Explicit conversions | Predictable performance |
| More compile errors | Fewer runtime errors |

**Hard to write. Hard to get wrong. And that's what AI prefers.**

---

## Zero-Overhead Safety

Every safety check compiles away to nothing:

| Runtime Check (Other Languages) | BMB Approach | Overhead |
|---------------------------------|--------------|----------|
| Bounds checking | `pre idx < arr.len()` | **0%** |
| Null checking | `T?` type + contract | **0%** |
| Overflow checking | Contract or explicit op | **0%** |
| Division by zero | `pre divisor != 0` | **0%** |

```bmb
fn get(arr: &[i32], idx: usize) -> i32
  pre idx < arr.len()
= arr[idx];
```

This generates **identical assembly** to unchecked C. The proof happens at compile-time. The runtime cost is zero.

---

## Why AI-First?

| | Humans | AI |
|---|--------|-----|
| Verbose types | Annoying | Trivial |
| Explicit contracts | Tedious | Natural |
| Formal proofs | Difficult | Preferred |
| No shortcuts | Frustrating | Irrelevant |

Traditional languages hide complexity to help humans. BMB exposes everything—because AI handles verbosity effortlessly.

---

## A Taste of BMB

### Contracts as Code

```bmb
fn binary_search(arr: &[i32], target: i32) -> usize?
  pre is_sorted(arr)
  post ret.is_none() implies forall i: 0..arr.len(). arr[i] != target
  post ret.is_some() implies arr[ret.unwrap()] == target
{
    // implementation
}
```

Pass an unsorted array? **Compile error.**

### Overflow—Your Choice

```bmb
let a = x + y;      // requires contract proving no overflow
let b = x +% y;     // wrapping (mod 2^n)
let c = x +| y;     // saturating (clamp to bounds)
let d = x +? y;     // checked (returns T?)
```

No silent overflow. No debug/release differences. You decide the semantics.

### Pure Functions

```bmb
pure fn square(x: i64) -> i64 = x * x;
```

The compiler guarantees: no side effects, deterministic output. Safe for memoization, reordering, parallelization.

### Refinement Types

```bmb
type NonZero = i64 where self != 0;
type Percentage = f64 where self >= 0.0 and self <= 100.0;
```

Types that carry their own proofs.

---

## Quick Start

```bash
bmb run examples/hello.bmb        # run
bmb check examples/simple.bmb     # type check
bmb verify examples/contracts.bmb # prove contracts (requires Z3)
bmb build examples/hello.bmb -o hello  # compile native (requires LLVM)
bmb repl                          # interactive
```

---

## Building BMB

### Option 1: Golden Binary (No Rust Required)

```bash
git clone https://github.com/lang-bmb/lang-bmb.git
cd lang-bmb
./scripts/golden-bootstrap.sh        # builds bmb-stage1
./scripts/install.sh --user          # install to ~/.local
```

**Requirements**: LLVM 21+ only (`opt`, `clang`)

### Option 2: From Source with Rust

```bash
cargo build --release --features llvm --target x86_64-pc-windows-gnu
```

**Requirements**: Rust 1.75+, LLVM 21+

See [Building from Source](docs/BUILD_FROM_SOURCE.md) for details.

---

## Performance

When the compiler knows your invariants, it knows what's safe to optimize.

| Benchmark | BMB vs C | Notes |
|-----------|----------|-------|
| fasta | **28%** | BMB 3.6x faster |
| ackermann | **0.4%** | TCO: BMB 250x faster |
| sorting | **37%** | Tail recursion |
| fibonacci | **102%** | Near parity |
| mandelbrot | **100%** | Identical IR |

**Gate Status (v0.60):**
- ✅ All benchmarks ≤1.10x vs C
- ✅ 20+ benchmarks where BMB > C
- ✅ 3-Stage Bootstrap: Fixed Point achieved

BMB's goal: safe code that generates **identical assembly** to unsafe C.

---

## Contract-Driven Optimization (CDO)

> **Contracts are not just guards. They are guides.**

Beyond safety verification, contracts enable unprecedented optimization:

```bmb
fn parse(s: &str) -> Value
  pre s.len() < 10000        // Enables: small-string optimization
  pre s.is_ascii()           // Enables: skip unicode handling
  post ret.is_valid()        // Enables: skip validation at call sites
```

| CDO Capability | Example | Impact |
|----------------|---------|--------|
| **Semantic DCE** | `pre x > 0` eliminates `if x <= 0` branch | Dead code removal |
| **Minimal Extraction** | Import only contract-compatible paths | 60-80% less dependency code |
| **Pure Precomputation** | `pure fn` + bounded input → lookup table | Zero runtime cost |
| **Semantic Deduplication** | Merge functions with equivalent contracts | Smaller binaries |

See [RFC-0001: Contract-Driven Optimization](docs/rfcs/RFC-0008-contract-driven-optimization.md).

---

## Ecosystem

| Tool | Purpose |
|------|---------|
| [bmb-mcp](ecosystem/bmb-mcp) | MCP server for AI integration (Chatter) |
| [bmb-test](ecosystem/bmb-test) | Property-based testing with contract awareness |
| [bmb-query](ecosystem/bmb-query) | Natural language queries against contracts |
| [gotgan](ecosystem/gotgan) | Package manager |
| [vscode-bmb](ecosystem/vscode-bmb) | VS Code extension |

---

## Documentation

| Document | Description |
|----------|-------------|
| [Specification](docs/SPECIFICATION.md) | Formal language definition |
| [Language Reference](docs/LANGUAGE_REFERENCE.md) | Complete feature guide |
| [Architecture](docs/ARCHITECTURE.md) | Compiler internals |
| [Build from Source](docs/BUILD_FROM_SOURCE.md) | Build instructions |
| [Benchmark](docs/BENCHMARK.md) | Performance results |
| [Ecosystem](docs/ECOSYSTEM.md) | Tools and submodules |
| [Roadmap](docs/ROADMAP.md) | Development roadmap |

---

## License

MIT

---

<p align="center">
  <b>Performance > Everything</b><br>
  <sub>Safety is not a goal—it's a consequence of pursuing maximum performance.</sub>
</p>
