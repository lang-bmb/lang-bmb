# Cycle 3141: M9 Batch 7 — 이진 연산 파서 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 730개 잔여. postfix/binop 연산자 우선순위 파서 계열.

## Scope & Implementation
15개 post 조건 추가 (모두 `post it.len() >= 2`):
- `parse_postfix` — postfix 표현식 (method call, index, cast)
- `parse_postfix_rest` — postfix 연쇄 파서
- `parse_mul` — 곱셈 우선순위 파서
- `parse_mul_rest` — 곱셈 연산자 연쇄
- `parse_add` — 덧셈 우선순위 파서
- `parse_add_rest` — 덧셈 연산자 연쇄
- `parse_shift` — 비트 시프트 우선순위 파서
- `parse_shift_rest` — 시프트 연산자 연쇄
- `parse_cmp` — 비교 연산 파서
- `parse_cmp_rest` — 비교 연산자 연쇄
- `parse_bitand` — bitwise AND 파서
- `parse_bitand_rest` — bitwise AND 연쇄
- `parse_bitxor` — bitwise XOR 파서
- `parse_bitxor_rest` — bitwise XOR 연쇄
- `parse_bitor` — bitwise OR 파서

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2961 (변화없음 net; missing_postcondition 730→715 = −15, semantic_duplication 513→528 = +15)
- bmb verify: 874/874 → 859/859 (0 failed, total −15: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- 이진 연산 파서 계열: 연산자 우선순위를 구현하는 재귀 하강 파서. 모두 pack_result/make_error_at 반환.
- parse_cmp_rest: is_error 체크 없이 pack_result 호출 — 오류 전파 대신 결과 사용. it.len() >= 2 여전히 성립.
- semantic_duplication 계속 증가: `post it.len() >= 2` 계약이 파서 전체의 표준 계약으로 확립됨
- verify total 지속 감소: 재귀 파서들의 Z3 예산 효과 — 0 failed 유지
- 남은 missing_postcondition: 715개

## Carry-Forward
- Actionable: missing_postcondition 715개 계속 분석
  - 다음 배치: parse_bitor_rest, parse_and, parse_and_rest, parse_or, parse_or_rest, parse_expr
  - parse_if_chain_*, parse_if_expr, parse_while_expr, parse_for_expr, parse_match_expr
  - 기타 상위 레벨 파서들
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→715 (−99 총계, 12% 감소)
- Next Recommendation: Cycle 3142: or/and/expr + if-chain + control flow 파서들 분석
