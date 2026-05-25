# Cycle 3132: M8-A bool partial contracts 6개 교체 (post not it or X 패턴)
Date: 2026-05-25

## Re-plan
Plan valid. Bool trivial 25개 잔여 분석 → 6개 partial contract 교체 가능 확인.
나머지 19개는 재귀/복합/2-param 분석으로 skip.

## Scope & Implementation
6개 교체 — `post it or not it` → `post not it or <necessary condition>`:

**필요조건 패턴 (6개)**:
- is_identity_copy_ir(line) L9096 → `post not it or line.contains("add nsw i64 0, ") or line.contains("fadd nsz double 0.0, ")`
  true 시 RHS가 반드시 위 패턴 포함
- cf_is_int_const(line) L11148 → `post not it or line.contains(" = const ")`
  true 시 line이 반드시 "= const" 포함
- dce_var_in_rhs(line, target) L11827 → `post not it or line.contains(target)`
  RHS slice도 line의 부분집합이므로 sound
- dsa_is_dead_line(line, dead) L10692 → `post not it or dead.len() > 0`
  dead == "" → 항상 false, exact necessary condition
- ifs_is_pure(line) L13791 → `post not it or line.contains(" = ")`
  pure 판정은 반드시 "= " assignment 패턴 필요
- decl_has_conflict(decl_line, user_fn_names) L18778 → `post not it or user_fn_names.len() > 0`
  user_fn_names == "" → 항상 false, exact necessary condition

**비교체 분류 (19개 잔여 skip 이유)**:
- enum_has_payload/enum_variant_has_payload: 레지스트리 파싱 복잡 — 함수 호출 결과 의존
- is_float_expr/is_pure_expr: 재귀 AST 탐색 — 선언적 계약 불가
- is_aliased_trunc_ir/is_aliased_inttoptr: 2-param aliasing 분석
- is_var_unused_in_ir: 2번째 출현 없음 검증 — 함수 호출 2회 필요
- mn_is_pure_call/mn_is_readonly_call: 파싱 후 이름 조회 — 파싱 선행 조건 복잡
- slf_is_bb_boundary: trim + ends_with 복합 — contains 과근사 부정확
- cf_can_simplify/fold/branch/phi/select: 복수 algebraic conditions
- mlcse_is_safe_call: 함수 호출(pfcse_is_pure) 포함 — body 재진술 위험
- trl_block_has_return/licm_all_args_invariant/lr2l_detect: 구조 검색 — 복잡

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 3013 → 3007 (−6)
- bmb verify ✅ 953/953

## Reflection
- Bool trivials: 25 → 19 (−6 교체 확인)
- `post not it or X` 패턴: tautology보다 의미있는 필요조건 — "반환값이 true면 이 조건이 반드시 성립"
- 19개 잔여: 재귀/복합/aliasing 분석으로 간단한 필요조건 표현 불가
- warnings -6: 각 함수가 독립 non-trivial 계약을 획득, semantic_duplication 축소

## Carry-Forward
- Actionable: 19개 잔여 bool trivial 최종 확인
  - 모두 복잡한 함수 바디로 의미있는 필요조건도 표현 어려움
  - M8-A bool 교체 실질적 완료 결정 필요
  - i64 trivial 7개(`post it == it`) — Cycle 3115에서 "임의 값, skip 확정"으로 결정됨
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M8-A bool 실질 완료 — 잔여 19개 skip 결정 권고
- Next Recommendation: Cycle 3133: 잔여 19개 bool trivial 최종 분석 + M8-A 완료 선언 또는 다음 마일스톤 전환
