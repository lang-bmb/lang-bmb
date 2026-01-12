# BMB v1.0.0-beta Release Notes

**Release Date**: TBD (after v0.44 completion)
**Codename**: Golden

---

## Overview

BMB v1.0.0-beta marks the first beta release of the BMB systems programming language. This release represents a major milestone: the language specification is frozen, the bootstrap compiler is self-hosting, and all core features are production-ready.

---

## Highlights

### Contract-Based Verification
BMB brings compile-time verification to systems programming:
```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0
  post ret * b == a
= a / b;
```

### Self-Hosted Compiler
The BMB compiler is written in BMB itself (~30K lines), demonstrating the language's capability for large-scale systems development.

### Performance
- **Self-compile time**: 0.56 seconds for 30K LOC
- **Type checking**: 4.4 seconds with full verification
- **Native compilation**: Competitive with C (requires LLVM backend)

### Comprehensive Test Suite
- 1,753 tests across Rust compiler and bootstrap
- CI/CD on Ubuntu, Windows, macOS
- Performance regression detection (2% threshold)

---

## Language Features

### Types
| Feature | Status |
|---------|--------|
| Primitive types (i8-i128, u8-u128, f64, bool, char) | Stable |
| Generics (`<T>`, `<K, V>`) | Stable |
| Enums with variants | Stable |
| Structs with fields | Stable |
| Option type (`T?`) | Stable |
| Result type (`Result<T, E>`) | Stable |
| Refinement types (`type NonZero = i64 where self != 0`) | Stable |
| Type aliases | Stable |

### Contracts
| Feature | Status |
|---------|--------|
| Preconditions (`pre`) | Stable |
| Postconditions (`post`) | Stable |
| Type refinements (`where`) | Stable |
| Trust annotation (`@trust`) | Stable |
| Pure functions (`pure`) | Stable |
| Invariants (`invariant`) | Stable |
| Runtime checks (`@check`) | Experimental |

### Control Flow
| Feature | Status |
|---------|--------|
| If-else expressions | Stable |
| Match expressions | Stable |
| While loops | Stable |
| For-in loops | Stable |
| Loop expressions | Stable |
| Break/Continue | Stable |

### Operators
| Category | Operators |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/`, `%` |
| Overflow-safe | `+%`, `-%`, `*%` (wrapping) |
| Saturating | `+|`, `-|`, `*|` |
| Checked | `+?`, `-?`, `*?` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical | `&&`, `||`, `!` or `and`, `or`, `not` |
| Bitwise | `band`, `bor`, `bxor`, `bnot` |
| Shift | `<<`, `>>` |

---

## Standard Library

### Stable Modules
- `core/num` - Number operations
- `core/bool` - Boolean operations
- `core/option` - Option type utilities
- `core/result` - Result type utilities
- `string/` - String and StringBuilder
- `array/` - Array and Vec
- `io/` - File I/O, console I/O
- `test/` - Testing utilities

### Experimental Modules
- `process/` - Shell execution
- `parse/` - Number parsing

---

## CLI Tools

### Commands
```bash
bmb run <file.bmb>              # Run with interpreter
bmb check <file.bmb>            # Type check only
bmb verify <file.bmb>           # Contract verification (Z3)
bmb build <file.bmb> -o <out>   # Native compile (LLVM)
bmb test <file.bmb>             # Run tests
bmb parse <file.bmb>            # Dump AST
bmb repl                        # Interactive REPL
```

### Package Manager (Experimental)
```bash
gotgan new <project>            # Create new project
gotgan build                    # Build project
gotgan add <package>            # Add dependency
gotgan run                      # Build and run
```

---

## Breaking Changes from v0.x

### Syntax Changes (v0.32)
| Old | New | Migration |
|-----|-----|-----------|
| `-- comment` | `// comment` | Automatic |
| `if X then Y else Z` | `if X { Y } else { Z }` | Automatic |

Run `bmb migrate <file.bmb>` or `node tools/migrate_syntax.mjs <file> --apply` for automatic conversion.

---

## Known Issues

### LLVM Backend
- Windows native compilation requires WSL Ubuntu with LLVM 21
- See `docs/WSL_LLVM_SETUP.md` for setup instructions

### Benchmark Status
- Gate #3.1 (native performance): Requires LLVM, blocked on Windows
- Gate #4.1 (self-compile < 60s): PASSED (0.56s)

---

## Upgrade Guide

### From v0.39 or earlier
1. Update syntax: `bmb migrate --apply <files...>`
2. Replace `--` comments with `//`
3. Convert `if then else` to `if { } else { }`

### From v0.40+
- No breaking changes, direct upgrade supported

---

## Platform Support

### Tier 1 (Fully Tested)
- Linux x86_64
- Windows x86_64
- macOS x86_64/aarch64

### Tier 2 (Best Effort)
- Linux aarch64
- WASM32 (experimental)

---

## Documentation

- [Language Reference](LANGUAGE_REFERENCE.md)
- [Specification](SPECIFICATION.md)
- [API Stability](API_STABILITY.md)
- [Getting Started](tutorials/GETTING_STARTED.md)
- [Contract Programming](tutorials/CONTRACT_PROGRAMMING.md)
- [From Rust Guide](tutorials/FROM_RUST.md)

---

## Contributors

Thanks to all contributors who made this release possible.

---

## What's Next

### v1.0.0 (Stable)
- Complete benchmark verification
- Full self-hosting without Rust dependency
- Extended trait system

### v1.1.0
- WASM backend stabilization
- Package manager stabilization
- Additional stdlib modules

---

## Installation

### From Source
```bash
git clone https://github.com/bmb-lang/bmb
cd bmb
cargo build --release
```

### With LLVM (Native Compilation)
```bash
# Requires LLVM 21
cargo build --release --features llvm
```

### Binary Downloads
Coming soon via GitHub Releases.

---

## Feedback

- GitHub Issues: Report bugs and request features
- RFC Process: Propose language changes via `docs/RFC/`
