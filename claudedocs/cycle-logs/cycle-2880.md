# Cycle 2880: str_char_at native 포팅 + struct pass-by-value 스펙 검사
Date: 2026-05-15

## Re-plan
Carry-Forward: struct pass-by-value P0 버그 수정 (Cycle 2879). 그러나 advisor 권고: 스펙 확인 우선.
스펙 검사 결과: `docs/SPECIFICATION.md` + `docs/LANGUAGE_REFERENCE.md` 양쪽 모두 구조체 파라미터 의미론(값 vs 참조) 침묵 → HUMAN-decision으로 파킹.
대안 범위 채택: str_char_at native 포팅 (Cycle 2877~2879 Carry-Forward).

## Scope & Implementation

### STEP 1: 스펙 검사 (struct pass-by-value)
- `grep "pass.*by.*value|pass.*by.*reference|value.*semantic" docs/SPECIFICATION.md` → 0 결과
- `grep "pass.*by.*value|pass.*by.*reference|by value|by reference" docs/LANGUAGE_REFERENCE.md` → 0 결과 (f64 벡터 관련 1건만)
- **결론**: 스펙 침묵 → 자율 P0 fix 근거 없음 → HUMAN-decision 파킹

### STEP 2: str_char_at native 포팅
- `str_char_at(s: String, idx: i64) -> String` — i64(byte value)를 반환하는 `char_at`과 달리, 단일 문자 String 반환
- `char_to_string(int32_t c)` 패턴 이미 runtime에 존재 → `bmb_str_char_at` 신규 함수로 래핑
- 변경 3곳:
  1. `bmb/runtime/bmb_runtime.c`: `bmb_str_char_at(BmbString* s, i64 idx) -> BmbString*` 추가 (line 1362)
  2. `bmb/src/codegen/llvm_text.rs`: `declare nonnull ptr @bmb_str_char_at(ptr nonnull, i64)` 선언 추가 (line 853)
  3. `llvm_text.rs` dispatch: `"str_char_at" => "bmb_str_char_at"` (line ~6562)
  4. `llvm_text.rs` return_type_of_fn: `| "str_char_at" | "bmb_str_char_at" => "ptr"` (line ~9155)

### 검증:
- `tests/native_str_char_at.bmb`: "hello" 문자열에서 str_char_at 3종, str_len/str_contains로 내용 검증
  - interpreter: 6 ✅
  - native build: 6 ✅

### bmb_reference.md 수정:
- line 85: `interp-only` → `native v0.98.9+`
- line 926: native support 목록에 `str_char_at` 추가
- line 927: interpreter-only 목록에서 `str_char_at` 제거
- Common Pitfalls: native struct parameter behavior(포인터 전달) 경고 문구 추가

## Verification & Defect Resolution
- interpreter: 6 ✅
- native: 6 ✅
- cargo test: 6249 PASS (0 FAIL) ✅

## Reflection
- Scope fit: str_char_at 포팅은 예상대로 단순 패턴 (runtime 1함수 + llvm_text.rs 3곳). struct pass-by-value 스펙 검사는 올바른 선행 단계였음.
- 스펙 침묵이 확인됨: BMB 언어 스펙이 struct 파라미터 의미론을 정의하지 않음 → 어느 쪽을 정답으로 할지 인간 결정 필요.
- Latent: `runtime_param_type`에 `str_char_at`가 없어도 동작 — i64 arg는 이미 i64 타입이고 ptr은 sext 무관. 별도 추가 불필요.
- Philosophy drift: 없음.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. **struct pass-by-value HUMAN-decision** — 스펙이 침묵. BMB가 값-의미론(interpreter처럼 memcpy)인지 참조-의미론(native처럼 포인터)인지 정의 필요. 결정 후: 값-의미론이면 call site에서 `alloca + llvm.memcpy` 추가; 참조-의미론이면 interpreter 수정 + docs 업데이트.
  2. **runtime_param_type 장기 해결** — types/mod.rs 시그니처 직접 참조 구조
  3. **to_string native 포팅** — i64/f64/bool 케이스는 기존 bmb_int_to_string/f64_to_string 활용 가능; String 케이스는 identity. 타입 기반 dispatch가 필요해 MIR 레벨 처리 검토 필요.
- Pending Human Decisions:
  - **struct parameter semantics**: BMB 스펙은 값-의미론인가 참조-의미론인가? (문서화 + 구현 방향 결정)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2881 — bmb_reference.md의 나머지 interpreter-only 항목 재검증 (already-works 탐색) 또는 to_string 부분 native 포팅 (i64/f64 케이스)
