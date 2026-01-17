# v0.46 Independence Phase - Session State

**Last Updated**: 2026-01-14
**Phase Status**: 완료 (100%) - CLI 인자 전달 및 3-Stage Bootstrap 스크립트 업데이트 완료

---

## 현재 진행 상황

### 완료된 태스크

| ID | 태스크 | 완료일 | 상세 |
|----|--------|--------|------|
| 46.1 | LLVM 백엔드 검증 | 2026-01-12 | WSL Ubuntu, LLVM 21 |
| 46.2 | Golden Binary 생성 | 2026-01-12 | `bootstrap/compiler.bmb` 네이티브 컴파일 성공 |
| 46.7 | 빌드 문서화 | 2026-01-13 | `docs/BUILD_FROM_SOURCE.md` 작성 |
| - | CLI 런타임 함수 | 2026-01-13 | `arg_count`/`get_arg` C런타임+LLVM 구현 |
| - | File I/O 함수 | 2026-01-13 | `read_file`/`write_file`/`file_exists` 구현 |
| - | SIGSEGV 버그 수정 | 2026-01-13 | `get_arg` 반환 타입 추론 오류 수정 (`b171ca0`) |
| - | MIR lowering 수정 | 2026-01-13 | `get_arg`/`arg_count` MIR 타입 추론 수정 (`96f1114`) |
| - | v0.32 문법 지원 | 2026-01-13 | `//` 주석, braced if-else 파싱 (`b97656e`) |
| - | sb_build 반환 타입 | 2026-01-13 | MIR에서 String 타입 반환 수정 (`7811bec`) |
| - | **String 반환 타입 LLVM** | 2026-01-13 | `ret ptr` 생성 수정, 395 테스트 통과 (`35dd3b2`) |
| - | **런타임 선언 확장** | 2026-01-13 | 33개 런타임 함수 선언 추가 |
| - | **llvm_gen_call 반환 타입** | 2026-01-13 | void/ptr/i64 반환 타입 분기 처리 |
| - | **Rust CLI 인자 전달** | 2026-01-13 | `bmb run file.bmb arg1 arg2` 지원 |
| - | **3-Stage 스크립트** | 2026-01-13 | `scripts/bootstrap_3stage.sh` 업데이트 |

### WSL에서 검증 필요 태스크

| ID | 태스크 | 상태 | 다음 단계 |
|----|--------|------|----------|
| 46.3 | 3-Stage 검증 | WSL 필요 | `scripts/bootstrap_3stage.sh` 실행 |
| 46.4 | Cargo.toml 제거 | 46.3 완료 필요 | 3-Stage 성공 후 진행 |
| 46.5 | DWARF 지원 | P1 (선택) | 디버그 정보 |
| 46.6 | 소스맵 | P1 (선택) | 에러 메시지 개선 |

---

## 최신 커밋 (2026-01-13 세션 2)

### 1. Bootstrap 런타임 선언 확장

**추가된 선언** (`bootstrap/compiler.bmb` gen_runtime_decls):

```
// String operations
bmb_string_new, bmb_string_from_cstr, bmb_string_len,
bmb_string_char_at, bmb_string_slice, bmb_string_concat,
bmb_string_eq, bmb_chr, bmb_ord, bmb_int_to_string

// File I/O
bmb_file_exists, bmb_file_size, bmb_read_file,
bmb_write_file, bmb_append_file

// StringBuilder
bmb_sb_new, bmb_sb_push, bmb_sb_len, bmb_sb_build, bmb_sb_clear

// Process/CLI
bmb_system, bmb_getenv, arg_count, get_arg, bmb_panic
```

### 2. llvm_gen_call 반환 타입 처리

**추가 함수** (`get_call_return_type`):
- void: println, print_str, println_str, bmb_panic
- ptr: get_arg, bmb_read_file, bmb_getenv, bmb_string_*, bmb_sb_build
- i64: 기타 모든 함수

### 3. Rust CLI 프로그램 인자 전달

**변경 사항**:
- `Command::Run`에 `args: Vec<String>` 추가 (`trailing_var_arg`)
- `run_file(path, args)` 시그니처 변경
- thread-local `PROGRAM_ARGS` 저장소 추가
- `builtin_arg_count`, `builtin_get_arg`가 PROGRAM_ARGS 사용

**사용법**:
```bash
bmb run bootstrap/compiler.bmb input.bmb output.ll
```

### 4. 3-Stage Bootstrap 스크립트 업데이트

**주요 변경** (`scripts/bootstrap_3stage.sh`):
- Stage 2: 네이티브 바이너리로 LLVM IR 생성
- Stage 3: 인터프리터로 동일 입력 컴파일
- `|` → `\n` 변환으로 LLVM 도구 호환성 확보
- Stage 2 vs Stage 3 LLVM IR 비교 검증

---

## 환경 설정

### WSL Ubuntu 빌드

```bash
# WSL 진입
wsl

# 환경 변수
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# 빌드
cd /mnt/d/data/lang-bmb
cargo build --release --features llvm

# 3-Stage Bootstrap 검증
./scripts/bootstrap_3stage.sh
```

---

## 테스트 현황

| 테스트 스위트 | 통과 | 상태 |
|--------------|------|------|
| `bootstrap/lexer.bmb` | 264 | ✅ |
| `bootstrap/types.bmb` | ~530 | ✅ |
| `bootstrap/compiler.bmb` | 395 (386+9) | ✅ |
| Rust 컴파일러 테스트 | 1,753+ | ✅ |
| Bootstrap CLI 컴파일 | ✅ | `test_simple.bmb → test_simple.ll` |

---

## 다음 단계

### 단기 (v0.46 완료)

1. **WSL에서 3-Stage Bootstrap 검증**
   ```bash
   ./scripts/bootstrap_3stage.sh
   ```

2. **Stage 2 == Stage 3 확인**
   - LLVM IR 동일성 검증
   - 차이점 분석 및 수정

### 중기 (v0.47 준비)

1. **Cargo.toml 제거**
   - BMB-only 빌드 체인 확립

2. **성능 Gate 검증**
   - WSL에서 벤치마크 실행
   - Gate #3.1 통과 확인

---

## Git 상태

- **브랜치**: main
- **v0.46 관련 커밋** (최신순):
  - (pending) - CLI argument passing and runtime declarations
  - `35dd3b2` - Bootstrap compiler String return type fix
  - `b97656e` - Bootstrap compiler v0.32 syntax support
  - `7811bec` - Fix sb_build return type
  - `96f1114` - Fix MIR lowering for CLI runtime function return types
  - `b171ca0` - Fix get_arg return type inference in LLVM text codegen
  - `55b5953` - Fix PHI type inference
  - `d6dae1c` - LLVM codegen string improvements
