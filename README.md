# BMB - Bare-Metal-Banter

A verified systems programming language with contract verification.

## Current Status: v0.3 Root

### Features

- **Lexer/Parser**: logos + lalrpop based tokenization and AST generation
- **Type System**: Basic types (i32, i64, f64, bool, unit), functions, let bindings
- **Contract Verification**: pre/post condition verification via SMT solver (Z3)
- **Interpreter**: Tree-walking interpreter for direct execution
- **REPL**: Interactive environment with rustyline
- **Error Reporting**: ariadne-based rich error messages

### Quick Start

```bash
# Build the compiler
cargo build --release

# Run a BMB program
bmb run examples/hello.bmb

# Start interactive REPL
bmb repl

# Check a file for type errors
bmb check examples/simple.bmb

# Verify contracts (requires Z3)
bmb verify examples/verify.bmb --z3-path /path/to/z3
```

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

## License

MIT
