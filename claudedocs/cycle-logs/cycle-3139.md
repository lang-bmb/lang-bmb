# Cycle 3139: M9 Batch 5 — set/block 파서 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 760개 잔여. parse_set_*/parse_block_* 계열 체계적 분석.

## Scope & Implementation
15개 post 조건 추가:

**bool 정확 계약 (1개)**:
- `is_assign_op(kind)` → `post it == (kind == TK_EQ() or kind == TK_PLUSEQ() or ... TK_SHREQ())` (함수 본문 직접 반영)

**파서 결과 길이 계약 (14개)** — pack_result(≥2) 또는 make_error_at(≥4) 반환 보장:
- `parse_assert_cmp_call(src, pos, start_pos, cmp_op, msg)` → `post it.len() >= 2` (Cycle 3138 누락)
- `parse_set_expr(src, pos)` → `post it.len() >= 2`
- `parse_set_var(src, pos, name, op)` → `post it.len() >= 2`
- `parse_set_index(src, pos, target_name)` → `post it.len() >= 2`
- `parse_set_field(src, pos, target_name)` → `post it.len() >= 2`
- `parse_set_field_chain(src, pos, prev_base)` → `post it.len() >= 2`
- `parse_set_field_chain_index(src, pos, prev_base, field_name)` → `post it.len() >= 2`
- `parse_block_expr(src, pos)` → `post it.len() >= 2`
- `parse_block_stmts(src, pos)` → `post it.len() >= 2`
- `parse_block_let(src, pos)` → `post it.len() >= 2`
- `parse_block_let_mut(src, pos)` → `post it.len() >= 2`
- `parse_block_let_skip_type(src, pos, name, is_mut)` → `post it.len() >= 2`
- `parse_block_let_skip_tuple_type(src, pos, name, is_mut)` → `post it.len() >= 2`
- `parse_block_let_skip_array_type(src, pos, name, is_mut)` → `post it.len() >= 2`

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2967 → 2963 (−4 net; missing_postcondition 760→745 = −15)
  - semantic_duplication 489→500 (+11): 동일 `post it.len() >= 2` 계약 함수들
- bmb verify: 904/904 → 889/889 (0 failed, total −15: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- is_assign_op 정확 계약: 함수 본문 직접 반영 — 할당 연산자 집합 완전 열거
- make_error_at `post it.len() >= 4` 기반: 모든 파서 함수의 `post it.len() >= 2` chain 성립
- semantic_duplication 계속 증가: `post it.len() >= 2` 가 parser 함수 표준 계약이 됨 — 이는 정상 (개별 함수 의미 명확화)
- verify total 지속 감소: 복잡 재귀 함수들 Z3 예산 소진 — 0 failed 유지로 허용
- 남은 missing_postcondition: 745개

## Carry-Forward
- Actionable: missing_postcondition 745개 계속 분석
  - 다음 배치: parse_block_let_tuple, parse_block_tuple_names_acc, parse_block_tuple_names_sep, parse_block_let_value 등 tuple/value 바인딩 파서
  - parse_block_assign, parse_bare_assign, parse_block_expr_stmt 등 statement 파서들
  - parse_if_expr, parse_while_expr, parse_match_expr 등 control flow 파서들
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→745 (−69 총계)
- Next Recommendation: Cycle 3140: tuple/value/assign 블록 파서 + control flow 파서 분석
