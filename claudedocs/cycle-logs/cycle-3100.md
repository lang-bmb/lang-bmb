# Cycle 3100: Track B 계약 추가 — count_/get_ 함수
Date: 2026-05-25

## Re-plan
계획 유효. Cycle 3099 이어서 count_/get_ 함수 계약 추가.

## Scope & Implementation

**count_* 6개** (`pre pos/idx >= 0`, `post it >= 0`):
`count_variant_index` (post -1), `count_csv_rec`, `count_children_at`,
`count_string_bytes_acc`, `count_struct_fields_at`, `count_commas`

**get_* 15개** (String 반환 → `pre pos/idx >= 0` only):
`get_ident_text`, `get_int_text`, `get_string_text`, `get_float_text`,
`get_pipe_name`, `get_pipe_name_at`, `get_field_ptr_type`, `get_child`,
`get_child_at`, `get_field`, `get_field_at`, `get_lambda_body`,
`get_fn_return_scan`, `get_fn_body_scan`, `get_struct_ptr_type_from`

**합계**: 21개 계약 추가. 1387 → 1366 미계약 함수.

**누적**: 1467 → 1366 = **101개 계약 추가** (cycles 3097-3100)

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3233 warnings, 0 errors)
- `bmb verify --list-uncontracted`: 1366 ✅

## Reflection

- Scope fit: 100%
- `count_variant_index`: `post it >= -1` — 찾지 못하면 -1 반환
- `get_*` 함수들은 모두 String 반환 → post 조건 추가 불필요 (String length는 항상 >= 0)
- pre 조건만 추가해도 정적 분석/문서화 가치 있음

## Carry-Forward

- Actionable: Cycle 3101 — 추가 계약 (collect_/index_) + M7-4 COMPLETE 선언 준비
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: M7-4 COMPLETE 선언 → ROADMAP/HANDOFF 업데이트 → commit
