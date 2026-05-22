# Cycle 3040: run-bench-tests.bmb 완성 + exec_with_stdin 빌트인
Date: 2026-05-22

## Re-plan
이전 사이클에서 `exec_with_stdin` 빌트인 구현 완료. 이번 사이클에서 `scripts/run-bench-tests.bmb` 작성 및 타입 에러 해결.

## Scope & Implementation

### exec_with_stdin 빌트인 (Cycle 3034-3039에서 착수, 이번 완결)
- `bmb/runtime/bmb_runtime.c`: CreatePipe/CreateProcess (Windows) + fork/pipe (POSIX) 구현
- `bmb/src/interp/eval.rs`: `Command::new().stdin(piped()).spawn()` + `wait_with_output()`
- `bmb/src/types/mod.rs`: `(String, String, String) -> String` 등록
- `bmb/src/codegen/llvm_text.rs` + `llvm.rs`: inkwell/text 양쪽 등록

### run-bench-tests.bmb 타입 에러 해결
**근본 원인**: `s.char_at(pos)` 메서드가 `String`을 반환하는데 (자유함수 `char_at`은 `Char` 반환),
정수 코드와 비교(`ch == 34`)하면서 타입 추론 오염 → 다운스트림 함수들까지 cascade.

**수정**:
1. `s.char_at()` → `s.char_code_at()` (i64 반환)
2. `file_read` → `read_file` (올바른 빌트인 이름)
3. 깊은 let-chain 중첩 해소: `run_tests` 내부를 `run_one_test`로 추출
   (타입 에러 위치를 격리해서 원인 추적 용이하게 함)
4. stdin 공백→줄바꿈 변환: `str_replace(stdin_str, " ", "\n")`
   — `read_int()`이 전체 라인을 읽고 파싱하므로 공백 구분 입력 불가

## Verification & Defect Resolution
- `bmb check scripts/run-bench-tests.bmb`: ✅ (warning 14개, error 0)
- binary_search: 15/15 ✅
- fibonacci: 15/15 ✅
- gcd: 15/15 ✅
- `cargo test --release`: 실행 중

## Reflection
- `s.char_at()` vs `s.char_code_at()` 구분이 문서화 부족 — 메서드 이름만 봐서는 반환 타입 예측 불가
- `read_int()` 줄 단위 파싱은 competitive programming 관례(`scanf`처럼 토큰 단위)와 다름
- 타입 에러가 발생 위치와 다른 곳에서 보고되는 BMB 타입 추론 특성 확인 (cascade 오염)

## Carry-Forward
- Actionable: Cycle 3041+ 계속 진행 (M6 dogfooding 계획)
- Structural Improvement Proposals:
  - `read_int()` 토큰 기반 파싱 전환 검토 (stdin buffer로 관리)
  - `s.char_at()` 이름을 `s.char_str_at()`로 rename하여 혼동 방지
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: `exec_with_stdin` 네이티브 빌드 지원 (현재 인터프리터만), M6 다음 단계
