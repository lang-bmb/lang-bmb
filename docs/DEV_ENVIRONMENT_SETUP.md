# BMB 개발환경 셋업 가이드

BMB 컴파일러 개발에 필요한 모든 도구와 환경 설정을 정리한 문서입니다.

---

## 필수 요구사항 요약

| 도구 | 버전 | 용도 |
|------|------|------|
| **Rust** | stable 1.75+ | 컴파일러 빌드 |
| **LLVM** | 21.x | IR 최적화, 네이티브 코드 생성 |
| **C 컴파일러** | GCC 또는 Clang | 런타임 컴파일 및 링킹 |
| **Python** | 3.6+ | 벤치마크 비교 스크립트 |
| **Git** | 2.x+ | 소스 관리 |

---

## 플랫폼별 설치

### Windows (MSYS2/MinGW) — 기본 개발 환경

#### 1. MSYS2 설치

[https://www.msys2.org](https://www.msys2.org)에서 설치 후 UCRT64 환경 사용.

```bash
# MSYS2 UCRT64 터미널에서 실행
pacman -Syu
pacman -S mingw-w64-ucrt-x86_64-llvm \
          mingw-w64-ucrt-x86_64-clang \
          mingw-w64-ucrt-x86_64-gcc \
          mingw-w64-ucrt-x86_64-make
```

#### 2. Rust 설치

```bash
# https://rustup.rs 에서 설치
rustup default stable
rustup target add x86_64-pc-windows-gnu
```

#### 3. PATH 설정

`C:/msys64/ucrt64/bin`이 시스템 PATH에 포함되어야 합니다.

#### 4. 빌드

```bash
# Windows에서는 반드시 MinGW 타겟 사용 (LLVM 헤더 충돌 방지)
cargo build --release --features llvm --target x86_64-pc-windows-gnu
```

> **주의**: `--target x86_64-pc-windows-gnu` 없이 빌드하면 MSVC와 MinGW LLVM 헤더 간 충돌이 발생합니다.

---

### Linux (Ubuntu/Debian)

#### 1. 시스템 패키지

```bash
sudo apt update
sudo apt install -y build-essential curl wget zlib1g-dev libzstd-dev
```

#### 2. LLVM 21 설치

```bash
wget -qO- https://apt.llvm.org/llvm.sh | sudo bash -s -- 21 all
sudo apt install -y clang-21 lld-21 llvm-21-dev
```

#### 3. Rust 설치

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

#### 4. 환경변수 (~/.bashrc에 추가)

```bash
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"
```

#### 5. 빌드

```bash
cargo build --release --features llvm
```

---

### macOS

#### 1. Xcode Command Line Tools

```bash
xcode-select --install
```

#### 2. Homebrew + LLVM

```bash
brew install llvm@21
```

#### 3. Rust 설치

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

#### 4. 환경변수 (~/.zshrc에 추가)

```bash
export LLVM_SYS_211_PREFIX=$(brew --prefix llvm@21)
export PATH="$(brew --prefix llvm@21)/bin:$PATH"
```

#### 5. 빌드

```bash
cargo build --release --features llvm
```

---

### WSL2 (Windows Subsystem for Linux)

Windows에서 네이티브 컴파일이 필요할 때 WSL2를 사용할 수 있습니다.

```powershell
# PowerShell (관리자)에서 실행
wsl --install -d Ubuntu-22.04
```

설치 후 위의 **Linux (Ubuntu/Debian)** 섹션을 따르세요. 자세한 내용은 [WSL_DEVELOPMENT.md](WSL_DEVELOPMENT.md) 참조.

---

## 환경변수

| 변수 | 필수 | 설명 | 예시 |
|------|------|------|------|
| `LLVM_SYS_211_PREFIX` | 조건부 | LLVM 설치 경로 (PATH에서 자동 감지 안 될 때) | `/usr/lib/llvm-21` |
| `BMB_RUNTIME_PATH` | 선택 | BMB 런타임 라이브러리 경로 | `d:/data/lang-bmb/bmb/runtime` |
| `PATH` | 필수 | LLVM 바이너리(`opt`, `llc`, `clang`) 포함 | — |

---

## C 런타임 빌드

BMB로 컴파일된 실행파일은 C 런타임에 링크해야 합니다.

```bash
cd bmb/runtime

# 런타임 오브젝트 컴파일
clang -c bmb_runtime.c -o bmb_runtime.o -O3
clang -c bmb_event_loop.c -o bmb_event_loop.o -O3

# 정적 라이브러리 생성
ar rcs libbmb_runtime.a bmb_runtime.o bmb_event_loop.o
```

플랫폼별 링크 플래그:

| 플랫폼 | 추가 링크 플래그 |
|--------|-----------------|
| Linux | `-lm -lpthread` |
| Windows | `-lm -lws2_32` |
| macOS | `-lm -lpthread` |

---

## 빌드 모드

### 인터프리터만 (LLVM 불필요)

```bash
cargo build --release
```

타입 체크, 인터프리터 실행, 포매터, 린터 사용 가능. 네이티브 컴파일은 불가.

### 네이티브 컴파일 포함 (LLVM 필요)

```bash
# Linux/macOS
cargo build --release --features llvm

# Windows
cargo build --release --features llvm --target x86_64-pc-windows-gnu
```

---

## 설치 검증

모든 도구가 올바르게 설치되었는지 확인:

```bash
# 1. Rust 툴체인
cargo --version          # 1.75+
rustc --version

# 2. LLVM 도구
opt --version            # LLVM 21.x
llc --version
clang --version

# 3. C 컴파일러/링커
gcc --version            # Windows: MinGW-w64

# 4. Python (벤치마크용)
python3 --version        # 3.6+

# 5. BMB 컴파일러 빌드 및 테스트
cargo test --release
```

### 전체 검증 (부트스트랩 포함)

```bash
# 빠른 검증 (~2분): 테스트 + Stage 1 부트스트랩
./scripts/quick-check.sh

# 전체 검증 (~15분): 3-Stage 부트스트랩 + 벤치마크
./scripts/full-cycle.sh
```

---

## 에디터/IDE 설정

### VS Code (권장)

1. `ecosystem/vscode-bmb/` 확장 설치 (BMB 구문 강조 + LSP)
2. 권장 설정:

```json
{
  "files.associations": {
    "*.bmb": "bmb"
  }
}
```

### 기타 에디터

- **Tree-sitter 지원 에디터** (Neovim, Helix 등): `ecosystem/tree-sitter-bmb/` 문법 사용
- **LSP 지원**: `bmb lsp` 명령으로 Language Server 실행

---

## 자주 발생하는 문제

| 증상 | 원인 | 해결 |
|------|------|------|
| `No suitable version of LLVM was found` | LLVM 경로 미설정 | `LLVM_SYS_211_PREFIX` 환경변수 설정 |
| LLVM 헤더 충돌 (Windows) | MSVC 타겟으로 빌드 | `--target x86_64-pc-windows-gnu` 추가 |
| `undefined reference` 링크 에러 | 런타임 누락 | `bmb_runtime.c` + `bmb_event_loop.c` 모두 컴파일 |
| `ws2_32` 링크 에러 (Windows) | 소켓 라이브러리 누락 | `-lws2_32` 링크 플래그 추가 |
| `opt` command not found | LLVM PATH 미설정 | LLVM bin 디렉토리를 PATH에 추가 |
| `cargo test` 실패 | `--release` 누락 | 반드시 `cargo test --release` 사용 |
| `bc` not found (Git Bash) | Windows Git Bash 제약 | awk/python으로 대체 (스크립트에서 자동 처리) |

---

## 참고 문서

- [BUILD_FROM_SOURCE.md](BUILD_FROM_SOURCE.md) — 소스에서 빌드하기
- [WSL_DEVELOPMENT.md](WSL_DEVELOPMENT.md) — WSL 환경 상세 가이드
- [BOOTSTRAP_BENCHMARK.md](BOOTSTRAP_BENCHMARK.md) — 부트스트랩 + 벤치마크 사이클
- [ARCHITECTURE.md](ARCHITECTURE.md) — 컴파일러 아키텍처
