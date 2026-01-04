# BMB Ecosystem

This document describes the BMB ecosystem tools and submodules.

## Submodule Overview

| Repository | Purpose | Technology |
|------------|---------|------------|
| gotgan | Package manager | Rust |
| tree-sitter-bmb | Syntax highlighting | JavaScript/C |
| vscode-bmb | VS Code extension | TypeScript |
| playground | Online playground | React/WASM |
| lang-bmb-site | Official website | Astro |
| bmb-samples | Examples and tutorials | BMB |
| benchmark-bmb | Performance benchmarks | Rust/BMB/C |
| action-bmb | GitHub Actions | Shell/YAML |

## gotgan (Package Manager)

Korean: storehouse

### Features

- Package management: new, build, run, test, publish
- Rust fallback: Use Cargo/crates.io dependencies
- Migration tools: Gradual Rust to BMB conversion

### Usage

```bash
gotgan new hello            # Create new project
gotgan build                # Build project
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

## Version Roadmap

| Version | gotgan | tree-sitter | vscode | playground | site |
|---------|--------|-------------|--------|------------|------|
| v0.8 | v0.1 | - | - | - | - |
| v0.9 | v0.2 | v0.1 | v0.1 | v0.1 | v0.1 |
| v0.10 | v0.3 | v0.2 | v0.2 | v0.2 | v0.2 |
| v1.0 | v1.0 | v1.0 | v1.0 | v1.0 | v1.0 |
