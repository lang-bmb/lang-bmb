# BMB - Bare-Metal-Banter

A verified systems programming language with contract verification.

## Current Status: v0.4 Stem

### Features

- **Lexer/Parser**: logos + lalrpop based tokenization and AST generation
- **Type System**: Basic types (i32, i64, f64, bool, unit), functions, let bindings
- **Contract Verification**: pre/post condition verification via SMT solver (Z3)
- **Interpreter**: Tree-walking interpreter for direct execution
- **REPL**: Interactive environment with rustyline
- **MIR**: Middle Intermediate Representation for code generation
- **LLVM Backend**: Native code generation via inkwell (optional)
- **Error Reporting**: ariadne-based rich error messages

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
├── src/
│   ├── lexer/         # Token definitions (logos)
│   ├── parser/        # Parser (lalrpop)
│   ├── ast.rs         # AST definitions
│   ├── types/         # Type checker
│   ├── error.rs       # Error reporting
│   ├── smt/           # SMT-LIB2 generation
│   ├── verify/        # Contract verification
│   ├── interp/        # Tree-walking interpreter
│   └── repl/          # Interactive REPL
└── examples/          # Example programs
```

## Requirements

- Rust 1.70+
- Z3 Solver (for contract verification)

## Documentation

- [Language Specification](docs/SPECIFICATION.md)
- [Design Laws](docs/LAWS.md)
- [Roadmap](docs/ROADMAP.md)
- [v0.1 Implementation](docs/IMPLEMENTATION_v0.1.md)
- [v0.2 Implementation](docs/IMPLEMENTATION_v0.2.md)
- [v0.3 Implementation](docs/IMPLEMENTATION_v0.3.md)
- [v0.4 Implementation](docs/IMPLEMENTATION_v0.4.md)

## License

MIT
