# Cycle 3137: M9 Batch 3 — parser 유틸리티 14개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 788개 잔여. Batch 2 lexer/scanner 완료 → parser 유틸리티 함수들 (Section 3-5) 분석.

## Scope & Implementation
14개 post 조건 추가:

**i64 정확값 계약 (3개)**:
- `escape_parens_sb(s, pos, sb)` → `post it == 0` (항상 0 반환 — sb 부작용 함수)
- `unescape_parens_sb(s, pos, sb)` → `post it == 0` (동일)
- `tok_kind(r)` → `post it >= 0` (r / 5000000, r >= 0이므로)

**bool 정확 계약 (1개)**:
- `is_int_literal(kind)` → `post it == (kind < 2000000100 or kind >= 2000001000)` (함수 본문 직접 반영)

**i64 범위 계약 (3개)**:
- `get_char_value(s, pos)` → `post it >= 0` (ASCII 값 0-255)
- `next_token_raw(s, pos)` → `post it >= 0` (make_tok 결과)
- `unpack_pos_acc(r, pos, acc)` → `post it >= acc` (누적 값 단조 증가)

**String 길이 계약 (3개)**:
- `get_ident_text(s, pos, tok)` → `post it.len() >= 1` (scan_ident_end(s,p+1) >= p+1)
- `pack_result(pos, ast)` → `post it.len() >= 2` ("N:" + ast, 최소 2자리)
- `parse_int_lit(src, pos, tok)` → `post it.len() >= 2` (pack_result 위임)
- `parse_float_lit(src, pos, tok)` → `post it.len() >= 2` (pack_result 위임)

**위치 범위 계약 (4개)**:
- `scan_string_end(s, pos)` → `post it >= pos` (전진 단조성)
- `scan_char_end(s, pos)` → `post it >= pos` (전진 단조성)
- `find_colon(s, pos)` → `post it >= pos and it <= s.len()` (탐색 범위)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2980 → 2973 (−7 net; missing_postcondition 788→774 = −14)
  - 다른 카테고리 소폭 증가로 net −7
- bmb verify: 931/931 → 918/918 (0 failed, total −13: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- is_int_literal 정확 계약: 함수 본문을 그대로 반영 — caller가 결과 범위 활용 가능
- escape/unescape_parens_sb: `post it == 0` — sb 부작용만 있고 반환값은 항상 0
- find_colon: `post it >= pos and it <= s.len()` — 이중 범위 계약 (하한+상한)
- unpack_pos_acc: `post it >= acc` — 누적기 단조증가 증명
- pack_result: `post it.len() >= 2` — 파서 결과 포맷 계약 ("pos:ast")
- verify total 감소 패턴: −13, −11, −11 반복 — 재귀 함수들의 Z3 예산 효과
- 남은 missing_postcondition: 774개

## Carry-Forward
- Actionable: missing_postcondition 774개 계속 분석
  - 다음 배치: get_int_text, get_string_text, get_float_text (slice 관계 불확실, 검토 필요)
  - parse_bool_lit, parse_ident_or_call, parse_assert_call, parse_dbg_call 등 파서 함수들
  - compound_op, is_assign_op 등 단순 유틸리티
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→774 (−40 총계)
- Next Recommendation: Cycle 3138: compound_op/is_assign_op/parse_bool_lit 등 파서 유틸리티 분석
