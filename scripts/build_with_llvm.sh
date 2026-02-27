#!/bin/bash
# MSYS2 UCRT64 환경에서 BMB 빌드 (v0.50.53)
# 실행: C:/msys64/msys2_shell.cmd -ucrt64 -defterm -no-start -here -c "./build_with_llvm.sh"

set -e

# Rust/Cargo 경로 추가
export PATH="/c/Users/iyu-nb02/.cargo/bin:$PATH"

# 프로젝트 디렉토리로 이동
cd /d/data/lang-bmb

# LLVM 버전 확인
echo "=== LLVM Version ==="
llvm-config --version

# 빌드 실행 (GNU 타겟 사용)
echo "=== Building BMB with LLVM (x86_64-pc-windows-gnu) ==="
cargo build --release --features llvm -p bmb --target x86_64-pc-windows-gnu

# 결과 확인
echo "=== Build Complete ==="
ls -la target/x86_64-pc-windows-gnu/release/bmb.exe

# 간단한 테스트
echo "=== Testing ==="
./target/x86_64-pc-windows-gnu/release/bmb.exe --version
