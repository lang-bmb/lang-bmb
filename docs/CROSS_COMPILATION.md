# BMB Cross-Compilation Design

> 다중 플랫폼 지원을 위한 크로스 컴파일 설계 문서

---

## Overview

BMB는 v1.0.0-beta에서 다음 타겟을 지원해야 합니다:
- Linux (x86_64)
- Windows (x86_64)
- macOS (x86_64, aarch64)
- WebAssembly (wasm32)

---

## 1. 타겟 아키텍처

### 1.1 지원 타겟 목록

| 타겟 | Triple | 상태 | 우선순위 |
|------|--------|------|----------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | 📋 계획 | P0 |
| Windows x86_64 | `x86_64-pc-windows-msvc` | 📋 계획 | P0 |
| macOS x86_64 | `x86_64-apple-darwin` | 📋 계획 | P1 |
| macOS ARM64 | `aarch64-apple-darwin` | 📋 계획 | P1 |
| WebAssembly | `wasm32-unknown-unknown` | 📋 계획 | P1 |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | 📋 계획 | P2 |

### 1.2 CLI 인터페이스

```bash
# 기본 (호스트 플랫폼)
bmb build main.bmb -o main

# 크로스 컴파일
bmb build main.bmb --target x86_64-unknown-linux-gnu -o main.linux
bmb build main.bmb --target x86_64-pc-windows-msvc -o main.exe
bmb build main.bmb --target wasm32-unknown-unknown -o main.wasm

# 타겟 목록 확인
bmb targets list

# 타겟 설치
bmb targets add x86_64-unknown-linux-gnu
```

---

## 2. LLVM 백엔드 수정

### 2.1 현재 구조

```rust
// bmb/src/codegen/llvm.rs (현재)
pub fn compile(mir: &Mir, output: &Path) -> Result<()> {
    let target = Target::get_default_triple();  // 호스트만 지원
    // ...
}
```

### 2.2 제안 구조

```rust
// bmb/src/codegen/llvm.rs (제안)
pub struct TargetConfig {
    pub triple: String,
    pub cpu: String,
    pub features: String,
    pub relocation_model: RelocModel,
    pub code_model: CodeModel,
}

impl TargetConfig {
    pub fn for_triple(triple: &str) -> Result<Self> {
        match triple {
            "x86_64-unknown-linux-gnu" => Ok(Self {
                triple: triple.to_string(),
                cpu: "generic".to_string(),
                features: "".to_string(),
                relocation_model: RelocModel::PIC,
                code_model: CodeModel::Default,
            }),
            "x86_64-pc-windows-msvc" => Ok(Self {
                triple: triple.to_string(),
                cpu: "generic".to_string(),
                features: "".to_string(),
                relocation_model: RelocModel::Default,
                code_model: CodeModel::Default,
            }),
            "wasm32-unknown-unknown" => Ok(Self {
                triple: triple.to_string(),
                cpu: "generic".to_string(),
                features: "+simd128".to_string(),
                relocation_model: RelocModel::Default,
                code_model: CodeModel::Default,
            }),
            _ => Err(Error::UnsupportedTarget(triple.to_string())),
        }
    }
}

pub fn compile(mir: &Mir, output: &Path, target: &TargetConfig) -> Result<()> {
    let target_machine = Target::from_triple(&target.triple)?
        .create_target_machine(
            &target.triple,
            &target.cpu,
            &target.features,
            OptLevel::Default,
            target.relocation_model,
            target.code_model,
        )?;
    // ...
}
```

---

## 3. 런타임 라이브러리

### 3.1 플랫폼별 런타임

각 타겟은 고유한 런타임 라이브러리가 필요합니다:

```
runtime/
├── common/
│   ├── string.c        # 문자열 연산
│   ├── vec.c           # 벡터 연산
│   └── hashmap.c       # 해시맵
├── linux/
│   ├── io.c            # Linux 파일 I/O
│   ├── process.c       # Linux 프로세스
│   └── time.c          # Linux 시간
├── windows/
│   ├── io.c            # Windows 파일 I/O
│   ├── process.c       # Windows 프로세스
│   └── time.c          # Windows 시간
├── macos/
│   ├── io.c            # macOS 파일 I/O
│   └── ...
└── wasm/
    ├── io.c            # WASM I/O (제한적)
    └── imports.c       # JS 인터페이스
```

### 3.2 런타임 함수 차이

| 함수 | Linux | Windows | macOS | WASM |
|------|-------|---------|-------|------|
| `read_file` | POSIX | Win32 | POSIX | Fetch API |
| `write_file` | POSIX | Win32 | POSIX | 미지원 |
| `get_time` | `clock_gettime` | `QueryPerformanceCounter` | `mach_absolute_time` | `performance.now` |
| `spawn_process` | `fork/exec` | `CreateProcess` | `fork/exec` | 미지원 |

### 3.3 WASM 제한사항

```bmb
// WASM에서 사용 불가 (컴파일 에러)
#[cfg(not(target = "wasm32"))]
fn read_file(path: String) -> String = ...;

// WASM 전용 대안
#[cfg(target = "wasm32")]
fn fetch_file(url: String) -> String = ...;
```

---

## 4. 링커 설정

### 4.1 플랫폼별 링커

| 타겟 | 링커 | 설명 |
|------|------|------|
| Linux | `ld` 또는 `lld` | ELF 바이너리 |
| Windows | `link.exe` 또는 `lld-link` | PE/COFF 바이너리 |
| macOS | `ld` 또는 `lld` | Mach-O 바이너리 |
| WASM | `wasm-ld` | WASM 모듈 |

### 4.2 링커 검색 순서

```rust
fn find_linker(target: &str) -> Result<PathBuf> {
    match target {
        "x86_64-unknown-linux-gnu" => {
            find_in_order(&["lld", "ld", "gcc"])
        }
        "x86_64-pc-windows-msvc" => {
            find_in_order(&["lld-link", "link.exe"])
        }
        "wasm32-unknown-unknown" => {
            find_in_order(&["wasm-ld"])
        }
        _ => Err(Error::NoLinker)
    }
}
```

### 4.3 크로스 컴파일 도구체인

```bash
# Linux에서 Windows 타겟
# 필요: mingw-w64 또는 MSVC 크로스 컴파일러
sudo apt install mingw-w64

# Linux에서 WASM 타겟
# 필요: wasm-ld (LLVM의 일부)
sudo apt install lld

# macOS에서 Linux 타겟
# 필요: musl 크로스 컴파일러
brew install filosottile/musl-cross/musl-cross
```

---

## 5. 빌드 시스템

### 5.1 타겟 정의 파일

```toml
# targets/x86_64-unknown-linux-gnu.toml
[target]
triple = "x86_64-unknown-linux-gnu"
data_layout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
pointer_width = 64
endian = "little"

[linker]
default = "lld"
fallback = ["ld", "gcc"]
flags = ["-pie", "-z", "relro", "-z", "now"]

[runtime]
path = "runtime/linux"
libs = ["c", "m", "pthread"]
```

### 5.2 타겟 관리 명령어

```bash
# 사용 가능한 타겟 목록
bmb targets list
# Output:
# x86_64-unknown-linux-gnu    [installed]
# x86_64-pc-windows-msvc      [available]
# wasm32-unknown-unknown      [available]

# 타겟 설치
bmb targets add x86_64-pc-windows-msvc
# Downloading runtime for x86_64-pc-windows-msvc...
# Installing linker wrapper...
# Done!

# 타겟 제거
bmb targets remove x86_64-pc-windows-msvc
```

---

## 6. 구현 계획

### Phase 1: 기반 작업 (1주)

1. **TargetConfig 구조체 구현**
   - 타겟 트리플 파싱
   - LLVM 타겟 머신 생성

2. **CLI 확장**
   - `--target` 플래그 추가
   - `bmb targets` 서브커맨드

3. **런타임 분리**
   - 공통/플랫폼별 분리
   - 빌드 스크립트 수정

### Phase 2: Linux 지원 (3일)

1. Linux 런타임 완성
2. 정적 링크 지원
3. 테스트 스위트 실행

### Phase 3: Windows 지원 (1주)

1. Windows 런타임 구현
2. MSVC 링커 통합
3. PE/COFF 생성 검증

### Phase 4: macOS 지원 (4일)

1. macOS 런타임 구현
2. Universal Binary 지원 (x86_64 + aarch64)
3. Code signing 통합

### Phase 5: WASM 지원 (1주)

1. WASM 런타임 구현
2. JS 바인딩 생성
3. 브라우저 테스트

---

## 7. 테스트 전략

### 7.1 크로스 컴파일 테스트

```bash
# CI에서 크로스 컴파일 테스트
for target in linux windows macos wasm; do
  bmb build tests/hello.bmb --target $target -o hello.$target
  # 에뮬레이터 또는 실제 환경에서 실행
done
```

### 7.2 QEMU 에뮬레이션

```bash
# Linux ARM64 테스트
qemu-aarch64 ./hello.linux-arm64

# Windows 테스트 (Wine)
wine ./hello.exe
```

### 7.3 CI 매트릭스

```yaml
# .github/workflows/cross.yml
jobs:
  cross-compile:
    strategy:
      matrix:
        host: [ubuntu-latest, macos-latest, windows-latest]
        target: [linux, windows, macos, wasm]
    steps:
      - uses: actions/checkout@v4
      - name: Build for ${{ matrix.target }}
        run: bmb build test.bmb --target ${{ matrix.target }}
```

---

## 8. 알려진 제한사항

### 현재 제한

| 제한 | 설명 | 해결 계획 |
|------|------|----------|
| 호스트 전용 | 현재 호스트 플랫폼만 지원 | 이 문서 구현 |
| 정적 링크만 | 동적 라이브러리 미지원 | v1.1 예정 |
| C 런타임 의존 | libc 필수 | musl 옵션 검토 |

### WASM 특별 제한

| 기능 | 상태 | 대안 |
|------|------|------|
| 파일 I/O | 미지원 | Fetch API |
| 프로세스 | 미지원 | Worker |
| 소켓 | 미지원 | WebSocket |
| 스레드 | 제한적 | SharedArrayBuffer |

---

## 9. 참고 자료

- [LLVM Target Triple](https://llvm.org/docs/LangRef.html#target-triple)
- [Rust Cross Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [WASM Target Features](https://webassembly.org/features/)

---

## 버전 이력

| 날짜 | 버전 | 변경 |
|------|------|------|
| 2026-01-14 | 0.1 | 설계 문서 초안 |

