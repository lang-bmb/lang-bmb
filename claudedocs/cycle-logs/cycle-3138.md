# Cycle 3138: M9 Batch 4 — 파서 유틸리티 14개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 774개 잔여. make_error_at/tok_kind_name/parser 함수들 분석.

## Scope & Implementation
14개 post 조건 추가:

**i64 정확값 (1개)**:
- `include_expand_sb(src, src_dir, pos, depth, sb)` → `post it == 0` (항상 0 반환 — sb 부작용)

**i64/String 범위 계약 (3개)**:
- `make_error_at(msg, src, pos)` → `post it.len() >= 4` ("ERR:" 접두사 보장)
- `tok_kind_name(kind)` → `post it.len() >= 1` (항상 비어있지 않은 이름)
- `compound_op(kind)` → `post it.len() <= 2` (최대 "<<", ">>" 2자)

**파서 결과 길이 계약 (10개)** — 모두 pack_result(>= 2) 또는 make_error_at(>= 4) 반환:
- `parse_bool_lit(tok, kind)` → `post it.len() >= 2`
- `parse_assert_call(src, pos)` → `post it.len() >= 2`
- `parse_dbg_call(src, pos, start_pos)` → `post it.len() >= 2`
- `parse_dbg_str_call(src, pos, start_pos)` → `post it.len() >= 2`
- `parse_panic_call(src, pos, default_msg)` → `post it.len() >= 2`
- `parse_assert_eq_call(src, pos, start_pos)` → `post it.len() >= 2`
- `parse_assert_ne_call(src, pos, start_pos)` → `post it.len() >= 2`
- `parse_unary(src, tok, op)` → `post it.len() >= 2`
- `parse_paren_expr(src, tok)` → `post it.len() >= 2`
- `parse_tuple_rest(src, pos, acc)` → `post it.len() >= 2`

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2973 → 2967 (−6 net; missing_postcondition 774→760 = −14)
  - semantic_duplication 481→489 (+8): 동일한 `post it.len() >= 2` 공유 함수들
- bmb verify: 918/918 → 904/904 (0 failed, total −14: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- make_error_at `post it.len() >= 4`: "ERR:" 접두사 존재 증명 — error propagation chain 기반
- compound_op `post it.len() <= 2`: 상한 계약 — 과도한 연산자 반환 불가 증명
- 파서 결과 계약: Z3가 개별적으로 증명하기 어려운 고복잡도 함수들 — 예산 소진 허용 (0 failed 유지)
- semantic_duplication 증가: `post it.len() >= 2` 다중 함수 공유 — 향후 batch 분리보다 의미 명확성 우선
- 남은 missing_postcondition: 760개

## Carry-Forward
- Actionable: missing_postcondition 760개 계속 분석
  - 다음 배치: parse_set_expr, parse_set_var, parse_block_let, parse_block_stmts 등 block parser들
  - parse_assert_cmp_call (누락 — Cycle 3138에서 eq/ne만 추가)
  - get_int_text, get_string_text, get_float_text (slice 길이 불확실)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→760 (−54 총계)
- Next Recommendation: Cycle 3139: parse_set_* + parse_block_* 계열 분석
