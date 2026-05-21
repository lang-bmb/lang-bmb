# Cycle 2873: str_trim_left/right/reverse + int_to_hex/bin native 포팅
Date: 2026-05-15

## Re-plan
Plan valid. Cycle 2872에서 vec 9종 완료. 이번 사이클: C 런타임에 없는 함수들을 추가 후 포팅.

## Scope & Implementation

### C 런타임 추가 (`bmb/runtime/bmb_runtime.c`):
- `bmb_string_trim_left(BmbString*)` — 좌측 공백 제거 (L225 이후 추가)
- `bmb_string_trim_right(BmbString*)` — 우측 공백 제거
- `bmb_int_to_hex(int64_t)` — 정수 → 소문자 16진수 문자열 (L970 이후 추가)
- `bmb_int_to_bin(int64_t)` — 정수 → 이진수 문자열 (최상위 비트부터)

### llvm_text.rs 추가:
- IR 선언: 5종 (trim_left/right, reverse, int_to_hex/bin)
- name mapping: str_trim_left/right/reverse + int_to_hex/bin
- infer_call_return_type: 5종 → ptr 반환 추가

참고: `bmb_string_reverse` 는 C 런타임에 이미 존재 (Cycle 2868에서 추가됨)

## Verification & Defect Resolution
- `tests/native_str2_builtins.bmb`: `bmb run` = `bmb build` = 10행 동일 ✅
  - str_trim_left("  hello") → "hello" ✅
  - str_trim_right("hello  ") → "hello" ✅
  - str_reverse("hello") → "olleh" ✅
  - int_to_hex(255) → "ff" ✅ (소문자, 0x 접두사 없음)
  - int_to_hex(0) → "0" ✅
  - int_to_bin(10) → "1010" ✅
  - int_to_bin(0) → "0" ✅
- `cargo build --release`: 0 errors ✅

## Reflection
- Scope fit: ✅ 5종 네이티브 포팅 완료
- int_to_hex 음수 처리: Rust `format!("{:x}", n)`과 동일한 두의보수 무부호 해석 사용
- int_to_bin 구현: 최상위 비트 자동 탐색으로 선행 0 제거 (Rust `format!("{:b}", n)` 동일 동작)

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2874 — str_substr native 포팅 (bmb_string_substr C 래퍼 추가) + str_count/pad_left/pad_right
