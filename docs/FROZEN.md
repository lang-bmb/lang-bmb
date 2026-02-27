# Rust Compiler Freeze — v0.94

> **Date**: 2026-02-14
> **Version**: v0.93.30
> **Status**: FROZEN

## What This Means

The Rust BMB compiler (`bmb/src/`) is **frozen**. All new compiler development
happens in `bootstrap/compiler.bmb` using the bootstrap-first workflow.

## Allowed Changes to Rust Code

| Action | Allowed? | Notes |
|--------|----------|-------|
| New features | ❌ No | Implement in compiler.bmb |
| Bug fixes | ⚠️ Only if blocking bootstrap | Minimal, targeted fixes only |
| New tests | ❌ No | Write golden tests instead |
| Optimization passes | ❌ No | Implement in compiler.bmb's codegen |
| Dependency updates | ⚠️ Security only | Keep `cargo test` passing |
| Refactoring | ❌ No | Code is frozen |

## Reason for Freeze

BMB's goal is to be a **self-hosting language** independent of Rust. The bootstrap
compiler (`bootstrap/compiler.bmb`) has achieved:

- 3-Stage Fixed Point (Stage 2 IR == Stage 3 IR)
- Golden binary bootstrap (Rust-free compilation)
- 60 golden tests passing
- 5-pass IR optimization pipeline (37.5% IR reduction)
- Full Rust-free development workflow (`scripts/bmb-dev.sh`)

All further development uses the bootstrap-first workflow:

```bash
# Development without Rust
1. Edit bootstrap/compiler.bmb
2. ./scripts/bmb-dev.sh full
3. If all tests pass, commit
```

## Development Workflow After Freeze

```
[Before — Rust-centric]
1. Modify bmb/src/*.rs
2. cargo test --release
3. Port to bootstrap/compiler.bmb
4. 3-Stage verification

[After — BMB-centric]
1. Modify bootstrap/compiler.bmb directly
2. ./scripts/bmb-dev.sh full
3. Commit (no Rust changes needed)
```

## Cargo Maintenance

The Rust compiler still needs to compile and pass tests for existing infrastructure:

```bash
# Verify Rust code still builds and passes
cargo test --release
cargo clippy --all-targets -- -D warnings
```

This is **maintenance only** — no new functionality is added to the Rust codebase.

## Key Files

| File | Status | Purpose |
|------|--------|---------|
| `bmb/src/**/*.rs` | FROZEN | Rust compiler (maintenance only) |
| `bootstrap/compiler.bmb` | ACTIVE | Bootstrap compiler (all new work) |
| `bootstrap/optimize.bmb` | ACTIVE | MIR optimization passes |
| `bootstrap/lowering.bmb` | ACTIVE | AST → MIR lowering |
| `golden/windows-x64/bmb.exe` | UPDATED | Golden binary (v0.93.30) |
| `scripts/bmb-dev.sh` | ACTIVE | Development workflow |
| `tests/bootstrap/*.bmb` | ACTIVE | Golden tests |

## Version at Freeze

- BMB Version: v0.93.30
- Rust Compiler: 2845 lib tests + 47 main tests + 2319 integration tests + 23 doc tests
- Bootstrap: 42,892 LOC, 43,680 lines Stage 2 IR
- Golden Tests: 60/60 passing
- Golden Binary: v0.93.30, 610KB
