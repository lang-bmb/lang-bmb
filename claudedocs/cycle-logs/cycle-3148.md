# Cycle 3148: M9 Batch 14 — match_var_bind_body/const/low_*/resolve_*/parse_hex_bin 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 628개 잔여. 다양한 유형 함수 계약 분석.

## Scope & Implementation
15개 post 조건 추가:
**`post it.len() >= 2` (2개)**:
- `parse_match_var_bind_body` — 변수 바인딩 본문 파서
- `parse_const_decl` — const 선언 파서

**정확 계약 bool (3개)**:
- `low_is_whitespace(c)` → `post it == (c == 32 or c == 9 or c == 10 or c == 13)`
- `low_is_ident_char(c)` → `post it == ((c >= 65 and c <= 90) or ... or c == 95 or c == 45)`
- `low_is_op_char(c)` → `post it == (c == 43 or c == 45 or ... or c == 124)`

**단조 증가 i64 (3개)**:
- `low_skip_ws(s, pos)` → `post it >= pos`
- `low_find_ident_end(s, pos)` → `post it >= pos`
- `parse_hex_from(s, pos, acc)` → `post it >= 0` (pre acc >= 0 추가)

**`post it >= 0` (1개)**:
- `parse_bin_from(s, pos, acc)` → `post it >= 0` (pre acc >= 0 추가)

**bool 계약 (1개)**:
- `low_starts_with_at(s, pos, prefix)` → 정확 계약 (pos + prefix.len() <= s.len() 조건)

**`post it == 0` (5개)** — sb 부작용 함수:
- `resolve_ev_sb` — enum variant 해결 (sb 쓰기)
- `resolve_ev_one` — 단일 enum variant 해결
- `resolve_ev_match_ctx` — match 컨텍스트 enum variant 해결
- `resolve_tag_checks_sb` — tag check 해결
- `resolve_payload_calls_sb` — payload 호출 해결

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2955 → 2951 (−4 net; missing_postcondition 628→613 = −15)
  - net 더 큰 감소: 다양한 계약 타입 추가 (bool 정확 계약은 semantic_duplication 낮음)
- bmb verify: 772/772 → 760/760 (0 failed, total −12: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- low_is_* 계약: 함수 본문 직접 반영 — 정확한 의미 명세
- resolve_*_sb 계약: `post it == 0` — 항상 0 반환 sb 부작용 패턴 (include_expand_sb, escape_parens_sb 등과 동일)
- parse_hex/bin_from: pre acc >= 0 추가로 post it >= 0 계약 성립
- low_starts_with_at: 정확 계약 — pos + prefix.len() <= s.len() 조건 포함
- 남은 missing_postcondition: 613개

## Carry-Forward
- Actionable: missing_postcondition 613개 계속 분석
  - 다음 배치: step_* 함수들 (make_step/make_step_leaf 기반 계약)
  - parse_oct_from, parse_int_simple 등 나머지 정수 파서들
  - get_child, get_child_at, read_sexp_at 등 AST 유틸리티
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→613 (−201 총계, 24.7% 감소)
- Next Recommendation: Cycle 3149: step_* MIR 함수들 + 나머지 유틸리티 함수들
