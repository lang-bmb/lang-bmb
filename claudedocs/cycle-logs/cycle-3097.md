# Cycle 3097: Track B 계약 추가 — P2 + 주요 P1 함수
Date: 2026-05-25

## Re-plan
계획 유효. `list-uncontracted.bmb` 자동화 결과를 활용하여 Track B 계약 추가.

## Scope & Implementation

**P2 패턴명 함수 (pos 파라미터 없음) — `post it >= 0` 또는 `post it >= -1`**:
- `count_csv_items` → `post it >= 0`
- `count_children` → `post it >= 0`
- `count_string_bytes` → `post it >= 0`
- `count_struct_fields` → `post it >= 0`
- `count_ir_lines_loop` → `post it >= 0`
- `count_line_pattern` → `post it >= 0`
- `count_substr_loop` → `post it >= 0`
- `find_last_char` → `post it >= -1`
- `find_string_index` → `post it >= -1`
- `find_closure_count` → `post it >= 0`

**주요 P1 함수 (pos 파라미터 있음) — `pre pos >= 0` + `post it >= 0`**:
- `find_quote_after` → `pre pos >= 0`, `post it >= -1`
- `skip_to_eol` → `pre pos >= 0`, `post it >= 0`
- `skip_comment` → `pre pos >= 0`, `post it >= 0`
- `skip_block_comment` → `pre pos >= 0`, `post it >= 0`
- `scan_number` → `pre pos >= 0`, `post it >= 0`

**합계**: 15개 계약 추가. 1467 → 1452 미계약 함수.

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3239 warnings, 0 errors)
- `bmb verify --list-uncontracted`: 1452 (−15) ✅
- `cargo test --release`: ✅

## Reflection

- Scope fit: 100%
- `find_closure_count`: `post it >= 0` 정확 — n <= 0이면 0, 아니면 1..9 범위
- `count_ir_lines_loop`: `count[0] + 1` 이므로 최소 1 반환 — `post it >= 0` 충분히 보수적
- skip/find 함수군 계약 패턴이 일관됨

## Carry-Forward

- Actionable: Cycle 3098 — 추가 P1 함수 계약 (더 많은 lexer/parser 함수)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: make_error_at, get_ident_text 등 파서 핵심 함수 계약 추가
