# Cycle 3079: M7-2 COMPLETE — String SMT Theory + Track B 계약 검증
Date: 2026-05-25

## Re-plan
Cycle 3078 Carry-Forward: M7-2 착수 — Rust SMT String theory 지원 추가.
Rule 6 P0 예외 적용: `bmb verify`가 String 조건을 `total:0`으로 silently skip하는 것은
검증 정확성 버그 (시스템이 "통과"를 보고하지만 실제로는 미검사).

## Scope & Implementation

### 변경 파일: `bmb/src/smt/translator.rs`

9개 targeted change:

1. **`SmtSort::Str` 추가** — `SmtSort` enum에 String sort 추가
2. **`SmtLibGenerator.has_strings` 필드** — String sort 선언 여부 추적
3. **`declare_var` Str 처리** — `SmtSort::Str → "String"` 선언, `has_strings = true`
4. **`generate` logic 전환** — `has_strings` 시 `QF_LIA → ALL`
5. **`clear` 초기화** — `has_strings = false` 추가
6. **`type_to_sort` 수정** — `Type::String → SmtSort::Str` (이전: `SmtSort::Int`)
7. **`StringLit` 번역** — `"s"` → `"\"s\""` (SMT-LIB2 string literal)
8. **`MethodCall.len()` 번역** — String 변수에 대한 `.len()` → `(str.len var)`
9. **`type_to_smt` 수정** — `Type::String → Ok("String")` (이전: `Err(UnsupportedFeature)`)

### 변경 파일: `bootstrap/compiler.bmb`

Track B 3개 함수에 실제 `pre` 계약 추가:
- `method_to_runtime_fn(method: String)`: `pre method.len() > 0`
- `get_call_return_type(fn_name: String)`: `pre fn_name.len() > 0`
- `is_string_returning_fn(name: String)`: `pre name.len() > 0`

### 테스트 업데이트

기존 테스트 3개 수정 + 신규 테스트 7개 추가 (6264 → 6271).

## Verification & Defect Resolution

- `cargo test --release`: **6271 PASS** ✅
- `bmb verify bootstrap/compiler.bmb`: **total:1513, verified:1513** ✅
  - Track B 3개: `✓ method_to_runtime_fn: pre verified`, `✓ get_call_return_type: pre verified`, `✓ is_string_returning_fn: pre verified`
- 3-Stage Fixed Point: S3 IR == S4 IR ✅ (새 hash: `ea550bf3`)
- Stage 1 빌드: `build_success` ✅

### total:1513 설명

Track B 3개 함수는 이전에 `// invariant:` 주석만 있어 "계약 없음 → auto-verified"로 카운트됨.
이제 실제 Z3 검증으로 전환됐지만 여전히 verified → 총 카운트 변동 없음.
Human 모드에서 `✓ pre verified` 확인으로 실제 검증 동작 확인.

## Reflection

- **Scope fit**: 100%
- **P0 예외 최소 패치 원칙**: 9개 targeted change (~100줄), 주변 코드 정리 없음
- **String SMT 설계**: `(str.len var)` + `"string literal"` + `ALL` logic — Z3 String theory 완전 지원
- **Track B 계약 미비**: `method.len() > 0` 외 더 강한 invariant (예: "ret starts with 'bmb_'")는 Z3 String theory로도 복잡 — 추후 개선 가능하나 현재 M7 범위 초과
- **Roadmap impact**: M7 ✅ COMPLETE

## Carry-Forward

- **Actionable**: 없음 (M7 완료)
- **Structural Improvement Proposals**:
  1. String 메서드 `contains`, `starts_with`, `ends_with` SMT 지원 — 더 강한 post-condition 가능
  2. untracked golden tests 5개 처리 (`tests/golden/test_golden_*.bmb`)
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M7 → ✅ COMPLETE 마킹
- **Next Recommendation**: 다음 세션 — untracked golden tests 처리 + M8 계획 수립
