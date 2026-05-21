# Cycle 2881: to_string<T> native 포팅 (i64/f64/String)
Date: 2026-05-15

## Re-plan
Carry-Forward: 없음. Cycle 2880 Next: to_string 부분 native 포팅 또는 interpreter-only 재검증.
to_string(i64/f64/String) native 포팅 채택.

## Scope & Implementation

### 진단 (STEP 1):
- 현재 IR: `call i64 @to_string(i64 42)` → @to_string 미정의, 반환 타입도 i64(오류)
- 원인: lower.rs가 `to_string`을 알 수 없는 함수 → 기본 `MirType::I64`로 처리
- BMB 런타임에는 `bmb_int_to_string(i64)` + `bmb_f64_to_string(double)` 이미 존재
- `to_string(String)` → identity (입력이 이미 String)

### 수정 (STEP 2):
1. **lower.rs**: `to_string` 호출 감지 후 arg MirType 기반 재작성 (line 1366 추가):
   - `I64/I32` → `int_to_string` (이미 native에서 지원)
   - `F64` → `bmb_f64_to_string`
   - `String` → identity (arg_op 직접 반환)
   - `Bool` + 기타 → 기존 경로 fall-through
2. **llvm_text.rs**: `bmb_f64_to_string` 선언 추가 + return_type_of_fn `"ptr"` 등록

### 검증:
- `to_string(42)` → interp=2(str_len), native=2 ✅
- `to_string(-7)` → interp=2, native=2 ✅
- `to_string("hi")` → interp=2, native=2 ✅
- `to_string(0)` → interp=1, native=1 ✅
- `tests/native_to_string.bmb` 총합: 7 interp=7, native=7 ✅

### bmb_reference.md 수정:
- `to_string(x)` 항목: native-supported for i64/f64/String, bool still interpreter-only
- interpreter-only 목록에서 to_string 제거 (bool case 제외)

## Verification & Defect Resolution
- interpreter: 7 ✅
- native: 7 ✅
- cargo test: 6249 PASS (0 FAIL) ✅

## Reflection
- Scope fit: to_string 포팅 성공 — lower.rs에서 MirType 기반 재작성이 깔끔한 접근.
- Bool 케이스 제외: Bool→String 변환은 별도 C runtime 함수(`bmb_bool_to_string`) 필요. 현재 미구현. 영향 낮음 (bool→i64 변환은 있지만 String은 드물다).
- to_string identity(String→String): `Constant::String`도 MirType::String으로 매핑해 직접 반환. 성능 최적 (call 없음).
- Philosophy drift: 없음.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. **struct pass-by-value HUMAN-decision** — 스펙 침묵, 의미론 결정 필요
  2. **to_string(bool) native 포팅** — bmb_bool_to_string("true"/"false") C 함수 추가 필요
  3. **runtime_param_type 장기 해결** — types/mod.rs 시그니처 직접 참조
- Pending Human Decisions:
  - struct parameter semantics (값 vs 참조)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2882 — bmb_reference.md 나머지 interpreter-only 항목 검증 (Cycle 2879 패턴 탐색) 또는 for-in-vec native 포팅 조사
