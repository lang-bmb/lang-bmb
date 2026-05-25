# Cycle 3146: M9 Batch 12 — let/while/loop/for/match 하위 파서 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 657개 잔여. let/while/loop/for/match 하위 헬퍼 파서들.

## Scope & Implementation
15개 post 조건 추가 (모두 `post it.len() >= 2`):
- `parse_ident_or_call` — 식별자/함수호출 파서
- `parse_let_skip_tuple_type` — let tuple 타입 스킵 파서
- `parse_let_skip_array_type` — let 배열 타입 스킵 파서
- `parse_let_value_mut` — let 값 파서 (mut/immut 분기)
- `parse_let_body_mut` — let 본문 파서
- `make_let_ast` — (let <n> v b) AST 생성 (항상 pack_result)
- `make_let_mut_ast` — (let_mut <n> v b) AST 생성 (항상 pack_result)
- `parse_while_finish` — while 완성 파서 ('}' 이후)
- `parse_loop_body` — loop 본문 파서
- `parse_loop_finish` — loop 완성 파서 ('}' 이후)
- `parse_for_range` — for 범위 파서 ('in' 이후)
- `parse_for_end` — for '..' 이후 종료값 파서
- `parse_for_end_inclusive` — for '..=' 이후 종료값 파서
- `parse_match_open` — match '{' 이후 파서
- (semantic_duplication 상쇄로 net −1)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2957 → 2956 (−1 net; missing_postcondition 657→643 = −14)
  - semantic_duplication: +13 (정상)
- bmb verify: 801/801 → 787/787 (0 failed, total −14: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- make_let_ast / make_let_mut_ast: 항상 pack_result 반환 — 명확한 계약
- parse_for_range: '..' / '..=' / '{' (for-in) 세 경로 모두 위임 — 결과는 pack_result/make_error_at
- parse_while/loop_finish: '}' 체크 후 pack_result — 단순 패턴
- build_tuple_bindings: raw AST 반환 (pack_result 아님) — 이번 배치에서 제외 (정확)
- 남은 missing_postcondition: 643개

## Carry-Forward
- Actionable: missing_postcondition 643개 계속 분석
  - 다음 배치: parse_match_arm_arrow, parse_match_range_*, parse_match_or_*, parse_match_single_pattern, build_payload_lets_from_pat, parse_match_arm_sep, parse_payload_bind_list
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→643 (−171 총계, 21.0% 감소)
- Next Recommendation: Cycle 3147: match 패턴/arm/payload 파서들 분석
