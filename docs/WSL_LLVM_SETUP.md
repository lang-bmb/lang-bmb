# WSL LLVM Development Setup

BMB native compilation requires LLVM 21+. This guide covers WSL Ubuntu setup.

## Prerequisites

- Windows 10/11 with WSL2
- Ubuntu 24.04 in WSL

## Quick Setup

```bash
# 1. Install LLVM 21
wget -qO- https://apt.llvm.org/llvm.sh | sudo bash -s -- 21 all

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# 3. Install dependencies
sudo apt-get install -y zlib1g-dev libzstd-dev

# 4. Build BMB with LLVM
cd /mnt/d/data/lang-bmb  # or your project path
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
cargo build --release --features llvm

# 5. Build BMB runtime library
cd bmb/runtime
clang -c bmb_runtime.c -o bmb_runtime.o
ar rcs libbmb_runtime.a bmb_runtime.o
cd ../..
```

## Usage

### Quick Build (Integrated)

```bash
# Set runtime path
export BMB_RUNTIME_PATH=/path/to/lang-bmb/bmb/runtime/libbmb_runtime.a

# Build with optimizations
./target/release/bmb build example.bmb --release -o example

# Build with maximum optimizations (-O3)
./target/release/bmb build example.bmb --aggressive -o example

# Run
./example
```

### Emit LLVM IR

```bash
./target/release/bmb build example.bmb --emit-ir -o example.ll
```

### Manual Compilation (for maximum performance)

For best performance, use clang -O3 directly on the generated IR:

```bash
# Generate LLVM IR
./target/release/bmb build example.bmb --emit-ir -o example.ll

# Compile with clang -O3
clang -O3 example.ll /path/to/libbmb_runtime.a -o example -lm

# Run
./example
```

## Build Flags

| Flag | Optimization | Description |
|------|--------------|-------------|
| (none) | -O0 | Debug build, no optimization |
| `--release` | -O2 | Standard optimizations |
| `--aggressive` | -O3 | Maximum optimizations |

## Environment Variables

| Variable | Value | Purpose |
|----------|-------|---------|
| `LLVM_SYS_211_PREFIX` | `/usr/lib/llvm-21` | llvm-sys crate configuration |
| `BMB_RUNTIME_PATH` | Path to `libbmb_runtime.a` | BMB runtime library |

## Verification

```bash
# Check LLVM version
llvm-config-21 --version  # Should show 21.x.x

# Check BMB build
./target/release/bmb --version

# Test compilation
export BMB_RUNTIME_PATH=$(pwd)/bmb/runtime/libbmb_runtime.a
./target/release/bmb build examples/fibonacci.bmb --release -o /tmp/fib
time /tmp/fib  # Exit code = fib(35) % 256 = 201
```

## Benchmark Results (v0.99.0)

Performance comparison:

| Benchmark | C (-O3) | BMB (IR + clang -O3) | BMB (--aggressive) |
|-----------|---------|----------------------|--------------------|
| fib(40) | 0.172s | 0.174s (100%) | 0.287s (1.7x) |
| fib(45) | 1.796s | 1.792s (100%) | ~3.1s (1.7x) |

**P0 Performance Goal: Achieved** - BMB matches C -O3 performance when using
`bmb build --emit-ir` + `clang -O3` workflow.

Note: inkwell's built-in optimizer (`--aggressive`) is about 1.7x slower than
clang -O3. For performance-critical applications, use the manual compilation workflow.

## Troubleshooting

### "Cannot find BMB runtime library"

Set the runtime path:
```bash
export BMB_RUNTIME_PATH=/path/to/lang-bmb/bmb/runtime/libbmb_runtime.a
```

Or build the runtime:
```bash
cd bmb/runtime
clang -c bmb_runtime.c -o bmb_runtime.o
ar rcs libbmb_runtime.a bmb_runtime.o
```

### Linking errors (libz, libzstd)

```bash
sudo apt-get install -y zlib1g-dev libzstd-dev
```

### LLVM version mismatch

Ensure `LLVM_SYS_211_PREFIX` matches your LLVM version:
```bash
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
```

### Executable says "required file not found"

Create the dynamic linker symlink:
```bash
sudo ln -sf /lib64/ld-linux-x86-64.so.2 /lib/ld64.so.1
```
