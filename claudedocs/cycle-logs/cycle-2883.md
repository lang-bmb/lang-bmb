# Cycle 2883: to_string(bool) native 포팅
Date: 2026-05-15

## Re-plan
Carry-Forward (Cycle 2882): 없음. Cycle 2882 Next: to_string(bool) native 포팅 또는 format() 조사.
to_string(bool) → bmb_bool_to_string C 함수 추가 + lower.rs Bool 분기 완성. 단순 패턴 매칭.
범위: to_string(bool) native 포팅.

## Scope & Implementation

### STEP 1 진단:
- Cycle 2881에서 to_string 특수화를 구현했으나, Bool 분기가 `None`으로 처리됨
  - 증상: `to_string(true)` → native에서 `@to_string` undefined 오류
  - 원인: `Operand::Constant(Constant::Bool(_)) => None` → specialized = None → 미분기
- C 런타임에 `bmb_bool_to_string` 함수 부재

### STEP 2 수정:
**bmb_runtime.c** — `bmb_bool_to_string(int8_t b)` 추가:
```c
BmbString* bmb_bool_to_string(int8_t b) {
    return b ? bmb_string_from_cstr("true") : bmb_string_from_cstr("false");
}
```
- LLVM i1 타입은 C의 int8_t로 매핑됨 (ABI 호환)

**lower.rs** — Bool 분기 활성화:
- `Operand::Constant(Constant::Bool(_))` → `Some(MirType::Bool)`
- `Some(MirType::Bool)` → `Some("bmb_bool_to_string")`

**llvm_text.rs** — 선언 + 반환형:
- `declare nonnull ptr @bmb_bool_to_string(i1) nocallback nounwind nosync willreturn`
- `infer_call_return_type`: `"bmb_bool_to_string" => "ptr"`

### 검증:
- to_string(true) → "true": interp ✅, native ✅
- to_string(false) → "false": interp ✅, native ✅
- cargo test: 6249 PASS (0 FAIL) ✅

### bmb_reference.md 수정:
- line 919: `bool` 제외 문구 → 전체 포함으로 수정
- line 928: interpreter-only 목록에서 `to_string` (bool args) 항목 제거

## Verification & Defect Resolution
- to_string(true): native "true" ✅
- to_string(false): native "false" ✅
- cargo test: 6249 PASS (0 FAIL) ✅

## Reflection
- Scope fit: to_string(bool) native 포팅 완성. to_string은 이제 i64/f64/String/bool 모든 타입에서 native 지원.
- Latent: to_string(bool) 구현 시 LLVM i1 → C int8_t ABI 매핑이 예상대로 작동.
- Philosophy drift: 없음.
- Roadmap impact: to_string 전체 native 완성으로 이후 string interpolation native 포팅 가능성 증가.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. **struct pass-by-value HUMAN-decision** — 스펙 침묵, 의미론 결정 필요
  2. **for-in-vec native 포팅** — 현재 channel recv 방식으로 MIR 처리됨
  3. **format() native 포팅** — varargs C 함수 + 템플릿 치환 로직 필요
  4. **string interpolation native 포팅** — format()에 의존
  5. **runtime_param_type 장기 해결** — types/mod.rs 시그니처 직접 참조
- Pending Human Decisions:
  - struct parameter semantics (값 vs 참조)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2884 — for-in-vec native 포팅 조사 또는 interpreter-only 잔여 항목 audit
