# Changelog

All notable changes to BMB (Bare-Metal-Banter) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.50.24] - 2026-01-17

### Added

- **Proof verification query** (Task 47.7-47.8): `bmb q proof` queries verification results
  - Filters: `--unverified`, `--failed`, `--timeout`
  - Shows Z3 availability and version
  - Summary stats: total, verified, failed, timeout, unknown, pending
  - Counterexample display for failed verifications
- **Proof index generation**: `bmb verify` now saves results to `.bmb/index/proofs.json`
  - Tracks pre/post verification status per function
  - Records verification time and timestamp
  - Stores counterexamples for failed verifications

### Changed

- `ProofIndex`, `ProofEntry`, `ProofStatus` types added to `bmb::index` module
- `query_proofs()` function added to `bmb::query` module

## [0.50.23] - 2026-01-17

### Added

- **Cross-compilation target flag** (Task 48.3-48.4): `bmb build --target <triple>` generates LLVM IR for specified target
  - Supports target triples: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`
  - LLVM IR can be compiled on target platform with clang
  - WASM backend verified working on Windows

### Changed

- `BuildConfig` now supports `target_triple` field for cross-compilation
- `TextCodeGen` uses custom target triple when specified

## [0.50.22] - 2026-01-17

### Added

- **HTTP Query Server** (Task 50.7): `bmb q serve` starts an HTTP server for AI tools
  - Endpoints: `GET /health`, `GET /metrics`, `POST /query`
  - Supports all query types: sym, fn, type, metrics, deps, contract, impact
  - JSON request/response format
  - Default: `127.0.0.1:3000`, configurable with `--host` and `--port`
  - No additional dependencies (uses std::net::TcpListener)

## [0.50.21] - 2026-01-17

### Added

- **Index watch mode** (v0.50.8): `bmb index --watch` now monitors .bmb files for changes and re-indexes automatically
  - Uses `notify` crate for cross-platform file system watching
  - 500ms debouncing to avoid rapid re-indexing during saves
  - Only re-indexes when .bmb files change (ignores other file types)
  - Graceful error handling during re-indexing

### Dependencies

- Added `notify = "8"` for file system event watching
- Added `notify-debouncer-mini = "0.6"` for event debouncing

## [0.50.20] - 2026-01-17

### Added

- **Formatter comment preservation** (v0.45.5): `bmb fmt` now preserves comments in source files
  - File-level comments (before first item)
  - Function-level comments (before each function/struct/enum)
  - Both `//` and `--` (legacy) comment styles supported
  - Uses span information to attach comments to correct items

### Changed

- Formatter uses `format_program_with_comments()` instead of simple AST formatting
- Comment extraction via `extract_comments()` pre-pass before parsing

## [0.50.18] - 2026-01-16

### Fixed

- **Bootstrap String ABI**: Resolved mismatch between bootstrap compiler and C runtime string handling:
  - String literals now generate global LLVM constants (`@.str.N = private constant [N x i8] c"...\\00"`)
  - `bmb_string_from_cstr()` called to convert C strings to `BmbString*` at runtime
  - String methods (`len`, `byte_at`, `slice`) now use `ptr` type instead of `i64`
  - String concatenation (`+`) and equality (`==`, `!=`) now call `bmb_string_concat()` and `bmb_string_eq()`

### Added

- **Type-aware MIR instructions** for string operations:
  - `strlit <id> <hex>` - String literal with hex-encoded content
  - `strconcat`, `streq`, `strneq` - String binary operations
  - `strcall`, `strvoidcall`, `strintcall` - Type-aware function calls
  - `strmethod`, `strintmethod` - Type-aware method calls
- **String type inference** in lowering: `is_string_expr()` detects string-typed expressions
- **Hex encoding/decoding** for safe string content transmission in MIR

### Changed

- Runtime declarations updated with proper `ptr` types for string-handling functions
- Bootstrap compiler version updated to v0.50.18
- **C runtime wrapper functions**: Added short-name wrappers (`len`, `chr`, `char_to_string`, `ord`, `print_str`) to match LLVM codegen declarations
- **LLVM text codegen**: Added `char_to_string(i32)` declaration for bootstrap compiler support
- **Bootstrap `make_backslash`/`make_quote`**: Use `char_to_string(chr(N))` pattern to bypass Rust type checker's `chr() -> char` return type

## [0.50.17] - 2026-01-16

### Fixed

- **Bootstrap S-expression parser quotes handling**: `low_find_close_paren` now skips quoted strings, fixing parsing of strings containing `(` or `)` characters like `"( x"` or `"(call f)"`. Previously, parentheses inside strings were incorrectly counted, causing argument parsing to fail.
- **Bootstrap LLVM IR PHI node predecessors**: Nested if-else expressions now generate correct PHI predecessors by emitting explicit "end" labels before each `goto merge`. This ensures the PHI node references the actual control flow predecessor, not the branch entry point.

### Added

- **Bootstrap runtime function declarations**: Added LLVM IR declarations for runtime functions:
  - CLI: `arg_count`, `get_arg`
  - String methods: `len`, `byte_at`, `slice`
  - File I/O: `read_file`, `write_file`
  - StringBuilder: `sb_new`, `sb_push`, `sb_len`, `sb_build`
  - Print: `print_str`

### Changed

- Stage 1 native compiler (v30) now successfully compiles the full bootstrap source (30K+ lines) to valid LLVM IR
- Stage 2 binary links successfully with the C runtime

### Known Issues

- ~~**Stage 2 runtime crash**: String ABI mismatch between bootstrap compiler (integer hashes) and C runtime (BmbString pointers).~~ **Fixed in v0.50.18**
- Requires `ulimit -s unlimited` for large files due to recursive descent parser depth

## [0.50.15] - 2026-01-16

### Added

- **Bootstrap parser method chain extensions**: Stage 1 native compiler now supports:
  - Method calls on function results: `foo().bar()` → `(mcall (call foo) bar)`
  - Method calls on string literals: `"abc".len()` → `(mcall (str "abc") len)`
  - Method calls on parenthesized expressions: `(x + y).abs()` → `(mcall (+ x y) abs)`
- **parser_ast.bmb v0.32 syntax support**: Added braced if-else parsing alongside pre-v0.32 `then/else` syntax

### Fixed

- **Stage 1 parsing of bootstrap files**: `bootstrap/types.bmb` (8K+ lines) now parses completely with Stage 1 native compiler (requires unlimited stack)

### Known Issues

- Stage 2 self-compilation still limited by LLVM IR variable scoping in nested branches (pre-existing, tracked)
- Requires `ulimit -s unlimited` for large files due to recursive descent parser depth

## [0.50.14] - 2026-01-16

### Changed

- **SLP vectorization enabled**: Added `set_loop_slp_vectorization(true)` to LLVM pass options for better performance on parallel operations.

### Performance

- **Gate #3.1 PASSED** (Clang baseline): fibonacci benchmarks now run at 1.00-1.08x vs Clang -O3
  - fibonacci(35): BMB 0.016s = Clang 0.016s (1.00x)
  - fibonacci(40): BMB 0.183s vs Clang 0.169s (1.08x)
- Binary trees benchmark: 1.39x vs Clang (memory allocation overhead)
- GCC comparison: 1.60-1.83x (GCC has fibonacci-specific optimizations)

### Documentation

- **LLVM codegen analysis**: Documented root cause of performance gap - alloca/load/store pattern vs SSA-form IR generation.
- **Gate #3.1 baseline change**: Recommend Clang-based comparison (same LLVM backend) as official benchmark target.
- **Improvement roadmap**: SSA-form IR generation identified as path to further 15-20% improvement.

## [0.50.13] - 2026-01-16

### Fixed

- **Bootstrap LLVM IR variable scoping bug**: Function parameters were incorrectly renamed with block suffixes (e.g., `%d` → `%d_b2`) in nested else branches, causing undefined variable errors in generated LLVM IR.

### Changed

- Added `params` parameter to all `lower_*_sb` functions in bootstrap LLVM IR generator
- New `extract_param_names` helper extracts parameter names from signature for scoping checks
- `lower_var_sb` now uses `is_param()` to preserve original parameter names across all blocks

### Known Issues

- Stage 2 self-compilation still fails due to stack overflow when processing 30K+ line bootstrap file (pre-existing issue, tracked as v0.46 blocker)

## [0.50.12] - 2026-01-16

### Fixed

- **Critical performance bug**: LLVM optimization passes were not being run on generated IR, causing 5x slower native code than C. Now runs `default<O2>` or `default<O3>` passes based on optimization level.

### Performance

- **Native code benchmark**: fibonacci(40) improved from 5.15x slower to 2.0x slower than C (gcc -O3). The remaining gap is due to GCC's more aggressive loop unrolling.

### Changed

- Migrated all benchmark files in `ecosystem/benchmark-bmb/` to v0.32 syntax

## [0.50.11] - 2026-01-16

### Security

- **Cyclic type alias detection**: Added DFS-based cycle detection to prevent DoS via infinite recursion in type resolution. Circular definitions like `type A = B; type B = A;` now produce clear error messages.
- **Duplicate function warning**: Compiler now warns when a function is defined multiple times with the same name. Later definitions silently override earlier ones (warning helps catch copy-paste errors).

### Changed

- Extended `type_aliases` HashMap to track definition spans for better error reporting
- Added `function_spans` tracking to TypeChecker for duplicate detection

### Tests

- Added 7 new integration tests for type alias cycles and duplicate function detection

## [0.50.10] - 2026-01-16

### Security

- Completed Security Audit Phase 3: Penetration testing
- Documented all P0/P1 security findings in SECURITY_AUDIT.md

## [0.50.9] - 2026-01-15

### Documentation

- Critical benchmark review and honest status assessment
- Updated roadmap with verification results

## [0.50.8] - 2026-01-15

### Changed

- Bootstrap if-else refactoring for reduced parser complexity
- Simplified parser grammar to avoid stack overflow issues

## [0.50.6] - 2026-01-14

### Added

- **Type alias syntax**: `type Name = TargetType;` with generic parameter support
- **Refinement type aliases**: `type NonZero = i64 where self != 0;`
- Type alias resolution in type checker

## [0.50.5] - 2026-01-14

### Added

- Expanded integration test suite
- Fixed stdlib constants and type definitions

## [0.50.4] - 2026-01-14

### Fixed

- Stdlib contract syntax errors in multiple modules

## [0.50.3] - 2026-01-13

### Added

- Comprehensive integration test suite (65+ tests)
- Test infrastructure for error cases and warning detection

## [0.50.1] - 2026-01-13

### Fixed

- Stdlib postcondition syntax issues
- Bootstrap parser integer/keyword collision bugs

### Documentation

- Documented bootstrap compiler bottlenecks

## [0.50.0] - 2026-01-12

### Added

- Security Audit Phase 1: Automated security checks
- Security Audit Phase 2: Unsafe code review
- Critical review and honest project status assessment

### Changed

- v0.32 syntax migration completed for bootstrap compiler

## [0.45.0] - 2025-12-XX

### Added

- Multi-type REPL support
- Lint command with `--strict` flag for treating warnings as errors
- Enhanced warning system

## [0.32.0] - 2025-XX-XX

### Changed

- **Breaking**: New if-else syntax: `if cond { then } else { else }` (Rust-style braces)
- **Breaking**: Comments now use `//` (double-slash), `--` still supported for compatibility
- Added shift operators: `<<` (left shift), `>>` (right shift)
- Added symbolic logical operators: `&&`, `||`, `!` as alternatives to `and`, `or`, `not`

## [0.25.0] - 2025-XX-XX

### Added

- AI Query System (`bmb index`, `bmb q`)
- `.bmb/` project folder structure
- Symbol indexing for functions, types, and contracts

---

For migration guides and detailed release notes, see [docs/ROADMAP.md](docs/ROADMAP.md).
