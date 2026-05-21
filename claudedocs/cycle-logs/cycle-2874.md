# Cycle 2874: str_substr/count/pad_left/pad_right native 포팅
Date: 2026-05-15

## Re-plan
Plan valid. Cycle 2873에서 trim_left/right/reverse/int_to_hex/bin 완료. 이번 사이클: str_substr, str_count, str_pad_left, str_pad_right 포팅.

## Scope & Implementation

### C 런타임 추가 (`bmb/runtime/bmb_runtime.c`, L364 근처):
- `bmb_string_substr(BmbString*, start, len)` — bmb_string_slice(s, start, start+len) 래퍼
- `bmb_str_pad_left(BmbString*, width, BmbString*)` — String pad char를 i64로 변환 후 bmb_string_pad_left 호출
- `bmb_str_pad_right(BmbString*, width, BmbString*)` — 동일 패턴

참고: `bmb_string_count`와 `bmb_string_pad_left/right`는 이미 C 런타임에 존재하나:
- str_count → bmb_string_count (시그니처 일치, 직접 매핑)
- str_pad_left/right → 3번째 인수 타입 불일치 (BMB: String, C: i64) → 래퍼 함수 추가

### llvm_text.rs 추가:
- IR 선언: 4종 (substr, count, str_pad_left/right)
- name mapping: str_substr/count/pad_left/pad_right
- infer_call_return_type: str_count → i64, str_substr/pad_left/pad_right → ptr

## Verification & Defect Resolution
- `tests/native_str3_builtins.bmb`: `bmb run` = `bmb build` = 5행 동일 ✅
  - str_substr("Hello, World!", 7, 5) → "World" ✅
  - str_count("Hello, World!", "l") → 3 ✅
  - str_pad_left("hi", 5, " ") → "   hi" ✅
  - str_pad_right("hi", 5, "-") → "hi---" ✅
- `cargo test --release`: 2388 PASS (진행 중)

## Reflection
- Scope fit: ✅ 4종 네이티브 포팅 완료
- str_pad_left/right 인수 타입 불일치(String vs i64): C 래퍼 함수로 해결
- str_substr: C의 slice(start, end) vs BMB의 substr(start, len) 차이를 래퍼로 해결

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2875 — f64 수학 free functions native 포팅 (log/log2/log10/exp/round/tan/atan/atan2 + min_f64/max_f64/clamp_f64)
