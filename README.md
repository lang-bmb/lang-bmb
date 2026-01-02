# BMB - Bare-Metal-Banter

A verified systems programming language with contract verification.

## Current Status: v0.10 Sunrise (Bootstrap)

### Features

- **Lexer/Parser**: logos + lalrpop based tokenization and AST generation
- **Type System**: Basic types (i32, i64, f64, bool, String, unit), functions, let bindings
- **Contract Verification**: pre/post condition verification via SMT solver (Z3)
- **Interpreter**: Tree-walking interpreter for direct execution
- **REPL**: Interactive environment with rustyline
- **MIR**: Middle Intermediate Representation for code generation
- **LLVM Backend**: Native code generation via inkwell (optional)
- **Error Reporting**: ariadne-based rich error messages
- **Standard Library** (v0.5+): Core types (Option, Result), collections, I/O, math
- **Ecosystem Tools** (v0.6+): Formatter (bmb-fmt), Language Server (bmb-lsp), Package Manager (gotgan)
- **Runtime** (v0.7+): Memory management, stack/heap allocation, panic handling
- **Bootstrap** (v0.10): Self-hosted compiler components written in BMB

### Quick Start

```bash
# Build the compiler
cargo build --release

# Run a BMB program (interpreter)
bmb run examples/hello.bmb

# Start interactive REPL
bmb repl

# Check a file for type errors
bmb check examples/simple.bmb

# Verify contracts (requires Z3)
bmb verify examples/verify.bmb --z3-path /path/to/z3

# Build native executable (requires LLVM, see below)
bmb build examples/hello.bmb -o hello
bmb build examples/hello.bmb --release  # optimized
bmb build examples/hello.bmb --emit-ir  # output LLVM IR
```

### Building with LLVM

For native code generation, build with the `llvm` feature:

```bash
# Requires LLVM 21 with llvm-config
cargo build --release --features llvm
```

> **Note**: Windows pre-built LLVM does not include `llvm-config`. Use MSYS2 LLVM or build from source. See [v0.4 Implementation](docs/IMPLEMENTATION_v0.4.md) for details.

### Example

```bmb
-- Function with contract
fn max(a: i32, b: i32) -> i32
  post ret >= a and ret >= b
= if a > b then a else b;

-- Precondition ensures non-zero division
fn safe_div(a: i32, b: i32) -> i32
  pre b != 0
= a / b;
```

### Verification Output

```
$ bmb verify max.bmb
✓ max: pre verified
✓ max: post verified

All 1 function(s) verified successfully.
```

## Project Structure

```
bmb/
├── bmb/               # Rust compiler implementation
│   ├── src/
│   │   ├── lexer/     # Token definitions (logos)
│   │   ├── parser/    # Parser (lalrpop)
│   │   ├── ast.rs     # AST definitions
│   │   ├── types/     # Type checker
│   │   ├── error.rs   # Error reporting
│   │   ├── smt/       # SMT-LIB2 generation
│   │   ├── verify/    # Contract verification
│   │   ├── interp/    # Tree-walking interpreter
│   │   └── repl/      # Interactive REPL
├── stdlib/            # Standard library (core, collections, io)
├── ecosystem/         # Dev tools (formatter, LSP, package manager)
├── runtime/           # Runtime support (memory, panic handling)
├── bootstrap/         # Self-hosted compiler in BMB
├── tools/             # Additional tooling
├── examples/          # Example programs
├── tests/             # Test suites
└── docs/              # Documentation
```

## Bootstrap Status (v0.10)

Self-hosted compiler components written in BMB:

| Component | Tests | Status |
|-----------|-------|--------|
| lexer.bmb | Tokens | ✅ Complete |
| parser.bmb | Syntax validation | ✅ Complete |
| parser_ast.bmb | S-expression AST | ✅ Complete |
| parser_test.bmb | 15 test categories | ✅ Complete |
| types.bmb | Type checking | ✅ Complete |
| mir.bmb | MIR foundation | ✅ Complete |
| lowering.bmb | AST→MIR transform | ✅ Complete |
| pipeline.bmb | End-to-end compile | ✅ Complete |
| llvm_ir.bmb | LLVM IR generation | ✅ Complete (93 tests) |

See [bootstrap/README.md](bootstrap/README.md) for details.

## Requirements

- Rust 1.70+
- Z3 Solver (for contract verification)

## Documentation

- [Language Specification](docs/SPECIFICATION.md)
- [Design Laws](docs/LAWS.md)
- [Roadmap](docs/ROADMAP.md)

### Implementation Notes

- [v0.1 Implementation](docs/IMPLEMENTATION_v0.1.md) - Lexer, Parser, AST
- [v0.2 Implementation](docs/IMPLEMENTATION_v0.2.md) - Type System, Contracts
- [v0.3 Implementation](docs/IMPLEMENTATION_v0.3.md) - Interpreter, REPL
- [v0.4 Implementation](docs/IMPLEMENTATION_v0.4.md) - MIR, LLVM Backend
- [v0.5 Implementation](docs/IMPLEMENTATION_v0.5.md) - Standard Library

## License

MIT
