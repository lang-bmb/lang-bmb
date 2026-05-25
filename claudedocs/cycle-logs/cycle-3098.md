# Cycle 3098: Track B 계약 추가 — 파서/렉서 skip_/find_/scan_ 배치
Date: 2026-05-25

## Re-plan
계획 유효. Cycle 3097 이어서 더 많은 P1 함수 계약 추가.

## Scope & Implementation

**skip_* 함수 14개** (`(src: String, pos: i64) -> i64` 패턴):
`skip_fn_type_param_list`, `skip_one_type_tok`, `skip_fn_type`, `skip_generic_params`,
`skip_optional_type_args`, `skip_contracts`, `skip_where_clause`, `skip_nullable`,
`skip_extern_fn`, `skip_struct_decl`, `skip_to_rbrace`, `skip_to_semi`,
`skip_brace_block`, `skip_field_type` → 모두 `pre pos >= 0`, `post it >= 0`

**find_/low_ 함수 7개**:
- `find_substr_pos`, `find_char_pos` → `pre pos >= 0`, `post it >= -1`
- `low_find_op_end`, `low_find_close_paren`, `find_angle_end`, `find_number_end`, `find_digits_end` → `pre pos >= 0`, `post it >= 0`

**scan_/skip_ 복합 파라미터 3개**:
- `skip_nested_braces(src, pos, depth)` → `pre pos >= 0`, `post it >= 0`
- `scan_structs(src, pos, rsb)` → `pre pos >= 0`, `post it >= 0`
- `scan_enums(src, pos, rsb)` → `pre pos >= 0`, `post it >= 0`

**Python regex 배치 패치** 사용으로 일관된 패턴 적용.

**합계**: 24개 계약 추가. 1452 → 1428 미계약 함수.

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3237 warnings, 0 errors)
- `bmb verify --list-uncontracted`: 1428 (−24) ✅

## Reflection

- Scope fit: 100%
- `low_starts_with_at` → `bool` 반환, 계약 추가 불필요 (trivially true)
- `skip_array_type_tokens`, `skip_tuple_type_tokens` → `String` 반환, `post it >= 0` 불가
- 배치 패치 방법이 효율적 — 동일 패턴 함수에 일관성 있게 적용

## Carry-Forward

- Actionable: Cycle 3099 — 추가 Track B 계약 (more P1, lexer scan_ functions)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: 렉서 스캔 함수, MIR 관련 함수 계약 추가
