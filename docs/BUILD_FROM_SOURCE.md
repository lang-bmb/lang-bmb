# Building BMB from Source

**Version**: v0.46 Independence Phase
**Date**: 2026-01-13
**Status**: In Progress

This document describes how to build the BMB compiler from source, including the 3-stage bootstrap verification process.

---

## Overview

BMB has two compiler implementations:

1. **Rust Compiler** (`bmb/`) - Reference implementation written in Rust
2. **Bootstrap Compiler** (`bootstrap/`) - Self-hosted compiler written in BMB (~32K LOC)

The goal of v0.46 Independence is to enable building BMB without Rust, using only a previously built BMB binary (golden binary).

---

## Quick Start

### Option 1: Using Rust Compiler (Current)

```bash
# Prerequisites: Rust 1.75+, LLVM 21+
cargo build --release --features llvm

# Verify
./target/release/bmb --version

# Run a BMB program
./target/release/bmb run examples/hello.bmb

# Compile to native
./target/release/bmb build examples/hello.bmb -o hello
./hello
```

### Option 2: Using Golden Binary (Future - v0.46)

```bash
# Download golden binary (when available)
wget https://github.com/bmb-lang/bmb/releases/download/v0.46/bmb-golden-linux-x64
chmod +x bmb-golden-linux-x64

# Build compiler from source
./bmb-golden-linux-x64 build bootstrap/compiler.bmb -o bmb
./bmb --version
```

---

## Build Requirements

### For Rust Compiler Build

| Component | Version | Notes |
|-----------|---------|-------|
| Rust | 1.75+ | With cargo |
| LLVM | 21+ | For native compilation |
| clang | 21+ | For linking |
| lld | 21+ | Optional, faster linking |

### For BMB-Only Build (v0.46 Target)

| Component | Version | Notes |
|-----------|---------|-------|
| BMB Golden Binary | v0.46+ | Self-contained |
| LLVM | 21+ | For native compilation |
| clang | 21+ | For linking |

---

## Platform-Specific Setup

### Linux (Ubuntu/Debian)

```bash
# Install LLVM 21
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 21

# Set environment
export LLVM_SYS_210_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# Verify
llvm-config --version  # Should show 21.x.x
```

### WSL Ubuntu (Windows)

```bash
# Enter WSL
wsl

# Install LLVM
sudo apt update
sudo apt install -y llvm-21 llvm-21-dev clang-21 lld-21

# Set environment
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# Build from Windows project directory
cd /mnt/d/data/lang-bmb
cargo build --release --features llvm
```

### macOS

```bash
# Install LLVM via Homebrew
brew install llvm@21

# Set environment
export LLVM_SYS_210_PREFIX=$(brew --prefix llvm@21)
export PATH="$(brew --prefix llvm@21)/bin:$PATH"
```

### Windows (Native)

Native Windows builds are not currently supported due to LLVM toolchain limitations. Use WSL Ubuntu instead.

---

## Build Steps

### 1. Clone Repository

```bash
git clone https://github.com/bmb-lang/bmb.git
cd bmb
git submodule update --init --recursive
```

### 2. Build Rust Compiler

```bash
# Without LLVM (interpreter only)
cargo build --release

# With LLVM (native compilation)
cargo build --release --features llvm
```

### 3. Run Tests

```bash
# Rust tests
cargo test

# Bootstrap tests
./target/release/bmb run bootstrap/compiler.bmb
# Expected output: 777...385...888...8...393...999
```

### 4. Generate Golden Binary (v0.46)

```bash
# Compile bootstrap to native binary
./target/release/bmb build bootstrap/compiler.bmb -o bmb-golden

# Verify golden binary
./bmb-golden  # Should output test markers
```

---

## 3-Stage Bootstrap Verification

The 3-stage bootstrap process ensures compiler correctness:

```
Stage 1: Rust BMB → bmb-stage1 (trust Rust compiler)
Stage 2: bmb-stage1 → bmb-stage2 (first self-compile)
Stage 3: bmb-stage2 → bmb-stage3 (verification compile)

Success: diff bmb-stage2 bmb-stage3 == 0
```

### Running Verification

```bash
# Run automated verification script
./scripts/bootstrap_3stage.sh
```

### Manual Verification

```bash
# Stage 1: Rust compiles bootstrap
./target/release/bmb build bootstrap/compiler.bmb -o bmb-stage1

# Stage 2: Stage 1 compiles bootstrap
./bmb-stage1 build bootstrap/compiler.bmb -o bmb-stage2

# Stage 3: Stage 2 compiles bootstrap
./bmb-stage2 build bootstrap/compiler.bmb -o bmb-stage3

# Verify identical binaries
diff bmb-stage2 bmb-stage3  # Must be empty
```

### Why 3-Stage Matters

1. **Stage 1** ensures the Rust implementation is correct
2. **Stage 2** ensures the BMB implementation matches Rust semantics
3. **Stage 3** ensures the BMB compiler generates identical code when compiled by itself

Reference: Ken Thompson, "Reflections on Trusting Trust" (1984)

---

## Build Configuration

### Cargo Features

| Feature | Description |
|---------|-------------|
| `llvm` | Enable LLVM backend for native compilation |
| `wasm` | Enable WASM backend (experimental) |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `LLVM_SYS_210_PREFIX` | LLVM installation prefix |
| `LLVM_SYS_211_PREFIX` | Alternative for LLVM 21.1+ |
| `BMB_DEBUG` | Enable debug output |

---

## Current Status (v0.46)

| Milestone | Status | Notes |
|-----------|--------|-------|
| Stage 1 (Rust → BMB) | ✅ Complete | Golden binary generated |
| Stage 2 (BMB → BMB) | ⏳ Pending | Requires CLI in bootstrap |
| Stage 3 (Verification) | ⏳ Pending | Depends on Stage 2 |
| Cargo.toml Removal | ⏳ Pending | After Stage 3 success |

### Blocking Issues

1. **Bootstrap CLI**: `compiler.bmb` is a test harness, not a full CLI
2. **Runtime Functions**: `arg_count`, `get_arg` not implemented in bootstrap
3. **Build Command**: Bootstrap needs `build` subcommand

### Workaround

Until full 3-stage is ready, use the Rust compiler to generate native binaries:

```bash
# Build any BMB file to native
./target/release/bmb build your_file.bmb -o output

# Run natively
./output
```

---

## Troubleshooting

### LLVM Not Found

```
error: No suitable version of LLVM was found
```

**Solution**: Set `LLVM_SYS_210_PREFIX` or `LLVM_SYS_211_PREFIX` to your LLVM installation.

### Link Errors

```
error: linking with `cc` failed
```

**Solution**: Install clang and ensure it's in PATH:
```bash
sudo apt install clang-21
export CC=clang-21
```

### Bootstrap Test Failure

```
Expected 999, got different output
```

**Solution**: Check recent commits for bootstrap compiler fixes. Run `git pull` and rebuild.

---

## Release Binaries

Pre-built binaries will be available starting from v0.46:

| Platform | Binary | Status |
|----------|--------|--------|
| Linux x64 | `bmb-golden-linux-x64` | Planned |
| macOS x64 | `bmb-golden-macos-x64` | Planned |
| macOS ARM | `bmb-golden-macos-arm64` | Planned |
| Windows x64 | `bmb-golden-windows-x64.exe` | Planned (via WSL cross-compile) |

---

## Contributing

To contribute to BMB compiler development:

1. Fork the repository
2. Create a feature branch
3. Make changes and test
4. Run `cargo test` and `./scripts/bootstrap_3stage.sh`
5. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

---

## References

- [Bootstrapping (compilers) - Wikipedia](https://en.wikipedia.org/wiki/Bootstrapping_(compilers))
- [Reproducible Builds](https://reproducible-builds.org/)
- [Ken Thompson - Reflections on Trusting Trust](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ResearchStudy.pdf)
