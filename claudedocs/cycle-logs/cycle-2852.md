# Cycle 2852: str_to_upper / str_to_lower / str_char_at
Date: 2026-05-15

## Re-plan
Carry-Forward (2851): None. 계획대로 fundamental string case/char builtins 구현.

## Scope & Implementation

**str_to_upper** (Cycle 2852, interpreter-only):
- `str_to_upper(s: String) -> String`: Rust `to_uppercase()` — Unicode 완전 지원
- 대문자 정규화, 비교 표준화 등 AI 생성 코드에서 필수 패턴

**str_to_lower** (Cycle 2852, interpreter-only):
- `str_to_lower(s: String) -> String`: Rust `to_lowercase()` — Unicode 완전 지원

**str_char_at** (Cycle 2852, interpreter-only):
- `str_char_at(s: String, idx: i64) -> String`: 인덱스의 문자를 String으로 반환
- 기존 `char_at(s, idx) -> char`와 달리 `String` 반환 — AI 코드에서 문자열 조작 시 직접 사용 가능
- 범위 밖 idx → `"\0"` (널 문자 문자열) 반환

변경 파일:
- `bmb/src/interp/eval.rs`: 3종 함수 구현 + 등록
- `bmb/src/types/mod.rs`: 3종 타입 서명 추가
- `bmb/tests/integration.rs`: `test_interp_str_case_and_char` (4케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 3종 문서화 + notes 갱신

## Verification & Defect Resolution
- test_interp_str_case_and_char: 4/4 통과 ✅
- cargo test --release 전체: **2377 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ str_to_upper/lower — Unicode-correct (Rust stdlib), 상태 정규화 패턴 지원
- ✅ str_char_at — `char_at` (→ char) 와 분리된 String-returning 버전. AI codegen에서 혼용 없이 일관된 String API
- 경고: `InterpMini::consume` dead_code 경고 여전히 존재 (Cycle 2848부터, 기능 영향 없음)

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * `{fn_call(args)}` 보간 — InterpMini에 함수 호출 파싱 추가
  * 필드 복합 할당 native 지원 (codegen)
  * vec_remove/reverse/fill + svec_sort/contains/remove — API 갭
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `vec_remove` / `vec_reverse` / `vec_fill` (Cycle 2853) — vec API 완성
