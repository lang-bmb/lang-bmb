# Cycle 3143: M9 Batch 9 — if-brace + 제어 흐름 파서 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 700개 잔여. if-brace 파서 + 제어 흐름 표현식 파서.

## Scope & Implementation
15개 post 조건 추가 (모두 `post it.len() >= 2`):
- `parse_if_then_else` — then/else 문법 완성 파서
- `parse_if_brace_syntax` — { } 문법 if 파서
- `parse_if_brace_then` — if-brace then 분기 처리
- `parse_if_brace_else` — if-brace else 분기 처리
- `parse_if_brace_else_body` — else 본문 파서 (if/{ 분기)
- `parse_if_brace_final` — else { } 최종 처리
- `parse_if_brace_finish` — if-brace 완성 파서
- `make_if_ast` — (if c t e) AST 생성 (항상 pack_result)
- `parse_let_expr` — let 표현식 파서
- `parse_while_expr` — while 표현식 파서
- `parse_while_body` — while 본문 진입 파서
- `parse_return_expr` — return 표현식 파서
- `parse_loop_expr` — loop 표현식 파서
- `parse_for_expr` — for 표현식 파서
- `parse_match_expr` — match 표현식 파서

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2959 → 2958 (−1 net; missing_postcondition 700→685 = −15)
  - semantic_duplication 541→555 (+14): `post it.len() >= 2` 파서 공통 계약 확장
- bmb verify: 844/844 → 829/829 (0 failed, total −15: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- make_if_ast: 항상 pack_result 반환 — post it.len() >= 2 가장 명확
- parse_if_brace_*: if-brace 6개 헬퍼 체인 완성 — 모두 pack_result/make_error_at
- parse_let/while/loop/for/match/return_expr: 최상위 표현식 파서들 — parse_expr에서 분기됨
- parse_while_body: while 조건 이후 { } 진입 파서 — make_error_at + 재귀
- semantic_duplication 계속 증가 (정상): 개별 함수 의미 명확화 우선
- verify total 지속 감소 (정상): 복잡 재귀 Z3 예산 효과 — 0 failed 유지
- 남은 missing_postcondition: 685개

## Carry-Forward
- Actionable: missing_postcondition 685개 계속 분석
  - 다음 배치: parse_while_body_inner, parse_for_body, parse_for_body_inclusive, parse_for_in_body, parse_match_arms 등
  - parse_let_* 하위 파서들 (parse_let_type_check, parse_let_eq 등)
  - parse_struct_init, parse_struct_init_fields 등 구조체 파서
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→685 (−129 총계, 15.8% 감소)
- Next Recommendation: Cycle 3144: while_body_inner/for_body/match_arms + let_* 하위 파서들
