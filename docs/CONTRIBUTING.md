# Contributing to BMB

Thank you for your interest in contributing to BMB. This guide covers everything you need to get started.

BMB is an experimental language — contributions of all kinds are valuable, from code to documentation to bug reports.

---

## Table of Contents

- [Ways to Contribute](#ways-to-contribute)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Good First Issues](#good-first-issues)
- [Architecture Quickstart](#architecture-quickstart)
- [Performance Philosophy](#performance-philosophy)
- [Getting Help](#getting-help)

---

## Ways to Contribute

| Type | Examples | Difficulty |
|------|----------|------------|
| **Bug reports** | Found a crash, wrong output, or confusing error? File an issue. | Easy |
| **Documentation** | Fix typos, clarify explanations, add examples | Easy |
| **Golden tests** | Add end-to-end tests in `tests/bootstrap/` | Easy-Medium |
| **Benchmarks** | Add new benchmarks in `ecosystem/benchmark-bmb/` | Medium |
| **Compiler fixes** | Fix bugs in `bootstrap/compiler.bmb` | Medium-Hard |
| **Optimization passes** | Improve MIR optimization in `bootstrap/optimize.bmb` | Hard |
| **Ecosystem packages** | Write BMB packages for `ecosystem/gotgan-packages/` | Medium |

---

## Getting Started

### Prerequisites

| Component | Version | Required For |
|-----------|---------|-------------|
| LLVM | 21+ | All compilation (`opt`, `llc`) |
| GCC (MinGW) | Any recent | Linking on Windows |
| Rust | 1.75+ | Building Rust compiler (optional) |
| Z3 | 4.12+ | Contract verification (optional) |

### Build the Compiler

**Option A: Golden Binary Bootstrap (recommended, no Rust needed)**

```bash
git clone https://github.com/iyulab/lang-bmb.git
cd lang-bmb
./scripts/golden-bootstrap.sh
```

This produces `target/golden-bootstrap/bmb-stage1.exe` in about 8 seconds.

**Option B: Rust Compiler Build**

```bash
# Windows (MSYS2/MinGW)
cargo build --release --features llvm --target x86_64-pc-windows-gnu

# Linux/macOS
cargo build --release --features llvm
```

### Verify Your Setup

```bash
# Run all tests (~5,200 tests)
cargo test --release

# Run golden tests (69 end-to-end tests)
./scripts/golden-bootstrap.sh --verify

# Quick validation (~2 minutes)
./scripts/quick-check.sh
```

If all tests pass, you're ready to contribute.

---

## Development Workflow

### Important: Bootstrap-First Development

As of v0.94, the Rust compiler (`bmb/src/`) is **frozen**. All new compiler work happens in BMB:

```
bootstrap/compiler.bmb    ← Active development (all new features, fixes)
bmb/src/*.rs              ← Frozen (maintenance only, no new code)
```

### Making Changes

**For compiler changes:**

```bash
# 1. Edit the bootstrap compiler
#    (editor) bootstrap/compiler.bmb

# 2. Build and test
./scripts/bmb-dev.sh full

# 3. If tests pass, commit
```

**For documentation, tests, or ecosystem changes:**

Standard git workflow — edit, test, commit.

### Key Scripts

| Script | Purpose | Time |
|--------|---------|------|
| `./scripts/quick-check.sh` | Tests + Stage 1 bootstrap + Tier 0 benchmarks | ~2 min |
| `./scripts/bmb-dev.sh full` | Build, test, verify, compile | ~5 min |
| `./scripts/full-cycle.sh` | Full 3-stage bootstrap + all benchmarks | ~15 min |
| `./scripts/benchmark.sh --tier 1` | Run Tier 1 performance benchmarks | ~5 min |

---

## Code Style

BMB follows the conventions in [docs/STYLE_GUIDE.md](STYLE_GUIDE.md). Key points:

- **Indentation**: 4 spaces (no tabs)
- **Line length**: 100 characters max
- **Functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_CASE`
- **Two blank lines** between top-level definitions

```bmb
fn calculate_sum(a: i64, b: i64) -> i64
  pre a >= 0
  post ret >= a
= a + b;


fn main() -> i64 = {
    let result = calculate_sum(10, 20);
    println(int_to_string(result));
    0
};
```

For Rust code (maintenance only): follow existing patterns, pass `cargo clippy -- -D warnings`.

---

## Testing

### Test Types

| Type | Location | How to Run |
|------|----------|-----------|
| Rust unit/integration tests | `bmb/src/` | `cargo test --release` |
| Golden tests | `tests/bootstrap/*.bmb` | `./scripts/golden-bootstrap.sh --verify` |
| Benchmarks | `ecosystem/benchmark-bmb/` | `./scripts/benchmark.sh --tier 1` |

### Adding a Golden Test

Golden tests verify the full compilation pipeline: BMB source → Stage 1 → LLVM IR → native → run → check output.

1. Create `tests/bootstrap/test_golden_<name>.bmb`
2. The program's `main()` must return `i64` (the expected output)
3. Add an entry to `tests/bootstrap/golden_tests.txt`:
   ```
   test_golden_<name>.bmb|<expected_output>
   ```
4. Run `./scripts/golden-bootstrap.sh --verify` to confirm it passes

**Example:**

```bmb
fn factorial(n: i64) -> i64 =
    if n <= 1 { 1 }
    else { n * factorial(n - 1) };

fn main() -> i64 = factorial(10);  // Expected: 3628800
```

```
# In golden_tests.txt
test_golden_factorial.bmb|3628800
```

### Adding a Benchmark

1. Create `ecosystem/benchmark-bmb/benches/compute/<name>/`
2. Add BMB version: `bmb/main.bmb`
3. Add C version: `c/main.c`
4. Both must produce identical output (verified automatically)
5. Classify into Tier 0/1/2 based on runtime and optimization usage

See [docs/BENCHMARK.md](BENCHMARK.md) for tier definitions and methodology.

### Test Requirements for PRs

| Requirement | Blocking? |
|-------------|-----------|
| All `cargo test --release` pass | Yes |
| All golden tests pass | Yes |
| No Tier 1 benchmark regression > 2% | Yes |
| New tests for new features | Yes |
| No Tier 0/2 benchmark regression > 5% | Warning |

---

## Submitting Changes

### Before You Submit

1. Run `./scripts/quick-check.sh` (tests + Stage 1 bootstrap)
2. For compiler changes, run `./scripts/full-cycle.sh` (full 3-stage verification)
3. For performance-affecting changes, run benchmarks and include results

### Pull Request Process

1. Fork the repository and create a branch
2. Make your changes with tests
3. Run the appropriate verification (see above)
4. Open a PR with:
   - **Title**: Short description (< 70 characters)
   - **Description**: What changed and why
   - **Test results**: Paste test output
   - **Benchmark results**: If performance-relevant

### Commit Message Format

```
<type>: <description>

Types:
  feat:  New feature or capability
  fix:   Bug fix
  perf:  Performance improvement
  docs:  Documentation change
  test:  New or modified tests
  chore: Maintenance (build, CI, deps)
```

Examples:
```
feat: add pattern matching for tuple types
fix: correct phi node generation for while loops
perf: eliminate redundant zext/trunc in IR generation
docs: add contract verification tutorial
test: add golden test for recursive fibonacci
```

---

## Good First Issues

These are concrete tasks suitable for first-time contributors. They're organized by difficulty and type.

### Documentation (Easy)

| # | Task | File(s) |
|---|------|---------|
| D-1 | Add examples to LANGUAGE_REFERENCE.md for each overflow operator (`+%`, `+\|`, `+?`) | `docs/LANGUAGE_REFERENCE.md` |
| D-2 | Document all `bmb` CLI subcommands with examples | `docs/` (new file) |
| D-3 | Add "Common Errors" troubleshooting section to Getting Started tutorial | `docs/tutorials/GETTING_STARTED.md` |
| D-4 | Document concurrency primitives (thread, mutex, channel) with examples | `docs/LANGUAGE_REFERENCE.md` |

### Tests (Easy-Medium)

| # | Task | Details |
|---|------|---------|
| T-1 | Add golden test for string methods (len, concat, slice) | `tests/bootstrap/` |
| T-2 | Add golden test for struct creation and field access | `tests/bootstrap/` |
| T-3 | Add golden test for match expressions with multiple patterns | `tests/bootstrap/` |
| T-4 | Add golden test for contract verification (pre/post) | `tests/bootstrap/` |
| T-5 | Add golden test for array methods (push, pop, slice, join) | `tests/bootstrap/` |

### Benchmarks (Medium)

| # | Task | Details |
|---|------|---------|
| B-1 | Add `json_parse` benchmark (BMB + C versions) | `ecosystem/benchmark-bmb/benches/` |
| B-2 | Add `matrix_multiply` benchmark with larger sizes | `ecosystem/benchmark-bmb/benches/` |
| B-3 | Add memory allocation stress test benchmark | `ecosystem/benchmark-bmb/benches/` |

### Ecosystem (Medium)

| # | Task | Details |
|---|------|---------|
| E-1 | Add syntax highlighting for more editors (Vim, Emacs) | `ecosystem/` |
| E-2 | Write a BMB package: `bmb-base64` (encode/decode) | `ecosystem/gotgan-packages/` |
| E-3 | Write a BMB package: `bmb-math` (trig, log, exp wrappers) | `ecosystem/gotgan-packages/` |

### Compiler (Medium-Hard)

| # | Task | Details |
|---|------|---------|
| C-1 | Fix `[0; n]` codegen when `n` is a function parameter (missing `%` prefix) | `bootstrap/compiler.bmb` |
| C-2 | Add support for struct-typed fields in bootstrap compiler | `bootstrap/compiler.bmb` |
| C-3 | Improve error messages for contract verification failures | `bootstrap/compiler.bmb` |

---

## Architecture Quickstart

### Compilation Pipeline

```
Source (.bmb)
    ↓ Lexer (logos-based)
Token stream
    ↓ Parser (lalrpop)
Untyped AST
    ↓ Type Checker (Hindley-Milner)
Typed AST
    ↓ MIR Lowering
Middle IR (MIR)
    ↓ Optimization Passes (5 passes, 37.5% IR reduction)
Optimized MIR
    ↓ CodeGen (LLVM IR)
LLVM IR → opt → llc → gcc → Native Binary
```

### Key Source Files

**Bootstrap Compiler** (active, all new development):

| File | Purpose | Approx LOC |
|------|---------|-----------|
| `bootstrap/compiler.bmb` | Main pipeline, codegen | 32K |
| `bootstrap/lexer.bmb` | Tokenization | 2K |
| `bootstrap/parser.bmb` | Parsing + AST generation | 8K |
| `bootstrap/types.bmb` | Type inference | 12K |
| `bootstrap/lowering.bmb` | AST → MIR | 4K |
| `bootstrap/mir.bmb` | MIR data structures | 2K |
| `bootstrap/optimize.bmb` | MIR optimization passes | 3K |
| `bootstrap/llvm_ir.bmb` | LLVM IR generation | 6K |

**Rust Compiler** (frozen, reference only):

| Module | Purpose |
|--------|---------|
| `bmb/src/lexer/` | Token generation |
| `bmb/src/parser/` | Grammar (`grammar.lalrpop`) |
| `bmb/src/types/` | Type inference and checking |
| `bmb/src/codegen/` | LLVM and WASM codegen |

### Where to Make Changes

| I want to... | Edit this file |
|-------------|---------------|
| Fix a parser bug | `bootstrap/parser.bmb` |
| Fix a type error | `bootstrap/types.bmb` |
| Fix codegen output | `bootstrap/llvm_ir.bmb` or `bootstrap/compiler.bmb` |
| Add an optimization | `bootstrap/optimize.bmb` |
| Add a golden test | `tests/bootstrap/test_golden_*.bmb` |
| Add a benchmark | `ecosystem/benchmark-bmb/benches/compute/` |
| Fix documentation | `docs/*.md` |

---

## Performance Philosophy

BMB's core principle is **Performance > Everything**. This has implications for contributors:

1. **Performance issues are bugs.** A change that makes BMB slower is treated with the same severity as a crash.

2. **No workarounds.** If a fix requires changing the language spec, compiler structure, or optimization passes — that's the right fix. Shortcuts that mask the problem are not accepted.

3. **Measure everything.** All performance claims must be backed by benchmarks. Run `./scripts/benchmark.sh --tier 1` before and after performance-related changes.

4. **Decision framework** (check from top to bottom):
   - Level 1: Does the **language spec** need to change?
   - Level 2: Does the **compiler structure** (AST/MIR) need to change?
   - Level 3: Does an **optimization pass** need to be added/modified?
   - Level 4: Does the **code generation** need to improve?
   - Level 5: Does the **runtime** need to change?

   Fix at the highest applicable level. Fixing at Level 4 what should be fixed at Level 1 is a workaround.

See [docs/PRINCIPLES.md](PRINCIPLES.md) for the full development philosophy.

---

## Getting Help

- **GitHub Issues**: Report bugs, request features, ask questions
- **GitHub Discussions**: Design discussions, proposals, general questions
- **Existing Documentation**:
  - [Language Reference](LANGUAGE_REFERENCE.md) — Complete feature guide
  - [Architecture](ARCHITECTURE.md) — Compiler internals
  - [Specification](SPECIFICATION.md) — Formal language definition
  - [Benchmark](BENCHMARK.md) — Performance methodology
  - [Roadmap](ROADMAP.md) — What's planned next

If you're unsure where to start, open an issue describing what you'd like to work on. We'll help you find the right approach.

---

## License

By contributing to BMB, you agree that your contributions will be licensed under the MIT License.
