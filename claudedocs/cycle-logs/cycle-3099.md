# Cycle 3099: Track B 계약 추가 — find_/skip_/keyword_ 배치
Date: 2026-05-25

## Re-plan
계획 유효. Cycle 3098 이어서 더 많은 P1 함수 계약 추가.

## Scope & Implementation

**find_* P1 함수 28개 배치 처리**:
- `find_work_sep`, `find_field_sep`, `find_single_pipe`, `find_var_end`, `find_pipe_skip_quotes`,
  `find_eq`, `find_arrow`, `find_double_pipe`, `find_string_in_mir`, `find_quote_in_mir`,
  `find_tab_in_list`, `find_fn_start`, `find_arrow_in_mir`, `find_fn_end`, `find_fn_name_end`
  → `pre pos >= 0`, `post it >= -1`
- `find_char_at`, `find_last_char_scan`, `find_pipe_in_range`, `find_at_name_end_in`,
  `find_char_in_range`, `find_byte_in_range`, `find_string_index_scan`, `find_char_from`,
  `find_pattern_at`, `find_pattern_at_slow`, `find_field_index_or_neg`, `find_alias_entry_noa`,
  `find_var_use_wb`
  → 각 파라미터에 맞는 pre + `post it >= -1`

**skip_* 추가 3개**:
- `skip_quoted_string`, `skip_spaces`, `skip_nullable_idx` → `pre pos >= 0`, `post it >= 0`

**keyword_* 9개**:
- `keyword_or_ident`, `keyword_len2`~`keyword_len10` → `pre endpos >= 0`, `post it >= 0`

**scan_mir_for_free_vars**:
- String 반환, `pre pos >= 0` only

**건너뛴 것**:
- `find_first_balanced`, `find_rest_balanced`, `find_mir_annotation_at` → String 반환
- `skip_annotation`, `skip_array_type_tokens`, `skip_tuple_type_tokens` → String 반환

**합계**: 41개 계약 추가. 1428 → 1387 미계약 함수.

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3234 warnings, 0 errors)
- `bmb verify --list-uncontracted`: 1387 (−41) ✅

## Reflection

- Scope fit: 100%
- Python regex 배치 패치 방법 효율적 — 동일 패턴 함수군 일괄 처리
- find_ 함수 계약 패턴 확립: `pre pos >= 0`, `post it >= -1` (position or not-found)

## Carry-Forward

- Actionable: Cycle 3100 — count_/get_/format_ 함수 계약 추가
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: count_ 6개 + get_ 17개 중 i64 반환 함수 계약 추가
