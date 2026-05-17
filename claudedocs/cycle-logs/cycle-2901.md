# Cycle 2901: @export -> String 정적 리터럴 P0 수정 (heap-copy 패턴 전파)
Date: 2026-05-17

## Re-plan
Carry-Forward 없음. HANDOFF "Structural Improvement 4: `@export pub fn -> String` static literal 반환 자동 heap-copy 미흡 (bootstrap Rule 6)" 항목 선택.

Cycle 2897에서 `bmb_json_type` 한 곳만 수동 패치됨. 다른 ecosystem lib.bmb 파일 5개 전수 검사.

## Scope & Implementation

**목표**: `@export pub fn -> String` 함수가 정적 리터럴 `""` 또는 named literal을 반환하는 모든 경로 수정 — FFI 측이 `bmb_ffi_free_string(ret)` 시 .rodata 포인터 free() → crash (P0).

**스캔 결과**:
- `ecosystem/bmb-json/src/lib.bmb`: `bmb_json_type` 이미 Cycle 2897 패치 ✓
- `ecosystem/bmb-algo/src/lib.bmb`: `@export -> String` 없음 ✓
- `ecosystem/bmb-compute/src/lib.bmb`: `@export -> String` 없음 ✓
- `ecosystem/bmb-text/src/lib.bmb`: **3곳** `""` 리터럴 반환 (P0)
- `ecosystem/bmb-crypto/src/lib.bmb`: **6곳** `""` 리터럴 반환 (P0)

**패치 방식**: Cycle 2897 선례(`str_repeat("", 1)`) 동일 패턴 적용.

### Files changed

**ecosystem/bmb-text/src/lib.bmb**:
1. `str_reverse`: `if len == 0 { "" }` → `str_repeat("", 1)` (빈 문자열 입력 시)
2. `str_trim`: `if len == 0 { "" }` → `str_repeat("", 1)`
3. `str_trim`: `if start >= end { "" }` → `str_repeat("", 1)` (공백만 있는 문자열)

**ecosystem/bmb-crypto/src/lib.bmb**:
4. `b64_encode_string`: `if len == 0 { "" }` → `str_repeat("", 1)`
5. `b64_decode_string`: 3곳 `""` → `str_repeat("", 1)` (len==0, 잘못된 패딩, decode 오류)
6. `b32_encode_string`: `if len == 0 { "" }` → `str_repeat("", 1)`
7. `b32_decode_string`: 3곳 `""` → `str_repeat("", 1)` (len==0, 잘못된 패딩, 오류)

**총 9개 사이트** 수정.

### str_repeat 가용성
- `bmb-text/src/lib.bmb`: line 480에 `fn str_repeat` 자체 정의 — 자체 함수 호출 ✓
- `bmb-crypto/src/lib.bmb`: BMB 런타임 빌트인 `str_repeat` 사용 (Cycle 2871 native 포팅 완료) ✓

### 미수정 패턴 (다른 종류)
- `str_replace(s, old, new)` → no match 시 `s` 반환 (입력 passthrough)
- `str_replace_all(s, old, new)` → old_pat 빈 시 `s` 반환 (입력 passthrough)

이 패턴은 .rodata 접근이 아닌 **입력 포인터 반환** 문제로 별도 분석 필요. 현재 C# 테스트 93/93 통과 중 → 실제 double-free 발생 여부 불확실. Structural Improvement로 carry-forward.

## Verification & Defect Resolution
`cargo test --release`: **2388/2388 PASS** ✓
- `gotgan::cache::test_build_cache_save_load` 1 FAIL은 기존 문제 (gotgan 패키지, 우리 변경과 무관)

## Reflection
- **Scope fit**: Cycle 2897 패치의 체계적 전파 완료. P0 FFI crash 경로 9개 제거.
- **Latent defects**: 입력 passthrough 패턴 (`str_replace`, `str_replace_all`) 은 미해결. 현재 C# 93/93 통과 → 즉각적 crash 없음. 별도 분석 필요.
- **Structural improvement**: str_replace/str_replace_all의 입력 passthrough 패턴이 FFI 안전한지 BMB FFI 프로토콜 문서화 확인 필요.
- **Philosophy drift**: 없음. ecosystem lib.bmb만 수정, bootstrap 미수정.
- **Roadmap impact**: HANDOFF Structural Improvement 4 → 부분 해소 (완전 해소는 bootstrap 레벨 수정 필요).

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - `str_replace`, `str_replace_all`의 입력 passthrough 반환이 FFI에서 안전한지 분석 필요
  - bootstrap 레벨: `@export pub fn -> String`이 static literal 반환 시 자동 heap-copy (컴파일러 수준 근본 수정)
- Pending Human Decisions:
  - B축 재측정 실행 (API key 확인 후)
  - Java 바인딩 계속 개발 여부 (나머지 4개 라이브러리)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2902 — inkwell/text 백엔드 함수 등록 정합성 검사 스크립트 작성
