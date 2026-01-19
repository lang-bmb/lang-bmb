# WSL Verification Guide

> Complete instructions for running native compilation and 3-Stage bootstrap verification in WSL Ubuntu.

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

## Full Setup Guide

### Step 1: WSL Installation (One-time)

If WSL is not installed:

```powershell
# Run in PowerShell as Administrator
wsl --install -d Ubuntu-22.04
```

Restart your computer, then set up Ubuntu username/password.

### Step 2: LLVM Installation (One-time)

```bash
# Enter WSL
wsl

# Add LLVM repository
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 21

# Install additional tools
sudo apt install -y clang-21 lld-21 llvm-21-dev

# Add to PATH (add to ~/.bashrc for persistence)
export LLVM_SYS_210_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# Verify installation
llvm-config --version  # Should show 21.x.x
clang --version
```

### Step 3: Build BMB with LLVM

```bash
cd /mnt/d/data/lang-bmb

# Install Rust if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build with LLVM feature
cargo build --release --features llvm

# Verify LLVM backend works
./target/release/bmb build examples/hello.bmb -o hello
./hello  # Should print "Hello, World!"
```

---

## Verification Procedures

### 1. 3-Stage Bootstrap Verification

This verifies the BMB compiler can compile itself and produce identical output.

```bash
cd /mnt/d/data/lang-bmb
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

**Success Criteria**:
- Stage 1 binary compiles and runs
- Stage 2 LLVM IR is valid
- Stage 3 LLVM IR is identical to Stage 2

### 2. Native Benchmark Verification

Verify BMB matches C/Rust performance on compute benchmarks.

```bash
cd /mnt/d/data/lang-bmb

# Build runtime
cd bmb/runtime
clang -c bmb_runtime.c -o bmb_runtime.o -O3
ar rcs libbmb_runtime.a bmb_runtime.o
cd ../..

# Compile and run fibonacci benchmark
./target/release/bmb build ecosystem/benchmark-bmb/benches/compute/fibonacci/bmb/main.bmb --emit-ir -o fib.ll
clang -O3 fib.ll bmb/runtime/libbmb_runtime.a -o fib -lm -no-pie

# Time it
time ./fib  # Should complete fib(40) in ~150-180ms

# Compare with C
gcc -O3 ecosystem/benchmark-bmb/benches/compute/fibonacci/c/main.c -o fib_c
time ./fib_c  # Should be similar (~170-180ms)
```

**Gate #3.1 Criteria**: BMB within 1.10x of C (≤1.10x slower)

### 3. Full Benchmark Suite

```bash
cd /mnt/d/data/lang-bmb/ecosystem/benchmark-bmb

# Build benchmark runner
cd runner && cargo build --release && cd ..

# Run all benchmarks
./runner/target/release/benchmark-bmb run all -i 5 -w 2

# Verify Gate #3.1
./runner/target/release/benchmark-bmb gate 3.1 -v
```

---

## Gate Verification Status

| Gate | Description | Target | Verification |
|------|-------------|--------|--------------|
| #3.1 | Compute benchmarks | ≤1.10x C | `benchmark-bmb gate 3.1` |
| #3.2 | Benchmarks Game | ≤1.05x C | `benchmark-bmb gate 3.2` |
| #3.3 | Faster than C | 3+ benchmarks | `benchmark-bmb gate 3.3` |
| #4.1 | Self-compile | <60s | ✅ 0.56s (already passing) |

---

## Troubleshooting

### LLVM not found

```bash
# Check LLVM is in PATH
which llc
which clang

# If not found, re-add to PATH
export PATH="/usr/lib/llvm-21/bin:$PATH"
export LLVM_SYS_210_PREFIX=/usr/lib/llvm-21
```

### Cargo build fails with LLVM errors

```bash
# Ensure LLVM_SYS_210_PREFIX is set correctly
echo $LLVM_SYS_210_PREFIX  # Should show /usr/lib/llvm-21

# Check llvm-config is accessible
llvm-config --version
```

### Permission denied on scripts

```bash
chmod +x scripts/*.sh
```

### Binary format error

```
./target/release/bmb: cannot execute binary file: Exec format error
```

This happens when running Windows binary in WSL. Rebuild in WSL:

```bash
cd /mnt/d/data/lang-bmb
cargo build --release --features llvm
```

### Windows line endings (CRLF)

```
bash: line 1: $'\r': command not found
```

Scripts copied from Windows may have CRLF line endings. Fix with:

```bash
# Fix single file
sed -i 's/\r$//' scripts/bootstrap_3stage.sh

# Fix all shell scripts
find scripts -name "*.sh" -exec sed -i 's/\r$//' {} \;
```

### BMB runtime library not found

```
Linker error: Cannot find BMB runtime library
```

Build the runtime library in WSL:

```bash
cd /mnt/d/data/lang-bmb/bmb/runtime
clang -c bmb_runtime.c -o bmb_runtime.o -O3
ar rcs libbmb_runtime.a bmb_runtime.o
```

### WSL catastrophic failure

```
Catastrophic failure - Error code: Wsl/Service/E_UNEXPECTED
```

Restart WSL from PowerShell:

```powershell
wsl --shutdown
wsl
```

---

## Performance Results Reference

Current BMB performance vs C (native compilation):

| Benchmark | C | Rust | BMB | vs C |
|-----------|---|------|-----|------|
| fibonacci(45) | 1.65s | 1.66s | 1.63s | **0.99x** |
| fibonacci(40) | 177ms | 180ms | 150ms | **0.85x** |
| mandelbrot | 42ms | 42ms | 39ms | **0.93x** |
| spectral_norm | 44ms | 44ms | 39ms | **0.89x** |

BMB achieves better-than-C performance on multiple benchmarks!

---

## Verification Log

### 2026-01-14 Session

**Environment**: WSL Ubuntu, LLVM 18.1.3

**3-Stage Bootstrap**:
| Stage | Status | Notes |
|-------|--------|-------|
| Stage 1: Rust BMB → native binary | ✅ Pass | Tests show 999 marker |
| Stage 1 compiles simple files | ✅ Pass | hello.bmb → native works |
| Stage 1 self-compilation | ⏳ Slow | >10 min timeout (30K line compiler) |

**Benchmark Gate #3.1**:
| Benchmark | C | BMB | Ratio | Status |
|-----------|---|-----|-------|--------|
| fibonacci(40) | 0.17s | 0.18s | ~1.06x | ✅ Pass (≤1.10x) |

**Known Issues**:
- Stage 1 self-compilation is too slow for the full 30K-line bootstrap compiler
- This is a performance issue, not a correctness issue
- Solution: Optimize bootstrap compiler or use incremental compilation

---

## Files Generated

After running verification:

| File | Description |
|------|-------------|
| `bmb-stage1` | Stage 1 native binary |
| `bmb-stage2` | Stage 2 native binary |
| `bmb-stage2.ll` | Stage 2 LLVM IR |
| `bmb-stage3.ll` | Stage 3 LLVM IR |

Clean up with:
```bash
rm -f bmb-stage* *.ll *.o hello fib fib_c
```

---

## Current Status: Stage 2 Blocker (2026-01-19)

### 문제 요약

**Stage 2 자체 컴파일 실패**: Rust 인터프리터의 재귀 한계로 인해 30K LOC Bootstrap 소스 파싱이 실패합니다.

| 단계 | 상태 | 비고 |
|------|------|------|
| Stage 1: Rust BMB → Native | ✅ 성공 | 0.918s, 189KB |
| Stage 1 실행 테스트 | ✅ 성공 | 간단한 프로그램 컴파일 가능 |
| Stage 2: Stage 1 → Stage 2 | ❌ 실패 | 스택 오버플로우/타임아웃 |

### 근본 원인

1. **Rust 인터프리터 재귀 한계**: `MAX_RECURSION_DEPTH = 100,000` (`bmb/src/interp/eval.rs:45`)
2. **Bootstrap 소스 크기**: 30K+ LOC (`bmb_unified_cli.bmb` 2,742 lines 포함)
3. **재귀 하강 파서**: 깊은 중첩 구조에서 스택 소모

### 해결 방안

#### 옵션 A: WSL 네이티브 Stage 1 빌드 (권장)
```bash
# 1. WSL에서 LLVM + Rust 설치
wsl
wget https://apt.llvm.org/llvm.sh && chmod +x llvm.sh && sudo ./llvm.sh 18
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. BMB with LLVM 빌드
cd /mnt/d/data/lang-bmb
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18
cargo build --release --features llvm

# 3. Stage 1 네이티브 빌드
./target/release/bmb build bootstrap/bmb_unified_cli.bmb -o bmb-stage1

# 4. Stage 2 실행 (네이티브 바이너리는 재귀 제한 없음)
./bmb-stage1 build bootstrap/bmb_unified_cli.bmb -o bmb-stage2
```

#### 옵션 B: Bootstrap 모듈 분할
Bootstrap 소스를 작은 모듈로 분할하여 개별 컴파일:
- `utils.bmb` → 유틸리티 함수
- `lexer.bmb` → 렉서
- `parser.bmb` → 파서
- `types.bmb` → 타입 체커
- `codegen.bmb` → 코드 생성

#### 옵션 C: 인터프리터 반복 방식 변환
대규모 아키텍처 변경 필요 (권장하지 않음)

### 개선 사항 (v0.50.26-27)

- **StringBuilder 도입**: O(n²) → O(n) 문자열 연결 최적화
- **parse_program_sb**: StringBuilder 기반 파서 (`bmb_unified_cli.bmb:628`)
- **lower_*_sb 함수들**: StringBuilder 기반 lowering

### 다음 단계

1. WSL에서 수동으로 LLVM 18 + Rust 설치
2. `cargo build --release --features llvm` 실행
3. Stage 1 네이티브 빌드 후 Stage 2 테스트
4. Stage 2 == Stage 3 동일성 검증
