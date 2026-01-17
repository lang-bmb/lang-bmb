# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BMB (Bare-Metal-Banter) is an AI-native systems programming language with contract-based verification. The project has two compiler implementations:

1. **Rust compiler** (`bmb/`) - Full-featured reference implementation (being deprecated)
2. **Bootstrap compiler** (`bootstrap/`) - Self-hosted compiler written in BMB itself (~30K lines)

## Build Commands

```bash
# Build the compiler
cargo build --release

# Build with LLVM backend (requires LLVM 21 with llvm-config)
cargo build --release --features llvm

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Check for warnings (must pass: 0 clippy + 0 doc warnings)
cargo clippy --all-targets
cargo doc --no-deps
```

## CLI Usage

```bash
bmb run <file.bmb>              # Run with interpreter
bmb check <file.bmb>            # Type check only
bmb verify <file.bmb>           # Contract verification (requires Z3)
bmb parse <file.bmb>            # Dump AST (JSON or S-expr)
bmb build <file.bmb> -o out     # Native compile (requires LLVM feature)
bmb build <file.bmb> --emit-mir # Output MIR
bmb test <file.bmb>             # Run tests in file
bmb repl                        # Interactive REPL (v0.45: multi-type support)
bmb fmt <file.bmb>              # Format source file
bmb lint <file.bmb>             # Lint source file (v0.45: warnings check)
bmb lint <file.bmb> --strict    # Treat warnings as errors
bmb lsp                         # Start Language Server
bmb index                       # Generate AI query index (.bmb/index/)
bmb q <query>                   # Query project index
```

## Developer Tools (`tools/`)

```bash
# BMB Syntax Migration (pre-v0.32 → v0.32)
node tools/migrate_syntax.mjs <files...> --stats   # Preview changes
node tools/migrate_syntax.mjs <files...> --apply   # Apply changes
# Handles: -- → //, if then else → if { } else { }, Option<T> → T?

# Rust to BMB Conversion
node tools/rust_to_bmb.mjs path/to/rust/*.rs --stats   # Preview
node tools/rust_to_bmb.mjs path/to/rust/*.rs --apply   # Convert
# Converts: fn, struct, enum, impl, match, Option, Result
```

## `.bmb/` Project Folder (v0.25 AI Query System)

The `.bmb/` folder contains project-specific data for AI tools and caching:

```
.bmb/
└── index/                    # AI Query Index (bmb index)
    ├── manifest.json         # Project metadata (version, counts)
    ├── symbols.json          # All symbols (functions, types, structs)
    ├── functions.json        # Function details (signatures, contracts)
    └── types.json            # Type definitions
```

**Usage:**
```bash
bmb index                     # Generate/update index
bmb q "function binary_search"  # Query functions
bmb q "struct Vec"            # Query types
bmb q "contract pre"          # Query contracts
```

**Planned Extensions:**
- `.bmb/config.toml` - Project configuration
- `.bmb/cache/` - Compilation cache
- `.bmb/deps/` - Dependency cache

## Architecture

### Compilation Pipeline
```
Source → Lexer (logos) → Parser (lalrpop) → AST → Types → MIR → Codegen (LLVM/WASM)
                                                    ↓
                                             SMT Verification (Z3)
```

### Key Modules (`bmb/src/`)
- `lexer/` - Token definitions using logos derive macros
- `parser/` - Grammar defined in `grammar.lalrpop` (LR(1) parser)
- `ast/` - AST nodes with span tracking; S-expression output in `output.rs`
- `types/` - Hindley-Milner type inference with generics and refinement types
- `mir/` - Middle IR with `lower.rs` (AST→MIR) and `optimize.rs`
- `codegen/` - LLVM backend (`llvm.rs`, requires feature flag) and WASM text (`wasm_text.rs`)
- `smt/` - SMT-LIB2 generation for Z3 verification
- `interp/` - Tree-walking interpreter (still required, not deprecated)

### Bootstrap Compiler (`bootstrap/`)
Self-hosted compiler components in BMB:
- `lexer.bmb` - Tokenizer with encoded token format: `kind * 1000000 + end_pos`
- `parser.bmb` / `parser_ast.bmb` - Recursive descent parser producing S-expressions
- `types.bmb` - Type checker with generic support (~530 tests)
- `mir.bmb` / `lowering.bmb` - AST to MIR transformation
- `llvm_ir.bmb` - LLVM IR text generation
- `compiler.bmb` - Unified pipeline entry point

**Note**: Bootstrap compiler currently uses pre-v0.32 syntax. Run with older compiler version or wait for migration.

Run bootstrap tests (requires pre-v0.32 compatible compiler):
```bash
bmb run bootstrap/lexer.bmb     # Should output 777...888...<count>...999
bmb run bootstrap/types.bmb     # ~530 tests
```

### Migration Tool (`tools/migrate_syntax.mjs`)
Converts BMB source files from pre-v0.32 to v0.32 syntax:
```bash
node tools/migrate_syntax.mjs <files...> --stats   # Preview changes
node tools/migrate_syntax.mjs <files...> --apply   # Apply changes
```
Handles: `--` → `//` comments, `if then else` → `if { } else { }`, `Option<T>` → `T?`

## Language Syntax (v0.32)

```bmb
// Function with contracts (v0.32: braced if-else)
fn max(a: i64, b: i64) -> i64
  post ret >= a && ret >= b
= if a > b { a } else { b };

// Refinement type
type NonZero = i64 where self != 0;

// Generic enum
enum Option<T> { Some(T), None }

// Closure
fn apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x);

// Shift operators (v0.32)
fn shift_example(x: i64) -> i64 = (x << 2) >> 1;
```

## Codebase Conventions

- Expression-based language: everything returns a value
- Comments: `//` (preferred, v0.32) or `--` (legacy, still supported)
- Function body follows `=` sign
- Contract annotations: `pre`, `post`, `@trust`, `@check`
- Type parameters use angle brackets: `<T>`, `<K, V>`

## v0.32 Syntax Changes

The Rust compiler (`bmb/`) supports v0.32 syntax:
- **Comments**: `//` preferred (double-slash), `--` still supported
- **If-else**: `if cond { then } else { else }` (braced, Rust-style)
- **Shift operators**: `<<` (left shift), `>>` (right shift)
- **Symbolic logical**: `&&` / `||` / `!` as alternatives to `and` / `or` / `not`

**Note**: Bootstrap compiler migrated to v0.32 syntax (Phase 38.1 complete, 2026-01-12).

## Standard Library (`stdlib/`)

Modules: `core/` (num, bool, option, result), `string/`, `array/`, `io/`, `process/`, `test/`

Some stdlib functions are interpreter builtins (I/O, process, StringBuilder).

## Workspace Structure

- Workspace members: `bmb`, `ecosystem/gotgan` (package manager)
- Submodules in `ecosystem/`: vscode-bmb, tree-sitter-bmb, playground, etc.
- Initialize submodules: `git submodule update --init --recursive`

## Development Guidelines

### Package-First Development

When implementing compiler, tools, bootstrapping, or ecosystem features:

1. **Identify reusable units** - Extract generic, reusable components from implementations
2. **Package to `ecosystem/gotgan-packages/`** - Publish as BMB packages for community reuse
3. **Dogfood the ecosystem** - Use gotgan packages in the compiler/bootstrap itself when practical

Examples of packageable units:
- Data structures (HashMap, BTreeMap, Vec variants)
- Parsing utilities (lexer helpers, AST builders)
- Code generation helpers (IR builders, text formatters)
- Testing utilities (assertion helpers, benchmark harnesses)
- String processing (formatting, escaping, interning)

### Rust Package Porting Workflow

When a BMB package is needed, leverage the mature Rust ecosystem:

1. **Find Rust crate** - Identify equivalent Rust crate (crates.io)
2. **Migrate with tool** - Use `tools/rust_to_bmb.mjs` for initial conversion:
   ```bash
   node tools/rust_to_bmb.mjs path/to/rust/*.rs --apply
   ```
3. **Add contracts** - Enhance with BMB's contract system:
   - Add `pre` conditions for input validation
   - Add `post` conditions for output guarantees
   - Use refinement types where appropriate
4. **Optimize** - Apply BMB-specific optimizations:
   - Leverage compile-time verification
   - Simplify where Rust's ownership isn't needed
   - Use BMB idioms (expression-based, pattern matching)
5. **Package** - Publish to `ecosystem/gotgan-packages/`:
   ```
   ecosystem/gotgan-packages/<package-name>/
   ├── gotgan.toml
   ├── src/
   │   └── lib.bmb
   └── tests/
   ```
6. **Document origin** - Credit original Rust crate in package metadata

This workflow accelerates ecosystem growth while adding BMB's verification advantages.

## LLVM Testing in WSL Ubuntu

**⚠️ IMPORTANT**: Native compilation benchmarks MUST be run in WSL Ubuntu with LLVM installed.
Windows lacks LLVM support. Interpreter-only benchmarks give unfair comparisons (2-4x slower).

### WSL Ubuntu Setup (One-time)

```bash
# 1. Enter WSL
wsl

# 2. Install LLVM 21 (or latest)
sudo apt update
sudo apt install -y llvm-21 llvm-21-dev clang-21 lld-21
# Or use LLVM's official repo:
# wget https://apt.llvm.org/llvm.sh && chmod +x llvm.sh && sudo ./llvm.sh 21

# 3. Set environment variables
export LLVM_SYS_210_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# 4. Verify LLVM
llvm-config --version  # Should show 21.x.x

# 5. Build BMB with LLVM feature
cd /mnt/d/data/lang-bmb
cargo build --release --features llvm

# 6. Verify native compilation works
./target/release/bmb build examples/hello.bmb -o hello
./hello
```

### Running Native Benchmarks

```bash
# In WSL Ubuntu:
cd /mnt/d/data/lang-bmb/ecosystem/benchmark-bmb

# Compile BMB benchmarks to native binaries (not interpreter!)
for dir in benches/*/*/bmb; do
  if [ -f "$dir/main.bmb" ]; then
    echo "Compiling $dir..."
    /mnt/d/data/lang-bmb/target/release/bmb build "$dir/main.bmb" -o "$dir/main" 2>/dev/null || echo "Failed: $dir"
  fi
done

# Run benchmark suite with native binaries
./runner/target/release/benchmark-bmb run all -i 5 -w 2

# Verify Gate #3.1
./runner/target/release/benchmark-bmb gate 3.1 -v
```

### Benchmark Gate Criteria

| Gate | Requirement | Command |
|------|-------------|---------|
| Gate #3.1 | Compute ≤ 1.10x C, Contract ≤ 0.90x C | `benchmark-bmb gate 3.1` |
| Gate #3.2 | All Benchmarks Game ≤ 1.05x C | `benchmark-bmb gate 3.2` |
| Gate #3.3 | 3+ benchmarks faster than C | `benchmark-bmb gate 3.3` |
| Gate #4.1 | Self-compile < 60s | ✅ Already passing (0.56s) |

### Known Issues

- **Windows**: LLVM not available, use WSL for native benchmarks
- **Interpreter mode**: `bmb run` is 2-4x slower than native - DO NOT use for benchmarks
- **Binary path**: WSL uses `/mnt/d/...` for Windows paths