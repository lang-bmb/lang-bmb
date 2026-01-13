# v0.46 Independence Phase - Session State

**Last Updated**: 2026-01-13
**Phase Status**: 진행중 (95% 완료) - 3-Stage Bootstrap 블로커 발견

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
| - | bmb-unified 컴파일 | 2026-01-13 | `bmb_unified_cli.bmb` 네이티브 바이너리 생성 성공 |
| - | SIGSEGV 버그 수정 | 2026-01-13 | `get_arg` 반환 타입 추론 오류 수정 (`b171ca0`) |
| - | MIR lowering 수정 | 2026-01-13 | `get_arg`/`arg_count` MIR 타입 추론 수정 (`96f1114`) |

### 대기 중인 태스크

| ID | 태스크 | 블로커 | 다음 단계 |
|----|--------|--------|----------|
| 46.3 | 3-Stage 검증 | **bmb-stage1 파싱 버그** | 아래 상세 참조 |
| 46.4 | Cargo.toml 제거 | 46.3 완료 필요 | 3-Stage 성공 후 진행 |
| 46.5 | DWARF 지원 | P1 우선순위 | 선택적 |
| 46.6 | 소스맵 | P1 우선순위 | 선택적 |

### 🚧 3-Stage Bootstrap 블로커

**증상**: `bmb-stage1`이 "fn" 키워드 포함 파일 파싱 시 SIGSEGV 발생

**분석 결과**:
- `compiler.bmb` (테스트 하네스) → 네이티브 작동 ✓
- `lexer.bmb`, `types.bmb` → 네이티브 작동 ✓
- CLI 인자/파일 처리 코드 → 네이티브 작동 ✓
- `bmb_unified_cli.bmb` → "fn" 키워드 파싱 시 SIGSEGV ✗

**재현 방법**:
```bash
./bmb-stage1 test.bmb out.ll  # where test.bmb contains "fn main() -> i64 = 1;"
# => Segmentation fault
```

**작동하는 케이스**:
- 빈 파일 → 정상 (빈 프로그램 반환)
- 단일 문자 "a" → 정상 (에러 반환)
- "f " (공백 있음) → 정상

**크래시 케이스**:
- "fn" (키워드) → SIGSEGV
- "fn main" → SIGSEGV

**추정 원인**:
- `bmb_unified_cli.bmb`의 파서 코드에서 특정 문자열 연산 또는 재귀 처리 시 문제
- 동일 코드가 인터프리터에서는 작동하므로, 네이티브 컴파일 특유의 문제로 추정
- 스택 오버플로우 또는 타입 불일치 가능성

**임시 해결책**:
- `compiler.bmb` 테스트 하네스로 Stage 1 검증 완료 (777→999 패턴)

---

## v0.46 핵심 커밋

### 2026-01-12: PHI 타입 추론 수정 (`55b5953`)

**문제**: Bootstrap 컴파일러를 네이티브로 컴파일하면 SIGSEGV 발생

**원인** (4개 버그):
1. PHI 결과 타입이 `ctx.locals`에 등록되지 않음
2. 메서드 호출 (`slice()` 등) 반환 타입 미추적
3. 런타임 함수 반환 타입 테이블 불완전
4. `constant_type()` 헬퍼의 부작용 문제

**수정** (`bmb/src/mir/lower.rs`):
```rust
// If 표현식 PHI 타입 등록 (line 326-329)
let phi_var = ctx.fresh_var();
ctx.locals.insert(phi_var.clone(), result_type.clone());

// 메서드 호출 반환 타입 (line 852-860)
let ret_type = match method.as_str() {
    "len" | "byte_at" => MirType::I64,
    "slice" => MirType::String,
    _ => ctx.func_return_types.get(method).cloned().unwrap_or(MirType::I64),
};
```

### 2026-01-12: 문자열 연산 개선 (`d6dae1c`)

**추가된 기능**:
- `bmb_string_from_cstr`: C 문자열 → BmbString 래핑
- StringBuilder API: `sb_new`, `sb_push`, `sb_build`, `sb_clear`
- 포인터 산술 연산 (`Add`, `Sub`)

### 2026-01-13: get_arg 반환 타입 수정 (`b171ca0`)

**문제**: `bmb-unified` 네이티브 바이너리 실행 시 SIGSEGV 발생

**원인**:
- `llvm_text.rs`의 `infer_call_return_type`에서 `get_arg` 함수 누락
- `get_arg`가 `ptr` 대신 `i64`를 반환한다고 잘못 추론
- 결과적으로 `BmbString*` 포인터가 `i64`로 잘리면서 `read_file` 호출 시 crash

**수정** (`bmb/src/codegen/llvm_text.rs:2036-2037`):
```rust
// v0.46: ptr return - CLI argument functions
"get_arg" | "bmb_get_arg" => "ptr",
```

### 2026-01-13: MIR lowering 타입 추론 수정 (`96f1114`)

**문제**: `b171ca0` 수정 후에도 SIGSEGV 지속

**원인** (Root Cause):
- MIR lowering (`lower.rs`)에서 `get_arg` 반환 타입이 `MirType::I64`로 기본 설정
- 이로 인해 `func.locals`에 잘못된 타입 등록
- LLVM codegen의 `build_place_type_map`에서 locals 타입을 먼저 읽어 `i64`로 설정
- Call 처리 시 타입 업데이트되지만, 일부 경로에서 잘못된 타입 참조

**수정** (`bmb/src/mir/lower.rs:462-468`):
```rust
// String-returning runtime functions
// v0.46: get_arg returns string (pointer to BmbString)
"int_to_string" | "read_file" | "slice" | "digit_char" | "get_arg" => MirType::String,
// i64-returning runtime functions
// v0.46: arg_count returns i64
"byte_at" | "len" | "strlen" | "cstr_byte_at" | "arg_count" => MirType::I64,
```

### 2026-01-13: CLI 런타임 함수 구현

**구현 내용**:

1. **C 런타임** (`bmb/runtime/bmb_runtime.c`):
   ```c
   // 전역 변수
   static int g_argc = 0;
   static char** g_argv = NULL;

   // main()에서 argc/argv 저장
   int main(int argc, char** argv) {
       g_argc = argc;
       g_argv = argv;
       return (int)bmb_user_main();
   }

   // 런타임 함수
   int64_t bmb_arg_count(void);
   char* bmb_get_arg(int64_t index);
   ```

2. **LLVM codegen** (`bmb/src/codegen/llvm.rs`):
   ```rust
   // arg_count() -> i64
   self.functions.insert("arg_count".to_string(), arg_count_fn);

   // get_arg(index: i64) -> ptr
   self.functions.insert("get_arg".to_string(), get_arg_fn);
   ```

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

# Bootstrap 테스트
./target/release/bmb build bootstrap/compiler.bmb -o bootstrap_compiler
./bootstrap_compiler
# Expected: 777 → 385 → 888 → 8 → 393 → 999
```

### 검증 명령어

```bash
# 3-Stage Bootstrap (스크립트)
./scripts/bootstrap_3stage.sh

# 수동 검증
./target/release/bmb build bootstrap/compiler.bmb -o bmb-stage1
./bmb-stage1  # 테스트 실행 (777...999)
```

---

## 알려진 제한사항

1. **`compiler.bmb`는 테스트 하네스**
   - `build` CLI 명령 없음
   - 3-Stage 자체 컴파일에는 `bmb_unified_cli.bmb` 사용 필요

2. ~~**런타임 함수 미구현**~~ ✅ 해결됨 (2026-01-13)
   - `arg_count()`: C 런타임 + LLVM codegen 구현 완료
   - `get_arg(n)`: C 런타임 + LLVM codegen 구현 완료

3. **Windows 네이티브 빌드 불가**
   - LLVM 미지원
   - WSL Ubuntu 사용 필수

---

## 다음 단계

### 단기 (v0.46 완료)

1. **`bmb_unified_cli.bmb` 완성**
   - `arg_count`, `get_arg` 런타임 함수 구현
   - `build` 서브커맨드 추가

2. **3-Stage Bootstrap 완료**
   - `scripts/bootstrap_3stage.sh` 실행
   - Stage 2 == Stage 3 바이너리 동일성 검증

3. **Cargo.toml 제거**
   - BMB-only 빌드 체인 확립

### 중기 (v0.47 준비)

1. **성능 Gate 검증**
   - WSL에서 벤치마크 실행
   - Gate #3.1 통과 확인

---

## Git 상태

- **브랜치**: main
- **최신 커밋**: `96f1114` - Fix MIR lowering for CLI runtime function return types
- **v0.46 관련 커밋**:
  - `96f1114` - Fix MIR lowering for CLI runtime function return types
  - `b171ca0` - Fix get_arg return type inference in LLVM text codegen
  - `330bab7` - Add File I/O runtime functions for CLI Independence
  - `86ec840` - Implement arg_count/get_arg runtime functions
  - `d8eca16` - Add 3-stage bootstrap script and build documentation
  - `55b5953` - Fix PHI type inference
  - `d6dae1c` - LLVM codegen string improvements

---

## 문서 현황

| 문서 | 상태 | 위치 |
|------|------|------|
| BUILD_FROM_SOURCE.md | ✅ 완료 | `docs/BUILD_FROM_SOURCE.md` |
| ROADMAP.md | ✅ 최신화 | `docs/ROADMAP.md` |
| bootstrap_3stage.sh | ✅ 완료 | `scripts/bootstrap_3stage.sh` |

---

## 참고 자료

- [Bootstrapping (compilers) - Wikipedia](https://en.wikipedia.org/wiki/Bootstrapping_(compilers))
- [Ken Thompson - Reflections on Trusting Trust](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ResearchStudy.pdf)
- [Reproducible Builds](https://reproducible-builds.org/)
