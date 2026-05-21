# Cycle 2877: bmb_reference 업데이트 + str_is_empty 추가
Date: 2026-05-15

## Re-plan
Plan valid. Carry-Forward 없음. HANDOFF 권장 우선순위 1: bmb_reference 업데이트 (interpreter-only 경고 해제) + str_is_empty 미등록 함수 추가.

## Scope & Implementation

### str_is_empty 추가 (interpreter + native):
- `bmb/src/types/mod.rs`: `str_is_empty(String) -> i64` 시그니처 등록
- `bmb/src/interp/eval.rs`: `builtin_str_is_empty` 함수 추가 + builtins map 등록
- `bmb/src/codegen/llvm_text.rs`: 이름 매핑 `"str_is_empty" => "bmb_string_is_empty"` + infer_call_return_type i64 섹션 추가
- C 런타임 `bmb_string_is_empty`와 IR 선언은 이미 존재 — 등록만으로 native 포팅 완료

### bmb_reference.md 업데이트:
- str_trim_left/right, int_to_hex/bin, str_replace/repeat, str_count/pad_left/pad_right, str_to_upper/lower, str_reverse, str_to_f64, read_f64: "(interp-only)" 제거
- popcount, min_f64/max_f64/clamp_f64, round/log/log2/log10/exp/tan/atan/atan2: "(interp-only)" 제거
- vec_sum/max/min/sort/contains/index_of/remove/reverse/fill: "interpreter-only" 제거
- str_is_empty 항목 추가
- Common Pitfalls 섹션: native/interpreter-only 구분 목록 명확화

## Verification & Defect Resolution
- interpreter: `bmb run tests/native_str_is_empty.bmb` → 1 ✅ (e1=1 for "", e2=e3=e4=0)
- native: `bmb build` → 1 ✅
- cargo test --release: 6249 PASS (3778+47+13+2388+23), 0 FAIL ✅

## Reflection
- Scope fit: ✅ 정확히 범위 내 — bmb_reference 업데이트 + str_is_empty 등록
- str_char_at: interpreter-only로 유지 (올바름 — String 반환 타입 처리 복잡도 미해결)
- Common Pitfalls 섹션이 더 명확해짐 — LLM이 native 사용 가능 함수를 정확히 파악 가능

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. 음수 리터럴 i32 narrow 근본 수정 (Cycle 2876에서 carry-forward)
  2. 필드 복합 할당 native 지원 (`set obj.field += e`)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2878 — 음수 리터럴 i32 narrow 근본 원인 추적 (Structural improvement → P0 버그 아직 남아있음)
