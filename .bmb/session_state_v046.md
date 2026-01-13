# v0.46 Independence Phase - Session State

**Last Updated**: 2026-01-13
**Phase Status**: 진행중 (97% 완료) - Bootstrap 컴파일러 String 반환 타입 수정 완료

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
| - | v0.32 문법 지원 | 2026-01-13 | `//` 주석, braced if-else 파싱 (`b97656e`) |
| - | sb_build 반환 타입 | 2026-01-13 | MIR에서 String 타입 반환 수정 (`7811bec`) |
| - | **String 반환 타입 LLVM** | 2026-01-13 | `ret ptr` 생성 수정, 395 테스트 통과 |

### 대기 중인 태스크

| ID | 태스크 | 블로커 | 다음 단계 |
|----|--------|--------|----------|
| 46.3 | 3-Stage 검증 | WSL 네이티브 컴파일 필요 | WSL에서 검증 |
| 46.4 | Cargo.toml 제거 | 46.3 완료 필요 | 3-Stage 성공 후 진행 |
| 46.5 | DWARF 지원 | P1 우선순위 | 선택적 |
| 46.6 | 소스맵 | P1 우선순위 | 선택적 |

---

## v0.46 핵심 커밋 (2026-01-13 세션)

### Bootstrap 컴파일러 String 반환 타입 수정

**문제**: Bootstrap 컴파일러에서 String을 반환하는 함수가 `ret i64` 대신 `ret ptr` 생성 필요

**원인**:
1. `llvm_gen_return` 함수가 항상 `ret i64` 생성
2. `llvm_gen_fn_header`에서 반환 타입 추출 시 앞뒤 공백 처리 미흡
3. 함수 생성 체인에서 반환 타입 정보 전달 누락

**수정** (`bootstrap/compiler.bmb`):

1. **`trim` 함수 추가** (line 979-982):
   ```bmb
   fn trim(s: String) -> String =
       let start = low_skip_ws(s, 0);
       if start >= s.len() { "" } else { trim_end(s.slice(start, s.len())) };
   ```

2. **`extract_mir_return_type` 추가** (line 860-866):
   ```bmb
   fn extract_mir_return_type(mir: String) -> String =
       let arrow_pos = find_arrow(mir, 0);
       let brace_pos = find_char(mir, arrow_pos, 123);
       if arrow_pos >= mir.len() or brace_pos <= arrow_pos + 2 { "i64" }
       else { trim(mir.slice(arrow_pos + 2, brace_pos)) };
   ```

3. **`llvm_gen_return_typed` 추가** (line 984-989):
   ```bmb
   fn llvm_gen_return_typed(line: String, pos: i64, ret_type: String) -> String =
       let val_start = low_skip_ws(line, pos + 6);
       let val = line.slice(val_start, line.len());
       let llvm_ret = if ret_type == "bool" { "i1" } else if ret_type == "String" { "ptr" } else { "i64" };
       "  ret " + llvm_ret + " " + trim_end(val);
   ```

4. **타입 인식 함수 체인**:
   - `gen_function` → `gen_function_lines_typed`
   - `gen_function_sb` → `gen_function_lines_sb_typed`
   - `llvm_gen_fn_line_typed` → `llvm_gen_line_with_ret` → `llvm_gen_return_typed`

5. **`llvm_gen_fn_header` 수정** (line 1111-1112):
   - `trim_end` → `trim` 사용으로 앞뒤 공백 모두 제거

**검증**:
- 통합 테스트 9 추가: String 반환 함수가 `ret ptr` 생성 확인
- 총 395 테스트 통과 (386 단위 + 9 통합)

### CLI 인자 처리 수정

**문제**: `bmb run compiler.bmb`로 실행 시 "run" 서브커맨드가 인자로 전달됨

**수정** (`bootstrap/compiler.bmb` main 함수):
```bmb
fn main() -> i64 =
    let argc = arg_count();
    if argc >= 5 {
        // Interpreter with args: bmb run compiler.bmb input.bmb output.ll
        compile_file_to(get_arg(3), get_arg(4))
    } else if argc == 3 {
        let arg1 = get_arg(1);
        if arg1 == "run" { run_tests() }
        else { compile_file_to(get_arg(1), get_arg(2)) }
    } else { run_tests() };
```

---

## 이전 커밋 요약

### 2026-01-12: PHI 타입 추론 수정 (`55b5953`)
- If/Match PHI 결과 타입 `ctx.locals` 등록
- 메서드 호출 반환 타입 추적

### 2026-01-12: 문자열 연산 개선 (`d6dae1c`)
- StringBuilder API: `sb_new`, `sb_push`, `sb_build`, `sb_clear`
- 포인터 산술 연산

### 2026-01-13: get_arg/arg_count 타입 수정 (`b171ca0`, `96f1114`)
- LLVM codegen과 MIR lowering에서 올바른 반환 타입 설정

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
# Expected: 777 → 386 → 888 → 9 → 395 → 999
```

---

## 다음 단계

### 단기 (v0.46 완료)

1. **WSL에서 3-Stage Bootstrap 검증**
   - `scripts/bootstrap_3stage.sh` 실행
   - Stage 2 == Stage 3 바이너리 동일성 검증

2. **Cargo.toml 제거**
   - BMB-only 빌드 체인 확립

### 중기 (v0.47 준비)

1. **성능 Gate 검증**
   - WSL에서 벤치마크 실행
   - Gate #3.1 통과 확인

---

## Git 상태

- **브랜치**: main
- **v0.46 관련 커밋** (최신순):
  - (pending) - Bootstrap compiler String return type fix
  - `b97656e` - Bootstrap compiler v0.32 syntax support
  - `7811bec` - Fix sb_build return type
  - `96f1114` - Fix MIR lowering for CLI runtime function return types
  - `b171ca0` - Fix get_arg return type inference in LLVM text codegen
  - `55b5953` - Fix PHI type inference
  - `d6dae1c` - LLVM codegen string improvements

---

## 테스트 현황

| 테스트 스위트 | 통과 | 상태 |
|--------------|------|------|
| `bootstrap/lexer.bmb` | 777→264→999 | ✅ |
| `bootstrap/types.bmb` | ~530 | ✅ |
| `bootstrap/compiler.bmb` | 395 (386+9) | ✅ |
| Rust 컴파일러 테스트 | 1,753+ | ✅ |

---

## 참고 자료

- [Bootstrapping (compilers) - Wikipedia](https://en.wikipedia.org/wiki/Bootstrapping_(compilers))
- [Ken Thompson - Reflections on Trusting Trust](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ResearchStudy.pdf)
- [Reproducible Builds](https://reproducible-builds.org/)
