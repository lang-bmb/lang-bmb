# BMB - Bare-Metal-Banter

A verified systems programming language with contract-based verification, designed for AI-native code generation.

## Current Status: v0.30.310

| Component | Status | Description |
|-----------|--------|-------------|
| Lexer/Parser | Complete | logos + lalrpop based tokenization, 113+ tests |
| Type System | Complete | Generics, refinement types, Option/Result, ownership, closures |
| Contracts | Complete | pre/post conditions, quantifiers, SMT verification (Z3) |
| Interpreter | Complete | Tree-walking interpreter with REPL |
| MIR | Complete | Middle Intermediate Representation with optimization |
| LLVM Backend | Complete | Native code generation via inkwell (optional) |
| Bootstrap | Complete | Self-hosted compiler in BMB (29,818 lines, 93+ tests) |
| Module System | Complete | Cross-package type references, use statements |
| Code Quality | Complete | 0 Clippy warnings, 0 doc warnings, 132 tests |

## Quick Start

```bash
# Build the compiler
cargo build --release

# Run a BMB program
bmb run examples/hello.bmb

# Type check a file
bmb check examples/simple.bmb

# Parse and output AST
bmb parse examples/simple.bmb                 # JSON format
bmb parse examples/simple.bmb --format=sexpr  # S-expression format

# Verify contracts (requires Z3)
bmb verify examples/verify.bmb --z3-path /path/to/z3

# Start interactive REPL
bmb repl

# Build native executable (requires LLVM)
bmb build examples/hello.bmb -o hello
bmb build examples/hello.bmb --release     # optimized
bmb build examples/hello.bmb --emit-ir     # output LLVM IR
```

## Building with LLVM

For native code generation, build with the `llvm` feature:

```bash
# Requires LLVM 21 with llvm-config
cargo build --release --features llvm
```

> **Note**: Windows pre-built LLVM does not include `llvm-config`. Use MSYS2 LLVM or build from source.

## Language Example

```bmb
-- Function with contract verification
fn max(a: i64, b: i64) -> i64
  post ret >= a and ret >= b
= if a > b then a else b;

-- Precondition ensures non-zero division
fn safe_div(a: i64, b: i64) -> i64
  pre b != 0
= a / b;

-- Generic type with refinement
type NonZero = i64 where self != 0;

enum Option<T> {
  Some(T),
  None
}

-- Method call syntax
fn example(x: Option<i64>) -> i64 =
  x.unwrap_or(0);

-- Closure expression (v0.20+)
fn apply_twice(f: fn(i64) -> i64, x: i64) -> i64 =
  f(f(x));
```

## Project Structure

```
lang-bmb/
├── bmb/                    # Rust compiler implementation
│   └── src/
│       ├── lexer/          # Token definitions (logos)
│       ├── parser/         # Parser (lalrpop) + tests
│       ├── ast/            # AST definitions + S-expr output
│       ├── types/          # Type checker with generics
│       ├── smt/            # SMT-LIB2 generation
│       ├── verify/         # Contract verification
│       ├── interp/         # Tree-walking interpreter
│       ├── mir/            # Middle IR
│       ├── codegen/        # LLVM/WASM backends
│       ├── lsp/            # Language Server Protocol
│       └── repl/           # Interactive REPL
├── bootstrap/              # Self-hosted compiler in BMB
├── stdlib/                 # Standard library
├── runtime/                # Runtime support
├── ecosystem/              # Development tools (submodules)
├── tests/                  # Integration tests
├── examples/               # Example programs
└── docs/                   # Documentation
```

## Ecosystem (Submodules)

| Repository | Description | Status |
|------------|-------------|--------|
| [gotgan](ecosystem/gotgan) | Package manager with Rust fallback | Active |
| [tree-sitter-bmb](ecosystem/tree-sitter-bmb) | Tree-sitter grammar for editors | Active |
| [vscode-bmb](ecosystem/vscode-bmb) | VS Code extension | Active |
| [playground](ecosystem/playground) | Online playground (WASM) | Active |
| [lang-bmb-site](ecosystem/lang-bmb-site) | Official website | Active |
| [bmb-samples](ecosystem/bmb-samples) | Example programs and tutorials | Active |
| [benchmark-bmb](ecosystem/benchmark-bmb) | Performance benchmarks | Active |
| [action-bmb](ecosystem/action-bmb) | GitHub Actions support | Active |

### Submodule Setup

```bash
# Clone with submodules
git clone --recursive https://github.com/lang-bmb/lang-bmb.git

# Or initialize after clone
git submodule update --init --recursive
```

## Bootstrap Status

Self-hosted compiler components written in BMB (29,818 lines):

| Component | Lines | Description | Status |
|-----------|-------|-------------|--------|
| lexer.bmb | 1,046 | Token generation | Complete |
| parser.bmb | 1,523 | Syntax validation | Complete |
| parser_ast.bmb | 3,666 | S-expression AST | Complete |
| types.bmb | 8,764 | Type checking (316 tests) | Complete |
| mir.bmb | 1,705 | MIR foundation | Complete |
| lowering.bmb | 3,867 | AST to MIR transform | Complete |
| llvm_ir.bmb | 4,621 | LLVM IR generation | Complete |
| pipeline.bmb | 2,107 | End-to-end compilation (42 tests) | Complete |
| compiler.bmb | 2,783 | Full compiler driver | Complete |

**Stage 3 Bootstrap**: 6/7 tests passing (86%)

See [bootstrap/README.md](bootstrap/README.md) for details.

## Requirements

- Rust 1.70+
- Z3 Solver (for contract verification)
- LLVM 21 (optional, for native codegen)

## Documentation

| Document | Description |
|----------|-------------|
| [SPECIFICATION.md](docs/SPECIFICATION.md) | Complete language specification |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Compiler architecture and internals |
| [ROADMAP.md](docs/ROADMAP.md) | Development roadmap and milestones |
| [GOTGAN.md](docs/GOTGAN.md) | Package manager specification |
| [ECOSYSTEM.md](docs/ECOSYSTEM.md) | Ecosystem tools and submodules |

## Design Philosophy

BMB is designed as an **AI-native** programming language:

| Principle | Description |
|-----------|-------------|
| Correctness First | Contract verification prevents bugs at source |
| Performance | Contracts enable optimizations beyond C/Rust |
| AI-Native | Optimized for LLM code generation |
| Minimal Rules | Same syntax = same meaning, zero exceptions |
| Composability | Small contracts compose into complex ones |

## License

MIT
