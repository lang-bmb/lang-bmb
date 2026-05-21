# Cycle 2871: str builtins native 포팅 (12종 → bmb_string_* / bmb_parse_int)
Date: 2026-05-15

## Re-plan
Plan valid. HANDOFF 권장 방향: interpreter-only builtins → native (LLVM codegen) 포팅.
Cycle 2871 scope: str 계열 12종 네이티브 지원.

## Scope & Implementation
`bmb/src/codegen/llvm_text.rs` 3곳 수정:

1. **IR 선언 추가** (L843): `bmb_parse_int` declare (str_to_int → bmb_parse_int)
2. **name mapping 추가** (L6319-): str_* 12종 → bmb_string_* / bmb_parse_int 매핑
3. **infer_call_return_type 추가**: i64 arm에 str_len/contains/starts_with/ends_with/find/to_int/bmb_parse_int, ptr arm에 str_trim/to_upper/to_lower/replace/repeat

포팅된 함수:
| BMB 함수 | C 런타임 함수 | 반환형 |
|----------|-------------|--------|
| str_len | bmb_string_len | i64 |
| str_contains | bmb_string_contains | i64 |
| str_starts_with | bmb_string_starts_with | i64 |
| str_ends_with | bmb_string_ends_with | i64 |
| str_find | bmb_string_index_of | i64 |
| str_to_int | bmb_parse_int | i64 |
| str_trim | bmb_string_trim | ptr |
| str_to_upper | bmb_string_to_upper | ptr |
| str_to_lower | bmb_string_to_lower | ptr |
| str_replace | bmb_string_replace | ptr |
| str_repeat | bmb_string_repeat | ptr |

제외 함수:
- `str_char_at`: types/mod.rs에서 `-> String` 반환이지만 bmb_string_char_at은 i64 반환 — 타입 불일치로 제외
- `str_is_empty`: types/mod.rs에 미등록 — 제외

## Verification & Defect Resolution
- `tests/native_str_builtins.bmb` 생성 → `bmb run` 출력 = `bmb build` 출력 = 13행 동일 ✅
- IR 확인: `call i64 @bmb_string_len`, `call ptr @bmb_string_trim` 등 정확 매핑 ✅
- `cargo build --release`: 0 errors ✅

## Reflection
- Scope fit: ✅ str 12종 네이티브 포팅 완료
- 기존 C 런타임 함수가 이미 선언됐고, LLVM IR 선언도 이미 존재했음. 누락된 것은 name mapping 뿐 → 최소 변경으로 최대 효과
- str_char_at 타입 불일치 발견: types/mod.rs는 `-> String`이지만 C 함수는 i64 반환. 별도 C 래퍼 필요 → Carry-Forward

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  - `str_char_at` native 지원: C 런타임에 `bmb_string_char_at_str(ptr, i64) -> ptr` 추가 필요
  - `str_is_empty` native 지원: types/mod.rs에 등록 + mapping 추가 필요
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2872 — vec aggregate builtins native 포팅 (vec_sum/min/max/contains/index_of/sort)
