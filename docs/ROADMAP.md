# BMB Language Roadmap: v0.1 → v1.0.0-rc

> Progressive difficulty progression • Complete ecosystem • Self-hosting completion • Rust removal • C/Rust performance parity

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Maturity Milestones](#programming-language-maturity-milestones)
3. [Version Overview](#version-overview)
4. [Completed Phases (v0.1-v0.29)](#completed-phases-v01-v029)
5. [Remaining Phases (v0.30-v1.0.0-rc)](#remaining-phases-v030-v100-rc)
6. [Ecosystem Repositories](#ecosystem-repositories)
7. [Success Criteria](#success-criteria)

---

## Design Principles

| Principle | Description | Reference |
|-----------|-------------|-----------|
| **Gradual Progression** | Minimize difficulty gaps between versions | Gleam's 5-year 0.x journey |
| **Built-in Tooling** | `bmb fmt`, `bmb lsp` work without separate installation | Gleam pattern |
| **Small Releases** | Split large features across minor versions | Zig pattern |
| **0.x = Experimental** | Breaking changes allowed; 1.0 = stability promise | Common practice |
| **Package-First** | All reusable code registered in gotgan | Ecosystem growth |
| **Performance Proof** | Benchmark verification against C/Rust | Contract-based optimization |
| **Self-Hosting** | Complete Rust removal, BMB-only composition | Rust (OCaml→Rust 2011) |

### Non-Negotiable Priorities

| Priority | Principle | Description |
|----------|-----------|-------------|
| **Performance** | Maximum Performance Syntax | Syntax must enable maximum performance without constraints |
| **Correctness** | Compile-Time Verification | If compile-time checking is possible, it MUST be in the language spec |
| **Self-Hosting** | Bootstrap Completion | BMB compiler must compile itself. No Rust dependency after v0.30 |

### Versioning Scheme

```
v0.MAJOR.MINOR
  │      │
  │      └── Small improvements, bug fixes, feature additions
  └────────── Major milestones (Seed, Sprout, Root, ...)
```

---

## Programming Language Maturity Milestones

> References: [Wikipedia - Self-hosting compilers](https://en.wikipedia.org/wiki/Self-hosting_(compilers)), [Earthly - Programming Language Tooling](https://earthly.dev/blog/programming-language-improvements/)

### Required Milestones

| Stage | Component | Description | BMB Status | Target |
|-------|-----------|-------------|------------|--------|
| **1. Compiler** | Lexer + Parser | Source code parsing | ✅ Complete | v0.1 |
| **2. Type System** | Type Checker | Static type checking | ✅ Complete | v0.2 |
| **3. Code Generation** | Code Generator | Native/WASM output | ✅ Complete | v0.4/v0.12 |
| **4. Standard Library** | stdlib | "Batteries Included" | ✅ Complete | v0.6 |
| **5. Package Manager** | Package Manager | Dependency management | ✅ Complete | v0.8 |
| **6. Toolchain** | Tooling | fmt, lsp, test, lint | ✅ Complete | v0.7 |
| **7. IDE Support** | LSP + Extensions | VS Code, IntelliJ, etc. | ✅ Complete | v0.9 |
| **8. Self-Hosting** | Bootstrap | Compile itself | 🔄 In Progress | v0.30 |
| **9. Benchmarks** | Performance Suite | C/Rust performance proof | ✅ Complete | v0.28 |
| **10. Documentation** | Documentation | Reference, tutorials | 📋 Planned | v0.31 |
| **11. Playground** | Online Editor | Browser execution environment | ✅ Complete | v0.24 |
| **12. Community** | Ecosystem | Packages, contributors, users | 📋 Planned | v1.0 |

### Self-Hosting Definition (Bootstrap Completion Criteria)

> "A self-hosting compiler is a compiler capable of compiling its own source code." - Wikipedia

| Condition | Description | Status |
|-----------|-------------|--------|
| **Stage 1** | Build BMB compiler with Rust compiler | ✅ Complete |
| **Stage 2** | Build BMB compiler with BMB compiler | 🔄 In Progress |
| **Stage 3** | Rebuild with Stage 2 output (identical binary) | 📋 Planned |
| **Rust Removal** | Remove all Rust code, BMB-only composition | 📋 Planned (v0.30) |

**Historical References**:
- Rust: Started with OCaml → First self-compile April 2011 (1 hour)
- Go: Bootstrapped 1.5 with Go 1.4 (GCC-Go also possible)
- Lisp: First self-hosting compiler at MIT 1962

---

## Version Overview

| Version | Codename | Goal | Status |
|---------|----------|------|--------|
| v0.1-v0.9 | Foundation | Compiler, tools, ecosystem (Rust) | ✅ Complete |
| v0.10-v0.18 | Language | Generics, modules, methods | ✅ Complete |
| v0.19-v0.23 | Bootstrap | MIR completion, Stage 1/2 verification | ✅ Complete |
| v0.24-v0.29 | Polish | Examples, AI Query, benchmarks, optimization | ✅ Complete |
| **v0.30** | **Pure** | **Rust code complete removal (Self-Hosting)** | 📋 Planned |
| **v0.31** | **Docs** | **Documentation completion + website launch** | 📋 Planned |
| **v0.32** | **Ecosystem** | **100+ packages + community** | 📋 Planned |
| **v1.0.0-rc** | **Golden** | **Final verification + stability promise** | 📋 Planned |

---

## Completed Phases (v0.1-v0.29)

> Summary of 29 completed versions representing the foundation-building phase

### Phase 1: Compiler Foundation (v0.1-v0.4)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.1 | Seed | Minimal parser + type checker | Lexer (logos), Parser (lalrpop), AST, CLI (clap) |
| v0.2 | Sprout | SMT integration + basic verification | Type checker, SMT-LIB generation, Z3 integration, Error reporting (ariadne) |
| v0.3 | Root | Interpreter + REPL | Tree-walking interpreter, REPL (rustyline), Stack trace |
| v0.4 | Stem | Code generation (LLVM) | MIR (CFG-based IR), LLVM IR generation, Native build |

### Phase 2: Language & Tooling (v0.5-v0.9)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.5 | Branch | Language extensions + Bootstrap start | Pattern matching, Generic basics, Module system, Attributes |
| v0.6 | Leaf | Standard library foundation (100+ functions) | core (50+), string (25+), math (30+), io (10+) |
| v0.7 | Bloom | Tooling foundation | bmb fmt, bmb lsp, bmb test, action-bmb GitHub Action |
| v0.8 | Fruit | Package manager (gotgan) | gotgan init/build/add, Dependency resolution (SAT solver) |
| v0.9 | Harvest | Ecosystem | tree-sitter-bmb, vscode-bmb, playground, lang-bmb-site |

### Phase 3: Component Packaging & WASM (v0.10-v0.12)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.10 | Sunrise | Component packaging | bmb-lexer, bmb-parser, bmb-types, bmb-smt packages |
| v0.11 | Dawn | AI-Native gotgan | BMBX bundle format, Contract-based dependency check, AI package exploration |
| v0.12 | Horizon | WASM dual target | MIR→WASM converter, WASI runtime bindings, Browser runtime, Conditional compilation (@cfg), Dual target build |

### Phase 4: Language Completion (v0.13-v0.18)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.13 | Forge | Language completion | extern fn support, Generic basics, Error handling (? operator + try blocks), @derive attribute macro |
| v0.14 | Foundation | Generic stdlib + package standardization | Package structure standard, Option<T> generics, Result<T,E> generics |
| v0.15 | Generics | Generic type system completion | Where clauses, Generic constraints, Associated types |
| v0.16 | Consolidate | Generic enum/struct type checker | Complete generic instantiation, Type inference improvements |
| v0.17 | Module | Module system + cross-package type reference | Module resolution, Import/export, Type visibility |
| v0.18 | Methods | Option/Result method call syntax | Method chaining, Self type, Trait method resolution |

### Phase 5: Bootstrap & Verification (v0.19-v0.24)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.19 | Complete | MIR Completion (Struct/Enum/Pattern) | Struct MIR lowering, Enum MIR lowering, Pattern matching MIR |
| v0.20 | Extend | Language Extensions | Closures, Traits foundation |
| v0.21 | Bootstrap | Bootstrap Enhancement | Struct/Enum MIR in bootstrap compiler |
| v0.22 | Mirror | Parser Struct/Enum + Type Checker | Bootstrap parser enhancement, Type checker for structs/enums |
| v0.23 | Verify | Self-hosting Stage 1/2 Verification | Stage 1/2 equivalence tests (19 tests) |
| v0.24 | Examples | Bootstrap Examples | 8 algorithm examples in BMB |

### Phase 6: Polish & Performance (v0.25-v0.29)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.25 | Query | AI Query System (RFC-0001) | Natural language code queries, Semantic search |
| v0.26 | Launch | Submodule completion + service launch | Production-ready submodules, Service deployment |
| v0.27 | Registry | gotgan local registry | Local package publishing, Version management |
| v0.28 | Benchmark | C/Rust/BMB benchmark suite | Compute-intensive benchmarks, Contract-optimized benchmarks, Real-world workloads |
| v0.29 | Velocity | C/Rust performance sprint | MIR optimization framework (6 passes), Contract-based optimization, Bootstrap optimization module |

### Bootstrap Statistics (as of v0.29.3)

| Metric | Value |
|--------|-------|
| Rust Codebase | ~21,783 LOC |
| BMB Bootstrap | ~9,700 LOC |
| Coverage | 44% |
| Stage 1/2 Tests | 19 tests passing |
| Bootstrap Tests | 335 tests (119 llvm_ir + 52 lowering + 46 mir + 45 types + 19 selfhost_equiv + 15 utils + 14 pipeline + 9 optimize + 8 selfhost_test + 8 compiler) |

---

## Remaining Phases (v0.30-v1.0.0-rc)

> Detailed task breakdown with gradual difficulty progression

### v0.30 Pure - Self-Hosting Completion

**Goal**: Complete Rust code removal, achieve full self-hosting

**Difficulty**: ⭐⭐⭐⭐⭐ (Highest - Core milestone)

**Duration Estimate**: 8-12 weeks

#### Phase 30.1: Bootstrap Compiler Enhancement

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 30.1.1 | Add generics to bootstrap type checker | P0 | 2 weeks |
| 30.1.2 | Add trait support to bootstrap | P0 | 3 weeks |
| 30.1.3 | Add closure codegen to bootstrap | P1 | 2 weeks |
| 30.1.4 | Implement bootstrap interpreter | P1 | 4 weeks |

**Deliverables**:
- Bootstrap compiler can type-check generic code
- Trait dispatch works in bootstrap
- Closure capture and codegen functional

#### Phase 30.2: Compiler Porting (lang-bmb)

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 30.2.1 | Port main.rs CLI to BMB | P0 | 2 weeks |
| 30.2.2 | Port AST types to BMB | P0 | 2 weeks |
| 30.2.3 | Port full MIR module to BMB | P0 | 4 weeks |
| 30.2.4 | Port codegen module to BMB | P0 | 3 weeks |
| 30.2.5 | Stage 3 verification | P0 | 2 weeks |

**Deliverables**:
- Complete BMB compiler written in BMB
- Stage 3 binary identical to Stage 2

#### Phase 30.3: Package Manager Porting (gotgan)

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 30.3.1 | Port registry client to BMB | P1 | 2 weeks |
| 30.3.2 | Port dependency resolver to BMB | P1 | 2 weeks |
| 30.3.3 | Port build system to BMB | P1 | 3 weeks |
| 30.3.4 | Port CLI and config to BMB | P1 | 1 week |

**Deliverables**:
- gotgan package manager written in BMB
- Full feature parity with Rust version

#### Phase 30.4: Rust Removal

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 30.4.1 | Remove bmb/src/*.rs | P0 | 1 week |
| 30.4.2 | Remove gotgan/src/*.rs | P0 | 1 week |
| 30.4.3 | Remove Cargo.toml files | P0 | 1 day |
| 30.4.4 | Update CI/CD for pure BMB | P0 | 1 week |

**Success Criteria**:
```bash
# Rust file count must be 0
$ git ls-files '*.rs' | wc -l
0

# Cargo.toml must not exist
$ git ls-files 'Cargo.toml' | wc -l
0

# Self-hosting verification
$ bmb build --release
✓ Built bmb compiler (Stage 1)

$ ./target/release/bmb build --release
✓ Built bmb compiler (Stage 2)

$ ./stage2/bmb build --release
✓ Built bmb compiler (Stage 3)

$ diff stage2/bmb stage3/bmb
(no differences - binary identical)
```

---

### v0.31 Docs - Documentation Completion

**Goal**: Comprehensive documentation and website launch

**Difficulty**: ⭐⭐⭐ (Medium)

**Duration Estimate**: 4-6 weeks

#### Phase 31.1: Language Reference

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.1.1 | Complete language syntax reference | P0 | 1 week |
| 31.1.2 | Document type system and generics | P0 | 1 week |
| 31.1.3 | Document contract system (pre/post/invariant) | P0 | 1 week |
| 31.1.4 | Document memory model (ownership/borrowing) | P0 | 1 week |

**Deliverables**:
- Complete language reference document
- Interactive examples for each feature

#### Phase 31.2: Standard Library Documentation

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.2.1 | Generate API documentation for stdlib | P0 | 1 week |
| 31.2.2 | Add usage examples for each module | P1 | 1 week |
| 31.2.3 | Document contract specifications | P1 | 1 week |

**Deliverables**:
- Searchable API documentation
- Code examples for all public functions

#### Phase 31.3: Tutorials and Guides

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.3.1 | Write "Getting Started" tutorial | P0 | 1 week |
| 31.3.2 | Write "By Example" guide | P0 | 2 weeks |
| 31.3.3 | Write "From Rust" migration guide | P1 | 1 week |
| 31.3.4 | Write "Contract Programming" guide | P1 | 1 week |

**Deliverables**:
- Step-by-step getting started guide
- Comprehensive example collection
- Migration guides for Rust/C developers

#### Phase 31.4: Website Launch (bmb.dev)

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.4.1 | Deploy documentation site | P0 | 1 week |
| 31.4.2 | Integrate playground (play.bmb.dev) | P0 | 1 week |
| 31.4.3 | Set up package registry UI (gotgan.bmb.dev) | P1 | 1 week |
| 31.4.4 | Set up benchmark dashboard (bench.bmb.dev) | P1 | 1 week |

**Deliverables**:
- Live documentation website
- Integrated playground
- Package registry with search

---

### v0.32 Ecosystem - Package Ecosystem Growth

**Goal**: 100+ packages and active community

**Difficulty**: ⭐⭐⭐ (Medium - Ongoing effort)

**Duration Estimate**: 6-8 weeks

#### Phase 32.1: Core Package Development

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 32.1.1 | Develop bmb-json (JSON serialization) | P0 | 1 week |
| 32.1.2 | Develop bmb-http (HTTP client) | P0 | 2 weeks |
| 32.1.3 | Develop bmb-regex (Regular expressions) | P0 | 2 weeks |
| 32.1.4 | Develop bmb-crypto (Cryptography) | P1 | 2 weeks |

**Target Package Categories**:

| Category | Count | Key Packages |
|----------|-------|--------------|
| Core/Foundation | 20 | bmb-core, bmb-iter, bmb-hash, bmb-fmt |
| Collections | 15 | bmb-vec, bmb-hashmap, bmb-btreemap |
| IO/Filesystem | 10 | bmb-io, bmb-fs, bmb-path, bmb-tar |
| Networking | 15 | bmb-http, bmb-websocket, bmb-grpc |
| Serialization | 10 | bmb-serde, bmb-json, bmb-toml, bmb-yaml |
| Async | 10 | bmb-async, bmb-future, bmb-channel |
| Crypto/Security | 10 | bmb-crypto, bmb-sha, bmb-aes |
| Database | 10 | bmb-sql, bmb-postgres, bmb-redis |
| **Total** | **100+** | |

#### Phase 32.2: Rust Library Migration

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 32.2.1 | Port serde patterns to BMB | P0 | 2 weeks |
| 32.2.2 | Port regex patterns to BMB | P0 | 2 weeks |
| 32.2.3 | Port clap patterns to BMB | P1 | 1 week |

**Migration Principles**:
- API compatibility maintained (Rust user familiarity)
- Active use of BMB contract system (enhanced type safety)
- Performance parity or improvement goal

#### Phase 32.3: Community Building

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 32.3.1 | Set up contribution guidelines | P0 | 1 week |
| 32.3.2 | Create package submission process | P0 | 1 week |
| 32.3.3 | Establish quality standards | P1 | 1 week |
| 32.3.4 | Set up community forum/Discord | P2 | 1 week |

**Deliverables**:
- CONTRIBUTING.md with clear guidelines
- Package quality checklist
- Community communication channels

---

### v1.0.0-rc Golden - Release Candidate

**Goal**: Final verification and stability promise

**Difficulty**: ⭐⭐⭐⭐ (High - Quality gate)

**Duration Estimate**: 4-6 weeks

#### Phase 1.0.1: Stability Verification

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 1.0.1.1 | Run complete test suite | P0 | 1 week |
| 1.0.1.2 | Verify all benchmarks pass thresholds | P0 | 1 week |
| 1.0.1.3 | Security audit | P0 | 2 weeks |
| 1.0.1.4 | Performance regression testing | P0 | 1 week |

#### Phase 1.0.2: API Freeze

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 1.0.2.1 | Document public API stability guarantees | P0 | 1 week |
| 1.0.2.2 | Mark experimental features | P0 | 1 week |
| 1.0.2.3 | Create deprecation policy | P0 | 1 week |

#### Phase 1.0.3: Release Preparation

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 1.0.3.1 | Write release notes | P0 | 1 week |
| 1.0.3.2 | Update all documentation | P0 | 1 week |
| 1.0.3.3 | Prepare binary distributions | P0 | 1 week |
| 1.0.3.4 | Set up release automation | P1 | 1 week |

**v1.0.0-rc Checklist**:

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Self-Hosting | ✅ 0 lines of Rust, BMB-only build | Pending |
| Performance | ✅ All benchmarks >= C -O3 | Pending |
| Documentation | ✅ Complete language reference + tutorials | Pending |
| Ecosystem | ✅ 100+ packages, active community | Pending |
| Tooling | ✅ fmt, lsp, test, lint, doc complete | Pending |
| Stability | ✅ No breaking changes after 1.0 | Promise |

---

## Ecosystem Repositories

| Repository | Purpose | Current Language | BMB Porting | Service |
|------------|---------|------------------|-------------|---------|
| **lang-bmb** | Main compiler | Rust | v0.30 ★ | - |
| **gotgan** | Package manager | Rust | v0.30 ★ | gotgan.bmb.dev |
| **gotgan-packages** | Additional packages | BMB | v0.26 ✅ | gotgan.bmb.dev |
| **action-bmb** | GitHub Action | YAML/Shell | Maintain | - |
| **bmb-samples** | Example programs | BMB | v0.26 ✅ | - |
| **benchmark-bmb** | Standard benchmarks | C/Rust/BMB | v0.28 ✅ | bench.bmb.dev |
| **playground** | Online playground | TypeScript | Maintain (WASM) | play.bmb.dev |
| **lang-bmb-site** | Official website | Astro/TS | Maintain | bmb.dev |
| **vscode-bmb** | VS Code extension | TypeScript | Maintain | Marketplace |
| **tree-sitter-bmb** | Grammar definition | JavaScript | Maintain | - |

★ = Self-Hosting target (Complete Rust code removal)

### Repository Classification

| Classification | Repositories | Reason |
|----------------|--------------|--------|
| **BMB Porting** | lang-bmb, gotgan | Self-Hosting required |
| **BMB Written** | gotgan-packages, bmb-samples | BMB code examples/libraries |
| **Maintain Current** | playground, lang-bmb-site | Web frontend (WASM integration) |
| **Maintain Current** | vscode-bmb, tree-sitter-bmb | Editor plugins (standard language) |
| **Maintain Current** | action-bmb | GitHub Action (YAML standard) |

---

## Benchmark Goals

> Reference: [Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/), [Is Rust C++-fast? (arXiv)](https://arxiv.org/abs/2209.09127)

| Metric | Target | Description |
|--------|--------|-------------|
| **Runtime Performance** | BMB >= C -O3 | Equal or better on all benchmarks |
| **Contract Optimization** | BMB > C -O3 | Contract-based optimization exceeds C |
| **Memory Usage** | BMB <= Rust | Equal or better than Rust |
| **Compile Speed** | BMB >= Rust | Equal or better than Rust |
| **Binary Size** | BMB <= Rust | Equal or better than Rust |

### Benchmark Categories

#### Category 1: Compute-Intensive (Benchmarks Game Standard)

| Benchmark | Description | Status |
|-----------|-------------|--------|
| fibonacci | Recursive function calls | ✅ Complete |
| n-body | N-body simulation (FP, SIMD) | 📋 Planned |
| mandelbrot | Fractal generation (parallel) | 📋 Planned |
| spectral-norm | Matrix operations | 📋 Planned |
| binary-trees | GC/Memory management | 📋 Planned |

#### Category 2: Contract-Optimized (BMB Unique)

| Benchmark | Contract Benefit | Expected Improvement |
|-----------|------------------|---------------------|
| bounds-check | `pre i < len(arr)` → bounds check elimination | 10-30% |
| null-check | `NonNull<T>` type → null check elimination | 5-15% |
| purity-opt | `pure` function → memoization/inlining | 20-50% |
| aliasing | Ownership-based → LLVM noalias hint | 10-25% |
| invariant-hoist | `invariant` → loop invariant extraction | 15-40% |

---

## Success Criteria

### v1.0.0-rc Release Requirements

```bash
# 1. No Rust code
$ git ls-files '*.rs' | wc -l
0

# 2. Self-hosting verification
$ bmb build --release
✓ Built bmb compiler (Stage 1)

$ ./target/release/bmb build --release
✓ Built bmb compiler (Stage 2)

$ ./stage2/bmb build --release
✓ Built bmb compiler (Stage 3)

$ diff stage2/bmb stage3/bmb
(no differences - binary identical)

# 3. Performance verification
$ bmb bench --all
✓ All benchmarks >= C -O3 threshold

# 4. Test suite
$ bmb test --all
✓ All tests passing (1000+ tests)

# 5. Documentation
$ bmb doc --check
✓ All public items documented
```

### Timeline Summary

```
2025 Q4 ─────────────────────────────────────────────────────
         v0.27 Registry ✅
         v0.28 Benchmark ✅
         v0.29 Velocity ✅

2026 Q1-Q2 ──────────────────────────────────────────────────
         v0.30 Pure (Self-Hosting Completion)
         - Bootstrap generics/traits/closures
         - Compiler/gotgan porting
         - Rust removal

2026 Q3 ─────────────────────────────────────────────────────
         v0.31 Docs (Documentation)
         - Language reference
         - API documentation
         - Tutorials and guides
         - Website launch

2026 Q4 ─────────────────────────────────────────────────────
         v0.32 Ecosystem (Package Ecosystem)
         - 100+ packages
         - Community building
         - Rust library migration

2027 Q1 ─────────────────────────────────────────────────────
         v1.0.0-rc Golden ★
         - Final verification
         - Stability promise
         - Official release
```

---

## Gap Analysis Reference

For detailed analysis of the remaining work, see [GAP_ANALYSIS.md](./GAP_ANALYSIS.md).

**Key Metrics (as of v0.29.3)**:
- Rust code to remove: ~21,783 LOC
- BMB bootstrap code: ~9,700 LOC (44% coverage)
- Gap to close: ~13,200 LOC additional BMB
- Bootstrap tests passing: 335 tests

---

**Last Updated**: 2026-01-04
**Version**: v0.29 → v1.0.0-rc Planning Document
