# BMB Ecosystem

This document describes the BMB ecosystem tools and submodules.

## Submodule Overview

| Repository | Purpose | Technology |
|------------|---------|------------|
| gotgan | Package manager | Rust |
| gotgan-packages | Package registry | BMB |
| bmb-mcp (Chatter) | MCP server for AI | Rust/TypeScript |
| bmb-query | Contract query interface | Rust |
| bmb-test | Testing framework | Rust/BMB |
| bmb-labs | Experimental tools | BMB |
| tree-sitter-bmb | Syntax highlighting | JavaScript/C |
| vscode-bmb | VS Code extension | TypeScript |
| playground | Online playground | React/WASM |
| lang-bmb-site | Official website | Astro |
| bmb-samples | Examples and tutorials | BMB |
| benchmark-bmb | Performance benchmarks | Rust/BMB/C |
| action-bmb | GitHub Actions | Shell/YAML |

## bmb-mcp (Chatter)

MCP (Model Context Protocol) server that enables AI models to generate high-quality BMB code.

### Problem

BMB doesn't exist in LLM training data. Without context injection, AI models:
- Confuse `T?` with `Option<T>`
- Use `&` for bitwise AND (should be `band`)
- Forget explicit `return` in block bodies
- Generate incorrect or missing contracts

### Solution

Chatter provides **selective, on-demand access** to BMB specifications at runtime instead of stuffing the entire spec into every prompt.

### Features

- Language specification lookup
- Compilation feedback
- Contract verification
- Example code retrieval

## bmb-query

Natural language query interface for BMB codebases — ask questions, get answers grounded in contracts.

### Usage

```bash
bmb query "what does the auth module guarantee?"
bmb query "can this function ever return null?"
bmb query "what preconditions does payment.charge require?"
bmb query "when does binary_search fail?"
```

### Philosophy

You don't need to read AI-generated code. You need to understand what it **guarantees**.

## bmb-test

Testing framework for BMB with property-based testing and contract-aware generation.

### Philosophy

> **Test is the new code. Code is just the implementation.**

In AI-first development, humans don't write implementations—they write expectations. Contracts are the specification; tests validate them.

### Features

- **Property-based testing**: Generate thousands of inputs automatically
- **Contract-aware generation**: Inputs respect preconditions (no wasted test cases)
- **Fuzz testing**: Find edge cases through randomization
- **CDO validation**: Verify that CDO transformations preserve semantics

### Example

```bmb
#[property]
fn sort_is_idempotent(arr: [i32; 100]) {
    assert(sort(sort(arr)) == sort(arr));
}

#[property]
fn sort_preserves_elements(arr: [i32; 100]) {
    let sorted = sort(&arr);
    assert(is_permutation(sorted, arr));
}
```

### CDO Integration

bmb-test validates that Contract-Driven Optimization preserves program semantics:

| Test Type | Purpose |
|-----------|---------|
| Equivalence tests | CDO-optimized code = original code |
| Specialization tests | Specialized functions maintain contracts |
| Extraction tests | Minimal extraction preserves behavior |

## bmb-labs

Experimental tools and verified implementations.

## gotgan-packages

Community package registry for BMB packages.

### Structure

```
gotgan-packages/
├── packages/
│   ├── json/
│   ├── http/
│   └── ...
└── MODULE_ROADMAP.md
```

## gotgan (Package Manager)

Korean: storehouse (곳간)

### Features

- Package management: new, build, run, test, publish
- Rust fallback: Use Cargo/crates.io dependencies
- Migration tools: Gradual Rust to BMB conversion
- **CDO-aware resolution** (v0.65+): Contract-compatible dependency extraction

### Usage

```bash
gotgan new hello            # Create new project
gotgan build                # Build project
gotgan build --cdo          # Build with CDO optimization
gotgan run                  # Run project
gotgan test                 # Run tests
gotgan verify               # Verify contracts
gotgan add json             # Add dependency
gotgan publish              # Publish package
```

### Manifest (gotgan.toml)

```toml
[package]
name = "hello"
version = "0.1.0"
edition = "2025"

[dependencies]
json = "0.1"

[rust-fallback]
enabled = true
crates = ["serde", "tokio"]

[cdo]
enabled = true              # Enable CDO (v0.65+)
extraction = "minimal"      # "minimal" | "full"
```

### CDO Integration (v0.65+)

gotgan supports Contract-Driven Optimization for dependency resolution:

```bmb
// Your code uses json::parse with constraints
fn my_parse(s: &str) -> Value
  pre s.len() < 1000
  pre s.is_ascii()
= json::parse(s).unwrap();

// gotgan extracts only:
// - ASCII parsing paths
// - Small-string optimized paths
// - No streaming support
// Result: 60-80% less dependency code
```

## tree-sitter-bmb

Tree-sitter grammar for editor integration.

### Features

- Syntax highlighting
- Code folding
- Indentation rules
- Local variable scoping

### Files

```
tree-sitter-bmb/
├── grammar.js            # Grammar definition
├── src/                  # Generated parser
├── queries/
│   ├── highlights.scm    # Syntax highlighting
│   ├── folds.scm         # Code folding
│   ├── indents.scm       # Indentation
│   └── locals.scm        # Local variables
└── bindings/
    ├── node/             # Node.js bindings
    └── rust/             # Rust bindings
```

## vscode-bmb

VS Code extension for BMB language support.

### Features

- Syntax highlighting (TextMate grammar)
- Code snippets
- LSP integration (diagnostics, hover, go-to-definition)
- Contract verification status

## playground

Online playground for trying BMB without installation.

### Features

- Monaco Editor with BMB highlighting
- WASM-compiled BMB interpreter
- Real-time type checking
- Contract verification
- URL sharing

### Technology Stack

- Frontend: React + TypeScript
- Editor: Monaco Editor
- Compiler: BMB compiled to WASM
- Hosting: Cloudflare Pages

## lang-bmb-site

Official BMB website with documentation, downloads, and blog.

### Technology Stack

- Framework: Astro (static site generator)
- Styling: Tailwind CSS
- Code highlighting: Shiki with BMB grammar
- Search: Pagefind
- Hosting: GitHub Pages / Cloudflare Pages

### Pages

| Page | Content |
|------|---------|
| / | Language introduction, key features |
| /docs | Reference, tutorials |
| /download | Installation guide, binaries |
| /changes | Version changelog |
| /blog | Development updates |

## bmb-samples

Example programs and tutorials.

### Categories

| Directory | Content |
|-----------|---------|
| basics/ | Basic syntax (hello, variables, functions) |
| contracts/ | Contract verification examples |
| data_structures/ | Structs, enums, arrays |
| algorithms/ | Sorting, searching, math |
| projects/ | Complete projects (calculator, todo-cli) |
| tutorials/ | Step-by-step guides |

## benchmark-bmb

Performance benchmark suite comparing C, Rust, and BMB.

### Goals

- BMB >= C -O3 (all cases)
- BMB > C -O3 (contract-enabled optimizations)

### Benchmark Categories

| Category | Benchmarks |
|----------|------------|
| Compute | n-body, mandelbrot, fannkuch, spectral-norm |
| Memory | binary-trees, reverse-complement |
| Real-world | json-parse, regex-redux, http-throughput |
| Contract | bounds-check-elim, null-check-elim, purity-opt |

## action-bmb

GitHub Actions for BMB projects.

### Usage

```yaml
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: lang-bmb/action-bmb@v1
        with:
          command: build

  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: lang-bmb/action-bmb@v1
        with:
          command: verify
```

### Inputs

| Input | Description | Default |
|-------|-------------|---------|
| command | Command to run (build, test, verify, check) | build |
| bmb-version | BMB compiler version | latest |

## Submodule Management

### Initial Setup

```bash
# Clone with all submodules
git clone --recursive https://github.com/lang-bmb/lang-bmb.git

# Or initialize after clone
git submodule update --init --recursive
```

### Updating Submodules

```bash
# Update all submodules to latest
git submodule update --remote --merge

# Update specific submodule
cd ecosystem/gotgan
git pull origin main
cd ../..
git add ecosystem/gotgan
git commit -m "Update gotgan submodule"
```

## Contract-Driven Optimization (CDO)

CDO is a cross-cutting feature that affects multiple ecosystem components:

> **RFC**: [RFC-0008-contract-driven-optimization](rfcs/RFC-0008-contract-driven-optimization.md)

### CDO in Ecosystem

| Component | CDO Role |
|-----------|----------|
| **BMB Compiler** | Core CDO passes (Semantic DCE, Specialization) |
| **gotgan** | Contract-aware dependency resolution |
| **bmb-mcp** | AI understands CDO for better code generation |
| **bmb-test** | Validates CDO preserves semantics |
| **vscode-bmb** | CDO-informed autocomplete and hints |

### CDO Benefits

```
┌──────────────────────────────────────────────────────────────┐
│                    Contract-Driven Optimization              │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│   Contracts ─────────▶ Optimization Opportunities            │
│                                                              │
│   • pre x > 0        → Eliminate x <= 0 branches            │
│   • pre len < 1000   → Use small-buffer optimized code      │
│   • pure fn          → Enable CSE, memoization              │
│   • post is_sorted   → Skip sorting at call sites           │
│                                                              │
│   Result: 50-80% smaller binaries, faster execution          │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Version Roadmap

| Version | gotgan | tree-sitter | vscode | playground | site |
|---------|--------|-------------|--------|------------|------|
| v0.8 | v0.1 | - | - | - | - |
| v0.9 | v0.2 | v0.1 | v0.1 | v0.1 | v0.1 |
| v0.10 | v0.3 | v0.2 | v0.2 | v0.2 | v0.2 |
| v1.0 | v1.0 | v1.0 | v1.0 | v1.0 | v1.0 |
