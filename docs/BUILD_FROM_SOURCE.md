# Building BMB from Source

**Version**: v0.60.251
**Status**: 3-Stage Bootstrap Verified (Fixed Point Achieved)

This document describes how to build the BMB compiler from source.

---

## Quick Start

### Option 1: Using Golden Binary (Recommended - No Rust Required)

```bash
# Clone repository
git clone https://github.com/lang-bmb/lang-bmb.git
cd lang-bmb

# Run golden bootstrap
./scripts/golden-bootstrap.sh

# Verify (optional)
./scripts/golden-bootstrap.sh --verify
```

**Output**: `target/golden-bootstrap/bmb-stage1.exe` (Windows) or `bmb-stage1` (Linux/macOS)

### Option 2: Using Rust Compiler

```bash
# Prerequisites: Rust 1.75+, LLVM 21+
cargo build --release --features llvm --target x86_64-pc-windows-gnu

# Verify
./target/x86_64-pc-windows-gnu/release/bmb --version
```

---

## Build Methods Comparison

| Method | Requirements | Build Time | Use Case |
|--------|--------------|------------|----------|
| **Golden Binary** | LLVM only | ~8s | Production, CI/CD |
| **Rust Compiler** | Rust + LLVM | ~30s | Development |

---

## Option 1: Golden Binary Bootstrap (BMB-Only)

### Requirements

| Component | Version | Notes |
|-----------|---------|-------|
| LLVM | 21+ | `opt`, `clang` commands |
| Golden Binary | v0.60.251 | Included in `golden/` |

### Available Golden Binaries

| Platform | Path | Status |
|----------|------|--------|
| Windows x64 | `golden/windows-x64/bmb.exe` | ✅ Available |
| Linux x86_64 | `golden/linux-x86_64/bmb` | 📋 Planned |
| Linux aarch64 | `golden/linux-aarch64/bmb` | 📋 Planned |
| macOS Universal | `golden/darwin-universal/bmb` | 📋 Planned |

### Build Steps

```bash
# 1. Clone repository
git clone https://github.com/lang-bmb/lang-bmb.git
cd lang-bmb

# 2. Run bootstrap script
./scripts/golden-bootstrap.sh

# 3. (Optional) Verify with 3-stage check
./scripts/golden-bootstrap.sh --verify

# 4. Use the bootstrapped compiler
export PATH="$(pwd)/target/golden-bootstrap:$PATH"
bmb-stage1 --help
```

### Manual Build (Without Script)

```bash
# Step 1: Generate LLVM IR using golden binary
./golden/windows-x64/bmb.exe bootstrap/compiler.bmb stage1.ll

# Step 2: Optimize with LLVM opt
opt -O3 stage1.ll -S -o stage1_opt.ll

# Step 3: Compile and link
clang -O3 stage1_opt.ll bmb/runtime/bmb_runtime.c -o bmb-stage1.exe -lm
```

---

## Option 2: Rust Compiler Build

### Requirements

| Component | Version | Notes |
|-----------|---------|-------|
| Rust | 1.75+ | With cargo |
| LLVM | 21+ | For native compilation |

### Platform Setup

#### Windows (MSYS2/MinGW)

```bash
# Install LLVM via MSYS2
pacman -S mingw-w64-ucrt-x86_64-llvm mingw-w64-ucrt-x86_64-clang

# Build with MinGW target
cargo build --release --features llvm --target x86_64-pc-windows-gnu
```

#### Linux (Ubuntu/Debian)

```bash
# Install LLVM 21
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 21

# Build
cargo build --release --features llvm
```

#### macOS

```bash
# Install LLVM via Homebrew
brew install llvm@21
export LLVM_SYS_211_PREFIX=$(brew --prefix llvm@21)

# Build
cargo build --release --features llvm
```

### Build Commands

```bash
# Clone
git clone https://github.com/lang-bmb/lang-bmb.git
cd lang-bmb
git submodule update --init --recursive

# Build (choose one)
cargo build --release                           # Interpreter only
cargo build --release --features llvm           # With native compilation

# Test
cargo test --release
```

---

## 3-Stage Bootstrap Verification

### Overview

```
Stage 0: Rust/Golden → Stage 1 Binary
Stage 1: Stage 1    → Stage 2 LLVM IR
Stage 2: Stage 2    → Stage 3 LLVM IR

✅ Success: Stage 2 IR == Stage 3 IR (Fixed Point)
```

### Running Verification

```bash
# Using Rust compiler
./scripts/bootstrap.sh --verbose

# Using Golden binary
./scripts/golden-bootstrap.sh --verify
```

### Current Status (v0.60.251)

```
Stage 1 (Rust/Golden → BMB₁):  ✅ (1.5s)
Stage 2 (BMB₁ → LLVM IR):      ✅ (1.2s, 29,925 lines)
Stage 3 (BMB₂ → LLVM IR):      ✅ (3.5s, 29,925 lines)
Fixed Point (S2 == S3):        ✅ VERIFIED
```

---

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `BMB_RUNTIME_PATH` | Path to BMB runtime | `d:/data/lang-bmb/bmb/runtime` |
| `LLVM_SYS_211_PREFIX` | LLVM installation prefix | `/usr/lib/llvm-21` |

---

## Directory Structure

```
lang-bmb/
├── golden/                  # Golden binaries (BMB-only bootstrap)
│   ├── windows-x64/bmb.exe
│   ├── VERSION
│   └── README.md
├── bootstrap/               # Self-hosted compiler source (~32K LOC)
│   ├── compiler.bmb         # Main entry point
│   ├── lexer.bmb
│   ├── parser.bmb
│   ├── types.bmb
│   ├── lowering.bmb
│   ├── mir.bmb
│   ├── optimize.bmb
│   └── llvm_ir.bmb
├── bmb/                     # Rust compiler and runtime
│   ├── src/                 # Rust source
│   └── runtime/             # C runtime
│       └── bmb_runtime.c
├── scripts/
│   ├── bootstrap.sh         # 3-stage with Rust
│   └── golden-bootstrap.sh  # 3-stage with Golden
└── target/                  # Build output
    ├── bootstrap/           # Rust bootstrap output
    └── golden-bootstrap/    # Golden bootstrap output
```

---

## Troubleshooting

### LLVM Not Found

```
error: No suitable version of LLVM was found
```

**Solution**: Set `LLVM_SYS_211_PREFIX`:
```bash
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21  # Linux
export LLVM_SYS_211_PREFIX=$(brew --prefix llvm@21)  # macOS
```

### Golden Binary Not Found

```
Error: Golden binary not found at golden/linux-x86_64/bmb
```

**Solution**: Build golden binary for your platform:
```bash
# Build with Rust first
cargo build --release --features llvm
./scripts/bootstrap.sh

# Copy Stage 2 as golden binary
mkdir -p golden/linux-x86_64
cp target/bootstrap/bmb-stage2 golden/linux-x86_64/bmb
```

### opt Command Not Found

```
Error: LLVM opt not found
```

**Solution**: Install LLVM and add to PATH:
```bash
# Ubuntu
sudo apt install llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# MSYS2
pacman -S mingw-w64-ucrt-x86_64-llvm
```

---

## References

- [BMB Specification](SPECIFICATION.md)
- [Bootstrap + Benchmark Cycle](BOOTSTRAP_BENCHMARK.md)
- [Ken Thompson - Reflections on Trusting Trust (1984)](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ResearchStudy.pdf)
