# WSL Development Guide

> BMB 네이티브 컴파일 및 검증을 위한 WSL Ubuntu 환경 가이드

**Requirements**: Windows 10/11 with WSL2, Ubuntu 22.04+

---

## Quick Start

```bash
# 1. Enter WSL
wsl

# 2. Navigate to project
cd /mnt/d/data/lang-bmb

# 3. Run full verification
./scripts/verify_all.sh
```

---

## 1. Initial Setup (One-time)

### WSL Installation

```powershell
# Run in PowerShell as Administrator
wsl --install -d Ubuntu-22.04
```

Restart your computer, then set up Ubuntu username/password.

### LLVM Installation

```bash
# Enter WSL
wsl

# Install LLVM 21
wget -qO- https://apt.llvm.org/llvm.sh | sudo bash -s -- 21 all

# Install additional tools
sudo apt install -y clang-21 lld-21 llvm-21-dev zlib1g-dev libzstd-dev

# Add to ~/.bashrc for persistence
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# Verify
llvm-config --version  # Should show 21.x.x
```

### Rust Installation

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

---

## 2. Build BMB with LLVM

```bash
cd /mnt/d/data/lang-bmb

# Build with LLVM feature
cargo build --release --features llvm

# Build runtime library
cd bmb/runtime
clang -c bmb_runtime.c -o bmb_runtime.o -O3
ar rcs libbmb_runtime.a bmb_runtime.o
cd ../..

# Verify
./target/release/bmb build examples/hello.bmb -o hello
./hello  # Should print "Hello, World!"
```

---

## 3. Native Compilation

### Environment Variables

| Variable | Value | Purpose |
|----------|-------|---------|
| `LLVM_SYS_211_PREFIX` | `/usr/lib/llvm-21` | llvm-sys crate configuration |
| `BMB_RUNTIME_PATH` | Path to `libbmb_runtime.a` | BMB runtime library |

### Build Commands

```bash
# Set runtime path
export BMB_RUNTIME_PATH=$(pwd)/bmb/runtime/libbmb_runtime.a

# Standard build (-O2)
./target/release/bmb build example.bmb --release -o example

# Maximum optimization (-O3)
./target/release/bmb build example.bmb --aggressive -o example

# Emit LLVM IR only
./target/release/bmb build example.bmb --emit-ir -o example.ll
```

### Manual Compilation (Best Performance)

For maximum performance, use clang -O3 directly:

```bash
# Generate LLVM IR
./target/release/bmb build example.bmb --emit-ir -o example.ll

# Compile with clang -O3
clang -O3 example.ll bmb/runtime/libbmb_runtime.a -o example -lm -no-pie

# Run
./example
```

---

## 4. Verification Procedures

### 3-Stage Bootstrap

Verifies the BMB compiler can compile itself with identical output.

```bash
./scripts/bootstrap_3stage.sh
```

**Expected Output**:
```
======================================
BMB 3-Stage Bootstrap Verification
======================================

[1/4] Stage 1: Rust BMB -> Stage 1 Binary
      Stage 1 OK (tests passed: 999 marker)

[2/4] Stage 2: Stage 1 -> LLVM IR
      Stage 2 OK (LLVM IR generated: ~2500 lines)

[3/4] Stage 3: Stage 2 Binary -> Stage 3 LLVM IR
      Stage 3 OK (LLVM IR generated: ~2500 lines)

[4/4] Verification: Comparing Stage 2 and Stage 3
      SUCCESS: Stage 2 == Stage 3
```

### Benchmark Verification

```bash
cd /mnt/d/data/lang-bmb/ecosystem/benchmark-bmb

# Build runner
cd runner && cargo build --release && cd ..

# Run all benchmarks
./runner/target/release/benchmark-bmb run all -i 5 -w 2

# Verify gates
./runner/target/release/benchmark-bmb gate 3.1 -v
```

### Gate Criteria

| Gate | Description | Target |
|------|-------------|--------|
| #3.1 | Compute benchmarks | ≤1.10x C |
| #3.2 | Benchmarks Game | ≤1.05x C |
| #3.3 | Faster than C | 3+ benchmarks |
| #4.1 | Self-compile | <60s (✅ 0.56s) |

---

## 5. Performance Results

Current BMB vs C (native compilation):

| Benchmark | C | Rust | BMB | vs C |
|-----------|---|------|-----|------|
| fibonacci(45) | 1.65s | 1.66s | 1.63s | **0.99x** |
| fibonacci(40) | 177ms | 180ms | 150ms | **0.85x** |
| mandelbrot | 42ms | 42ms | 39ms | **0.93x** |
| spectral_norm | 44ms | 44ms | 39ms | **0.89x** |

---

## 6. Troubleshooting

### LLVM not found

```bash
export PATH="/usr/lib/llvm-21/bin:$PATH"
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
```

### Linking errors (libz, libzstd)

```bash
sudo apt-get install -y zlib1g-dev libzstd-dev
```

### "Cannot find BMB runtime library"

```bash
export BMB_RUNTIME_PATH=/path/to/lang-bmb/bmb/runtime/libbmb_runtime.a

# Or rebuild runtime:
cd bmb/runtime
clang -c bmb_runtime.c -o bmb_runtime.o -O3
ar rcs libbmb_runtime.a bmb_runtime.o
```

### Executable "required file not found"

```bash
sudo ln -sf /lib64/ld-linux-x86-64.so.2 /lib/ld64.so.1
```

### Binary format error

Running Windows binary in WSL. Rebuild in WSL:
```bash
cd /mnt/d/data/lang-bmb && cargo build --release --features llvm
```

### Windows line endings (CRLF)

```bash
find scripts -name "*.sh" -exec sed -i 's/\r$//' {} \;
```

### Permission denied on scripts

```bash
chmod +x scripts/*.sh
```

### WSL catastrophic failure

```powershell
wsl --shutdown
wsl
```

---

## 7. Files Generated

| File | Description |
|------|-------------|
| `bmb-stage1` | Stage 1 native binary |
| `bmb-stage2` | Stage 2 native binary |
| `bmb-stage2.ll` | Stage 2 LLVM IR |
| `bmb-stage3.ll` | Stage 3 LLVM IR |

Clean up:
```bash
rm -f bmb-stage* *.ll *.o hello fib fib_c
```
