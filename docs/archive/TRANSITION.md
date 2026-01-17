# BMB Rust-to-BMB Transition Guide

> Document tracking the self-hosting transition from Rust to BMB

---

## Overview

BMB is transitioning from a Rust-based compiler to a fully self-hosted BMB compiler. This document tracks the progress, blockers, and migration path.

### Current Status (v0.32.0)

| Component | Rust | BMB Bootstrap | Status |
|-----------|------|---------------|--------|
| Lexer | `bmb/src/lexer/` | `bootstrap/lexer.bmb` | ✅ Feature parity |
| Parser | `bmb/src/parser/` | `bootstrap/parser.bmb`, `parser_ast.bmb` | ✅ Feature parity |
| Type Checker | `bmb/src/types/` | `bootstrap/types.bmb` | ✅ Feature parity |
| MIR Generator | `bmb/src/mir/` | `bootstrap/mir.bmb`, `lowering.bmb` | ✅ Feature parity |
| LLVM Codegen | `bmb/src/codegen/` | `bootstrap/llvm_ir.bmb` | ✅ Feature parity |
| Optimizer | `bmb/src/mir/` | `bootstrap/optimize.bmb` | ✅ Feature parity |
| Pipeline | `bmb/src/build/` | `bootstrap/pipeline.bmb` | ✅ Feature parity |
| CLI | `bmb/src/main.rs` | `bootstrap/bmb_unified_cli.bmb` | ✅ Native compiler working |
| Interpreter | `bmb/src/interp/` | N/A | ❌ Not ported |
| REPL | `bmb/src/repl/` | N/A | ❌ Not ported |
| LSP | `bmb/src/lsp/` | N/A | ❌ Not ported |

### v0.32 Achievement: Native BMB Compiler

**bmb_unified_cli.bmb** (2,072 LOC): Self-contained native compiler
- Parses BMB source → Generates MIR → Outputs LLVM IR
- Native binary: 301KB standalone executable
- No Rust dependency for compilation pipeline

---

## Codebase Statistics

### Rust Compiler (`bmb/src/`)

| Directory | Files | Lines | Purpose |
|-----------|-------|-------|---------|
| `ast/` | 3 | ~2,500 | AST types |
| `codegen/` | 3 | ~5,000 | LLVM code generation |
| `error/` | 1 | ~200 | Error reporting |
| `interp/` | 2 | ~1,500 | Interpreter |
| `lexer/` | 2 | ~800 | Tokenizer |
| `mir/` | 5 | ~3,000 | Mid-level IR |
| `parser/` | 1 | ~300 | Parser utilities |
| `types/` | 3 | ~2,500 | Type checker |
| `main.rs` | 1 | ~1,500 | CLI entry point |
| **Total** | **~40** | **~18,700** | |

### BMB Bootstrap (`bootstrap/`)

| File | Lines | Purpose |
|------|-------|---------|
| `types.bmb` | 8,764 | Type checker |
| `llvm_ir.bmb` | 3,533 | LLVM IR generation |
| `lowering.bmb` | 2,959 | MIR lowering |
| `parser_ast.bmb` | 2,854 | Parser AST |
| `compiler.bmb` | 2,232 | Compiler orchestration |
| `pipeline.bmb` | 1,576 | Compilation pipeline |
| `mir.bmb` | 1,328 | MIR definitions |
| `selfhost_test.bmb` | 1,218 | Self-hosting tests |
| `parser.bmb` | 1,206 | Parser |
| `parser_test.bmb` | 1,137 | Parser tests |
| `optimize.bmb` | 1,112 | Optimizer |
| `utils.bmb` | 1,046 | Utilities |
| `lexer.bmb` | 781 | Lexer |
| Other | ~2,000 | Tests, demos |
| **Total** | **~30,800** | |

---

## Blockers for Full Rust Removal

### Critical Blockers

All critical blockers have been resolved as of v0.31.23.

### Resolved Blockers (v0.31.23)

| Blocker | Resolution | Version |
|---------|------------|---------|
| **No argv support** | Added `arg_count()` and `get_arg()` builtins | v0.31.22 |
| **No standalone CLI** | Created `bootstrap/bmb_cli.bmb` with native compilation | v0.31.23 |
| **Void type codegen** | Fixed Copy instruction handling for void types | v0.31.23 |
| **Runtime argv init** | Added `main()` wrapper in runtime.c | v0.31.23 |

### Remaining Blockers (Non-Critical)

| Blocker | Resolution | Version |
|---------|------------|---------|
| No File I/O | Added `read_file`, `write_file` builtins | v0.31.10 |
| No Process Exec | Added `system`, `exec` builtins | v0.31.11 |
| O(n²) string concat | StringBuilder pattern in lowering | v0.31.13 |

---

## Transition Phases

### Phase 1: Feature Parity (Complete)

✅ All core compiler components ported to BMB
✅ 393+ tests passing in bootstrap compiler
✅ Stage 2 verification complete (152 equivalence tests)
✅ Stage 3: 86% documented

### Phase 2: Archive Rust Code (Current)

The Rust codebase is preserved in the `archive/rust-v0.31` branch for:
- Historical reference
- Fallback if issues discovered
- Performance comparison baseline

### Phase 3: CLI Independence (Planned)

Required for full Rust removal:
1. Add `get_args()` / `arg_count()` builtins to BMB
2. Create native BMB CLI wrapper
3. Update build system for BMB-only compilation

### Phase 4: Full Removal (Future)

Once CLI independence achieved:
1. Remove `bmb/src/*.rs`
2. Remove `Cargo.toml`
3. Update CI/CD for BMB-only build

---

## Archive Information

### Archive Branch: `archive/rust-v0.31`

Created: 2026-01-08
Purpose: Preserve complete Rust implementation at v0.31.21

Contents:
- `bmb/` - Complete Rust compiler implementation
- `Cargo.toml` - Rust dependencies
- Full build system and tests

### How to Access Archived Rust Code

```bash
# Switch to archive branch
git checkout archive/rust-v0.31

# Build with Rust compiler
cargo build --release

# Run Rust compiler tests
cargo test
```

---

## Deprecation Status

The following Rust components are deprecated:

| Component | File | Replacement | Status |
|-----------|------|-------------|--------|
| Rust Lexer | `bmb/src/lexer/` | `bootstrap/lexer.bmb` | Deprecated |
| Rust Parser | `bmb/src/parser/` | `bootstrap/parser.bmb` | Deprecated |
| Rust Type Checker | `bmb/src/types/` | `bootstrap/types.bmb` | Deprecated |
| Rust MIR | `bmb/src/mir/` | `bootstrap/mir.bmb` | Deprecated |
| Rust Codegen | `bmb/src/codegen/` | `bootstrap/llvm_ir.bmb` | Deprecated |
| Rust CLI | `bmb/src/main.rs` | N/A | **Still Required** |
| Rust Interpreter | `bmb/src/interp/` | N/A | **Still Required** |

---

## Migration Timeline

| Phase | Target | Deliverable | Status |
|-------|--------|-------------|--------|
| v0.31 | Stage 3 + Gate #1 | Bootstrap parity verified | ✅ Complete |
| v0.32-alpha | Archive | Rust code preserved | ✅ Complete |
| v0.32-beta | CLI Independence | `get_args()` builtin | 📋 Planned |
| v0.32 | Full Removal | 0 lines Rust | 📋 Planned |

---

## FAQ

### Q: Why not remove Rust immediately?

The BMB bootstrap compiler lacks CLI argument support (`argv`). Without this, we cannot create a standalone compiler that reads source files from command line arguments.

### Q: Can I still build with Rust?

Yes. The Rust compiler is fully functional and required for running the BMB bootstrap via `bmb run bootstrap/compiler.bmb`.

### Q: When will Rust be fully removed?

After implementing `get_args()` builtin and a native BMB CLI wrapper. Target: v0.32 release.

### Q: How do I contribute to the transition?

See `bootstrap/` directory for BMB compiler components. Priority task: implement `get_args()` builtin in `bmb/src/interp/mod.rs`.
