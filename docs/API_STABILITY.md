# BMB API Stability

**Version**: v0.45
**Date**: 2026-01-12
**Status**: Frozen for v1.0.0-beta

---

## 1. Overview

This document defines BMB's stability guarantees, experimental features, and deprecation policy. It serves as the contract between BMB and its users regarding API stability from v1.0.0 onwards.

---

## 2. Stability Tiers

### 2.1 Stable (No Breaking Changes)

Features marked as **Stable** will not have breaking changes after v1.0.0:

| Category | Stable Components |
|----------|-------------------|
| **Language Syntax** | All syntax in SPECIFICATION.md v0.32 |
| **Type System** | Primitives (i8-i128, u8-u128, f64, bool, String, char) |
| **Type System** | Generics (`<T>`, bounds, where clauses) |
| **Type System** | Enums, Structs, Type aliases |
| **Type System** | Option (`T?`), Result (`Result<T,E>`) |
| **Contracts** | `pre`, `post`, `invariant`, `where`, `pure`, `@trust` |
| **Control Flow** | `if`/`else`, `match`, `while`, `for`/`in`, `loop` |
| **Operators** | All operators in SPECIFICATION.md v0.32 |
| **CLI Commands** | `run`, `check`, `verify`, `build`, `test`, `parse`, `repl`, `fmt`, `lint` |
| **stdlib/core** | `num`, `bool`, `option`, `result` |
| **stdlib/string** | `String`, `StringBuilder` |
| **stdlib/array** | `Array<T>`, `Vec<T>` |
| **stdlib/io** | `print`, `println`, `read_line`, `read_file`, `write_file` |
| **stdlib/test** | `assert`, `assert_eq`, `test` attribute |

### 2.2 Experimental (May Change)

Features marked as **Experimental** may change or be removed:

| Feature | Description | Stability Target |
|---------|-------------|------------------|
| `@check` attribute | Runtime-only contract verification | v1.1 |
| `todo` keyword | Placeholder implementation | v1.1 |
| WASM backend | `--target wasm32` compilation | v1.1 |
| `trait` system | Generic trait bounds | v1.2 |
| `impl` blocks | Method implementations | v1.0 (becoming stable) |
| Package manager (`gotgan`) | Dependency management | v1.1 |
| `stdlib/process` | Shell execution, `exec`, `shell` | v1.1 |
| `stdlib/parse` | Number parsing utilities | v1.1 |

### 2.3 Internal (No Stability Guarantee)

These are implementation details with no stability guarantee:

- MIR format and representation
- LLVM IR generation details
- SMT-LIB2 output format
- Index file format (`.bmb-index/`)
- Bootstrap compiler internals

---

## 3. CLI Stability

### 3.1 Stable Commands

```bash
bmb run <file.bmb>              # Stable
bmb check <file.bmb>            # Stable
bmb verify <file.bmb>           # Stable
bmb build <file.bmb> -o <out>   # Stable
bmb test <file.bmb>             # Stable
bmb parse <file.bmb>            # Stable
bmb repl                        # Stable
bmb fmt <file.bmb>              # Stable (v0.9.0)
bmb lint <file.bmb>             # Stable (v0.45)
```

### 3.2 Stable Flags

| Flag | Description | Status |
|------|-------------|--------|
| `-o <file>` | Output file | Stable |
| `--emit-mir` | Output MIR | Stable |
| `--target <target>` | Compilation target | Experimental |
| `-O0`, `-O1`, `-O2`, `-O3` | Optimization levels | Stable |
| `--features <list>` | Feature flags | Experimental |

### 3.3 Exit Codes

| Code | Meaning | Status |
|------|---------|--------|
| 0 | Success | Stable |
| 1 | Compilation error | Stable |
| 2 | Runtime error | Stable |
| 3 | Verification failure | Stable |

---

## 4. Deprecation Policy

### 4.1 Deprecation Process

1. **Announcement**: Deprecated features are announced in release notes
2. **Warning Period**: Deprecated features emit warnings for 2 minor versions
3. **Removal**: Features are removed in the next major version

### 4.2 Deprecation Timeline

| Version | State |
|---------|-------|
| v1.x.0 | Feature deprecated, warning emitted |
| v1.x+1.0 | Warning continues |
| v1.x+2.0 | Warning continues (final warning) |
| v2.0.0 | Feature removed |

### 4.3 Exception: Security Fixes

Security vulnerabilities may be fixed immediately without deprecation period if:
- The fix changes behavior
- The original behavior was a security risk
- A migration path is documented

### 4.4 Currently Deprecated

| Feature | Deprecated Since | Removal Target | Alternative |
|---------|------------------|----------------|-------------|
| `--` comments | v0.32 | v2.0 | `//` comments |
| `if X then Y else Z` | v0.32 | v2.0 | `if X { Y } else { Z }` |
| `Option<T>` | v0.32 | Never (kept for compatibility) | `T?` (preferred) |

---

## 5. Breaking Change Definition

A **breaking change** is any modification that causes previously valid code to:
- Fail to compile
- Produce different runtime behavior
- Emit new errors where none existed

### 5.1 NOT Breaking Changes

The following are NOT considered breaking changes:
- New compiler warnings
- Performance improvements/regressions
- New features that don't affect existing code
- Bug fixes that correct incorrect behavior
- New compiler error messages with better wording

### 5.2 Allowed After v1.0

- Adding new keywords (reserved but unused)
- Adding new operators (reserved but unused)
- Adding new stdlib modules
- Adding new CLI commands/flags
- Improving error messages
- Performance optimizations

---

## 6. Version Numbering

BMB follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

| Component | Meaning |
|-----------|---------|
| MAJOR | Breaking changes to stable API |
| MINOR | New features, deprecations, experimental changes |
| PATCH | Bug fixes, security patches |

### 6.1 Pre-1.0 Policy

During v0.x development:
- Breaking changes may occur in minor versions
- This stability document applies from v1.0.0 onwards

---

## 7. Migration Guides

When breaking changes occur (in major versions), migration guides will be provided:
- `docs/migrations/v2.0.md` (future)
- Automated migration tool: `bmb migrate <file.bmb>`

---

## 8. Platform Support

### 8.1 Tier 1 (Full Support)

| Platform | Architecture |
|----------|--------------|
| Linux | x86_64 |
| Windows | x86_64 |
| macOS | x86_64, aarch64 |

### 8.2 Tier 2 (Best Effort)

| Platform | Architecture |
|----------|--------------|
| Linux | aarch64 |
| WASM | wasm32 (experimental) |

---

## 9. Feedback

Report stability concerns or request changes:
- GitHub Issues: `github.com/bmb-lang/bmb/issues`
- RFC Process: `docs/RFC/`

---

## Changelog

- **v0.45 (2026-01-12)**: Added `bmb lint` and `bmb fmt` to stable CLI commands
- **v0.43 (2026-01-12)**: Initial API stability document
