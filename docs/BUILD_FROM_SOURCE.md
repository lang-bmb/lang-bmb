# Building BMB from Source

**Version**: v0.46 Independence Phase
**Date**: 2026-01-20
**Status**: 3-Stage Bootstrap Verified (v0.50.56)

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

### Option 2: Using Golden Binary (v0.46+)

```bash
# Download golden binary
wget https://github.com/bmb-lang/bmb/releases/download/v0.46/bmb-golden-linux-x64
chmod +x bmb-golden-linux-x64

# Also download the runtime library
wget https://github.com/bmb-lang/bmb/releases/download/v0.46/libruntime_linux.a

# Build compiler from source
./bmb-golden-linux-x64 build bootstrap/bmb_unified_cli.bmb -o bmb
./bmb --version
```

**Note**: The golden binary generates LLVM IR. You need LLVM 21+ toolchain (llc, clang) for linking.

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
./target/release/bmb build bootstrap/bmb_unified_cli.bmb -o bmbc_stage1

# Stage 2: Stage 1 compiles bootstrap (generates IR)
./bmbc_stage1 build bootstrap/bmb_unified_cli.bmb > stage2.ll

# Compile Stage 2 to binary
llc-21 -filetype=obj -O2 stage2.ll -o stage2.o
clang-21 -o bmbc_stage2 stage2.o runtime/libruntime_linux.a -lm

# Stage 3: Stage 2 compiles bootstrap
./bmbc_stage2 build bootstrap/bmb_unified_cli.bmb > stage3.ll

# Verify identical IR output
diff stage2.ll stage3.ll  # Must be empty (0 differences)
```

**Note**: We compare IR output (`.ll` files) rather than binaries because LLVM
compilation is deterministic but binary layout may vary with toolchain versions.

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

## Current Status (v0.46 - v0.50.56)

| Milestone | Status | Notes |
|-----------|--------|-------|
| Stage 1 (Rust → BMB) | ✅ Complete | 160KB golden binary generated |
| Stage 2 (BMB → BMB) | ✅ Complete | 1,068,850 bytes IR output |
| Stage 3 (Verification) | ✅ Complete | Stage 2 = Stage 3 identical |
| Cargo.toml Removal | 🔄 In Progress | Documentation phase |

### 3-Stage Bootstrap Verified

The BMB compiler successfully compiles itself in a 3-stage bootstrap:

```
Stage 1 (Rust BMB):     bootstrap/bmb_unified_cli.bmb → bmbc_stage1 (160KB)
Stage 2 (Stage 1):      bootstrap/bmb_unified_cli.bmb → stage2.ll (1,068,850 bytes)
Stage 3 (Stage 2):      bootstrap/bmb_unified_cli.bmb → stage3.ll (1,068,850 bytes)

✅ sha256(stage2.ll) == sha256(stage3.ll)
```

### Key Files for BMB-Only Build

| File | Description |
|------|-------------|
| `bootstrap/bmb_unified_cli.bmb` | Full bootstrap compiler (143KB, 2895 lines) |
| `runtime/libruntime_linux.a` | Linux runtime library |
| `runtime/runtime.c` | Runtime source (for other platforms) |

### Building from Golden Binary

```bash
# 1. Generate LLVM IR
./bmb-golden build your_file.bmb --emit-ir -o output.ll

# 2. Compile to object file
llc-21 -filetype=obj -O2 output.ll -o output.o

# 3. Link with runtime
clang-21 -o output output.o libruntime_linux.a -lm

# 4. Run
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
