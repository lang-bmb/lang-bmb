# BMB — Bare-Metal-Banter

A contract-verified systems programming language that compiles to native code via LLVM.

[![Version](https://img.shields.io/badge/version-0.100.0-blue.svg)](docs/ROADMAP.md)
[![Bootstrap](https://img.shields.io/badge/bootstrap-3--stage%20fixed%20point-green.svg)](docs/BOOTSTRAP_BENCHMARK.md)
[![Tests](https://img.shields.io/badge/tests-6264%20passed-brightgreen.svg)](bmb/src)
[![B-axis](https://img.shields.io/badge/B--axis-100%25%20(300%2F300)-brightgreen.svg)](ecosystem/bmb-ai-bench)

---

## Hello, BMB

```bmb
fn main() -> i64 = {
    let name = "BMB";
    println("Hello, {name}!");
    0
};
```

```bash
bmb run hello.bmb
# Hello, BMB!
```

---

## What Makes BMB Different

BMB pursues **maximum performance through compile-time proofs**. Safety is not a separate goal — when contracts are verified, it follows from those same compile-time proofs.

```bmb
fn get(arr: &[i64], idx: i64) -> i64
  pre idx >= 0
  pre idx < arr.len()
= arr[idx];
```

At runtime, contracts generate **zero overhead** — no bounds checks, no null checks, nothing; unproven access compiles to unchecked native code (same speed as unsafe C). Safety is **opt-in**: `bmb verify` discharges the `pre` conditions via an SMT solver (Z3) as a separate step — `bmb build` does **not** enforce it. The proof obligation is explicit, but not yet build-gated.

| Approach | Safety Method | Runtime Cost |
|----------|--------------|-------------|
| C | None (programmer responsibility) | 0% |
| Rust | Ownership + borrow checker | 0% (most cases) |
| Go/Java | Runtime checks (GC, bounds) | >0% |
| **BMB** | Compile-time contract proofs | **0%** |

---

## Performance

BMB targets parity with — and on several workloads beats — the best C compilers. All numbers
below are measured with **checksum-verified identical work** across compilers (not assumed), under
a single pinned configuration: bootstrap S1 compiler, BMB `opt -O2`, LLVM 21.1.7, GCC MinGW
(ucrt64), commit `a5fd27de` (benchmark-bmb submodule `757f85c`), 2026-06-01. Ratio = BMB time / baseline time; **< 1.0 = BMB faster**.

**vs Clang -O3** — same LLVM backend, the toughest and fairest comparison:

| Benchmark | BMB / Clang -O3 | Reading |
|-----------|-----------------|---------|
| http_parse | **0.84×** | 1.2× faster |
| lexer | **0.85×** | 1.2× faster |
| brainfuck | **0.92×** | 1.08× faster |
| json_parse | **0.95×** | ~parity |
| csv_parse | **0.98×** | parity |
| json_serialize | **1.00×** | parity |
| sorting | **1.11×** | ~11% slower |

Against the LLVM peer, BMB is **at parity, modestly faster on 3 of 7** (http_parse, lexer,
brainfuck ~1.1–1.2×), parity on 3, and ~11% slower on sorting. This is the expected result —
BMB shares the LLVM backend, so it tracks LLVM performance rather than beating it. The honest
takeaway: a contract-verified language reaches LLVM-backend performance with no overhead.

**vs GCC -O2** — a different backend; results vary by workload:

| Benchmark | BMB / GCC -O2 | | Benchmark | BMB / GCC -O2 |
|-----------|---------------|-|-----------|---------------|
| sorting | **0.18×** (5.5× faster) | | brainfuck | 0.86× |
| lexer | **0.31×** (3.2× faster) | | csv_parse | 1.00× (parity) |
| http_parse | 0.86× | | **json_parse** | **2.15× (slower)** |
| json_serialize | 0.89× | | | |

The large sorting/lexer margins are **LLVM-vs-GCC backend** differences (GCC -O2 optimizes those
loops less aggressively), not a BMB-specific advantage; conversely GCC beats BMB ~2× on
json_parse. We report the full picture rather than headlining the largest ratio.

> **Correction (2026-06-01)**: an earlier version claimed "7/7 faster than Clang -O3" with
> figures (e.g. sorting 0.155×, lexer 0.174×) that were actually GCC-relative ratios mislabeled
> as vs-Clang. A forensic audit (LLVM IR inspection + checksum verification) found the work is
> real (not DCE'd) but the headline was a **baseline-selection + harness-fairness artifact**:
> against the LLVM peer (Clang -O3) BMB is at parity. Five benchmark C harnesses were corrected
> for fairness — inlined char classification (lexer, json_parse; libc `isspace`/`isdigit` were
> heavier than BMB's inlined comparisons), runtime-opaque input (http_parse, brainfuck; defeats
> clang const-folding), and matched csv checksum. See [audit](claudedocs/measurements/al1_forensic_audit_2026-06-01.md).

See [Benchmark Details](docs/BENCHMARK.md) for methodology, raw numbers, and noise analysis.

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

### String Interpolation

```bmb
let greeting = "Hello, {name}!";
let report = "Found {count} items in {ms}ms";
```

### Control Flow

```bmb
// if without else (unit branch)
if x > 0 { println("positive"); }

// while-let pattern
while let Some(item) = iter_next(it) {
    process(item);
}

// for-in collections
for x in vec { sum += x; }
for x in svec { process(x); }
```

### Compound Assignment

```bmb
count += 1;
total -= cost;
score *= multiplier;
```

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

### Rich Standard API

Over 70 built-in functions across strings, vectors, math, and collections — all with native codegen:

```bmb
// Strings
str_split(s, ",")        // Vec of parts
str_replace(s, "a", "b")
str_to_upper(s)
"pad: {str_pad_left(n, 5, '0')}"

// Vectors
vec_sort(v)
vec_sum(v)
vec_contains(v, x)

// Math
pow_i64(base, exp)
gcd_i64(a, b)
clamp_i64(x, lo, hi)
```

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

See [OWNERSHIP](docs/tutorials/OWNERSHIP.md) for the full tutorial and [SPECIFICATION §3](docs/SPECIFICATION.md) for the formal rules.

---

## Verification Model

Contracts are checked by an SMT solver (Z3) at compile time.

| SMT outcome | Compiler behavior |
|-------------|-------------------|
| `proved` | Compiles; runtime check elided |
| `disproved` (counterexample) | Compile error with witness |
| `unknown` / `timeout` | **Compile error** (default) |

There is **no runtime fallback**. Two escape hatches exist for genuinely undecidable conditions:

```bmb
@trust "monotonicity follows from sorted invariant; see lemma 4.2"
fn binary_search(arr: &[i64], target: i64) -> i64? = { ... };
```

See [VERIFICATION](docs/VERIFICATION.md) for the full policy and comparison with Dafny / F* / SPARK / Kani.

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

## AI Code Generation (B-axis)

BMB was designed with AI code generation in mind. We measure this directly.

**GPUStack benchmark** (100 problems × 3 runs, qwen3.6-35b-a3b, 2026-05-21):

```
300 / 300 runs passed  →  100.0%
```

The explicit, contract-first syntax that feels verbose for humans proves tractable for AI. Every contract BMB asks for enables a runtime check to be erased at compile time — and AI can write those contracts without complaint.

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
| Performance-critical numeric computation | Good | Parity with Clang -O3 (faster on 3/7 measured workloads) |
| Safety-critical systems (avionics, medical) | Not yet | Verification is opt-in (`bmb verify`), not build-enforced; contract completeness immature |
| AI-generated code pipelines | Good | 100% pass rate on GPUStack benchmark |
| General application development | Not yet | Ecosystem still growing |
| Rapid prototyping | No | Use Python/TypeScript instead |

### Current Limitations

- **Ecosystem**: ~14 packages. Standard library growing.
- **Community**: Early stage. Contributions welcome.
- **Tooling**: VS Code extension available. LSP in progress.
- **Platforms**: Windows x64 primary. Linux/macOS via CI.

---

## Design Philosophy

BMB's direction is opposite to Rust:

| Language | Primary Goal | Method | Consequence |
|----------|-------------|--------|------------|
| Rust | Memory Safety | Ownership + Borrow Checker | Good performance |
| **BMB** | **Performance** | **Compile-time proofs** | **Safety provable** |

**BMB is not "a safe language." It is a fast language whose speed model makes safety provable — to the extent contracts are written and verified.**

---

## Ecosystem

| Tool | Purpose |
|------|---------|
| [gotgan](ecosystem/gotgan) | Package manager |
| [vscode-bmb](ecosystem/vscode-bmb) | VS Code extension |
| [tree-sitter-bmb](ecosystem/tree-sitter-bmb) | Syntax highlighting |
| [playground](ecosystem/playground) | Online editor (WebAssembly) |
| [benchmark-bmb](ecosystem/benchmark-bmb) | Performance test suite |
| [bmb-mcp](ecosystem/bmb-mcp) | MCP server for AI integration |

### Language Bindings

| Language | Package | Tests |
|----------|---------|-------|
| Python | PyPI (`bmb-algo`, `bmb-compute`, `bmb-text`, `bmb-crypto`, `bmb-json`) | ✅ |
| Node.js | npm | ✅ |
| C# | NuGet | ✅ 93 |
| Java | JNA | ✅ 120 |
| C | Header + shared lib | ✅ 216 |

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
| [Roadmap](docs/ROADMAP.md) | Development roadmap |

---

## Status

BMB is an **experimental language** in active development (v0.100.0). The compiler self-hosts via a 3-Stage Fixed Point bootstrap, the 7 measured real-world benchmarks reach parity with Clang -O3 (3/7 modestly faster, 3 parity, 1 ~11% slower; see Performance), and the AI code generation benchmark reaches 100% on GPUStack.

Milestones completed: M1 (Performance Parity), M2 (AI-Ready Infrastructure), M3 (Language Ecosystem — Python/Node/C#/Java/C bindings, MCP server, LSP, playground), M4 (Language Completeness — 70+ builtins, string interpolation, while-let, for-in, compound assignment).

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
