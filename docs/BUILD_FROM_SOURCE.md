# Building BMB from Source

**Status**: 3-Stage Bootstrap Verified (Fixed Point Achieved)

This document describes how to build the BMB compiler from source.

---

## Quick Start

```bash
# Prerequisites: Rust 1.75+, LLVM 21+
git clone https://github.com/lang-bmb/lang-bmb.git
cd lang-bmb

# Build (Windows)
cargo build --release --features llvm --target x86_64-pc-windows-gnu

# Build (Linux/macOS)
cargo build --release --features llvm

# Verify
./target/release/bmb --version
```

---

## Platform Setup

### Windows (MSYS2/MinGW)

```bash
# Install LLVM via MSYS2
pacman -S mingw-w64-ucrt-x86_64-llvm mingw-w64-ucrt-x86_64-clang

# Build with MinGW target
cargo build --release --features llvm --target x86_64-pc-windows-gnu
```

### Linux (Ubuntu/Debian)

```bash
# Install LLVM 21
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 21

# Build
cargo build --release --features llvm
```

### macOS

```bash
# Install LLVM via Homebrew
brew install llvm@21
export LLVM_SYS_211_PREFIX=$(brew --prefix llvm@21)

# Build
cargo build --release --features llvm
```

---

## 3-Stage Bootstrap Verification

### Overview

```
Stage 0: Rust → Stage 1 Binary
Stage 1: Stage 1 → Stage 2 LLVM IR
Stage 2: Stage 2 → Stage 3 LLVM IR

Success: Stage 2 IR == Stage 3 IR (Fixed Point)
```

### Running Verification

```bash
# Quick check (Stage 1 only, ~13s)
./scripts/bootstrap.sh --stage1-only

# Full 3-stage Fixed Point verification (~70s)
./scripts/bootstrap.sh
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
│   └── bootstrap.sh         # 3-stage verification
└── target/                  # Build output
    ├── release/             # Cargo release build
    └── bootstrap/           # Bootstrap stage outputs
```

---

## Troubleshooting

### LLVM Not Found

```
error: No suitable version of LLVM was found
```

**Solution**: Set `LLVM_SYS_211_PREFIX`:
```bash
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21          # Linux
export LLVM_SYS_211_PREFIX=$(brew --prefix llvm@21)  # macOS
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
