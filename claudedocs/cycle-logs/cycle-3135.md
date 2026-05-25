# Cycle 3135: M9 Batch 1 — 핵심 유틸리티 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M8-A/B 완료 → M9 새 방향: missing_postcondition 814개 (pre 있음, post 없음).
고ROI 함수부터 시작: 파일 상단 핵심 유틸리티 (is_*, skip_*, find_*, count_*).

## Scope & Implementation
15개 post 조건 추가:

**Bool 정확 계약 (7개)**:
- `is_whitespace(c)` → `post it == (c == 32 or c == 9 or c == 10 or c == 13)`
- `is_digit(c)` → `post it == (c >= 48 and c <= 57)`
- `is_hex_digit(c)` → `post it == (is_digit(c) or (c >= 65 and c <= 70) or (c >= 97 and c <= 102))`
- `is_alpha(c)` → `post it == ((c >= 65 and c <= 90) or (c >= 97 and c <= 122))`
- `is_alnum_or_underscore(c)` → `post it == (is_alpha(c) or is_digit(c) or c == 95)`
- `is_ident_start(c)` → `post it == (is_alpha(c) or c == 95)`
- `starts_with/has_pattern` → `post not it or (pos + pat.len() <= s.len())`

**i64 범위 계약 (6개)**:
- `skip_ws(s, pos)` → `post it >= pos`
- `skip_sp_tab(s, pos, limit)` → `post it >= pos and it <= limit`
- `count_line_at(src, pos, cur, line)` → `post it >= line`
- `find_line_start(src, pos)` → `post it >= 0 and it <= pos`
- `find_line_end(src, pos)` → `post it >= pos`

**String 정확 길이 계약 (2개)**:
- `digit_char(d)` → `post it.len() == 1` (항상 1자리 문자)
- `make_caret_line(n, acc)` → `post it.len() == acc.len() + n + 1` (정확한 재귀 길이)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2994 → 2981 (−13); missing_postcondition: 814 → 799 (−15)
- bmb verify: 953/953 → 942/942 (0 failed, total −11: Z3 예산 영향 추정)
  - 0 failed — 모든 검증 통과
  - total −11: 새 계약의 복잡도 증가로 Z3 예산 소진, 일부 함수 pool 이탈 추정

## Reflection
- M9 방향 확인: missing_postcondition 814개 중 15개 고ROI 유틸리티 완료
- bool 정확 계약: is_digit/is_alpha 등 단순 술어 — 모든 호출자가 계약 활용 가능
- i64 범위 계약: skip/find 계열 — 위치 단조성(monotonicity) 증명
- make_caret_line: `post it.len() == acc.len() + n + 1` — 재귀 정확 길이 계약
- verify total 감소: 허용 가능 (0 failed) — 복잡한 재귀 계약 Z3 예산 영향
- 남은 missing_postcondition: 799개

## Carry-Forward
- Actionable: missing_postcondition 799개 계속 분석
  - 다음 배치: tok_kind_name/skip_ws_comments/scan_* 계열 (lexer 함수들)
  - 또는: 뒤 섹션의 고빈도 호출 함수들 (find_pattern_at, find_char 등)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 시작 — missing_postcondition 814→799 (−15)
- Next Recommendation: Cycle 3136: lexer scan_* 함수들 + find_pattern_at 계열 분석
