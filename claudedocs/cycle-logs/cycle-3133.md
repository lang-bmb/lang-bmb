# Cycle 3133: M8-A bool partial contracts 13개 교체 + M8-A 실질 완료
Date: 2026-05-25

## Re-plan
Plan valid. Bool trivial 19개 잔여 전수 분석 → 13개 교체 가능, 6개 최종 skip 확정.

## Scope & Implementation
13개 교체 — `post it or not it` → meaningful partial/exact contracts:

**필요조건 패턴 (9개)**:
- is_aliased_trunc_ir(line, aliases) L9273 → `post not it or line.contains("trunc i64 ")`
- is_aliased_inttoptr(line, aliases) L9558 → `post not it or line.contains("inttoptr i64 ")`
- mn_is_pure_call(line) L9614 → `post not it or line.contains("@llvm.") or line.contains("@bmb_f64_")`
- mn_is_readonly_call(line) L9665 → `post not it or line.contains("@bmb_") or line.contains("@llvm.")`
- slf_is_bb_boundary(line) L10548 → `post not it or line.contains(":") or line.contains("br ") or line.contains("ret ")`
- trl_block_has_return(fn_mir, label) L12903 → `post not it or fn_mir.contains(label)`
- lr2l_detect(fn_ir, fn_name, param) L18653 → `post not it or fn_ir.contains(fn_name)`

**함수 호출 패턴 (4개)**:
- cf_can_simplify(line, table) L11400 → `post not it or cf_is_arith(line) or cf_is_shift(line) or cf_is_bitwise(line)`
- cf_can_fold(line, table) L11534 → `post not it or cf_is_arith(line) or cf_is_cmp(line) or cf_is_neg(line) or cf_is_not(line) or cf_is_shift(line) or cf_is_bitwise(line)`
- cf_can_simplify_branch(line, table) L11647 → `post not it or cf_is_branch(line)`
- cf_can_fold_phi(line, table) L11671 → `post not it or ube_is_phi(line)`
- cf_can_simplify_select(line, table) L11727 → `post not it or cf_is_select(line)`

**특수 패턴 (1개)**:
- mlcse_is_safe_call(fn_name, pure_set, safe_set) L12480 → `post it or fn_name.len() > 0`
  (fn_name == "" → 항상 true, 따라서 fn_name == "" → it)

**최종 skip 6개 확정**:
- enum_has_payload/enum_variant_has_payload: 레지스트리 파싱 복잡 — 함수 호출 결과 의존
- is_float_expr/is_pure_expr: 재귀 AST 탐색 — 선언적 계약 불가
- is_var_unused_in_ir: 2-pass 출현 카운팅 — 단순 필요조건 없음
- licm_all_args_invariant: 재귀적 불변성 검사 — 복합 조건

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 3007 → 2994 (−13)
- bmb verify ✅ 953/953

## Reflection
- Bool trivials: 19 → 6 (−13 교체 확인)
- 6개 잔여는 진정 복잡한 함수 — 임의 수준의 partial contract도 inaccurate하거나 표현 불가
- M8-A 실질 완료: bool 97개 중 91개 교체 (94%), 6개 irreducible skip
- i64 trivial 7개(`post it == it`): Cycle 3115 결정 "임의 값, skip 확정" 유지
- String trivial 77개: Cycle 3130 결정 "skip 확정" 유지
- 총 warnings: 3173 → 2994 (−179) M8-A/B 전체를 통해

## Carry-Forward
- Actionable: M8-A/B 실질 완료 — 다음 마일스톤 검토
  - 잔여 6 bool + 7 i64 + 77 String trivials: 구조적으로 skip, 변경 없음
  - 다음 단계: M9 또는 다른 고가치 작업 검토
  - ROADMAP §M8 완료 마킹 권고
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M8-A bool 실질 완료 (91/97 교체, 6개 irreducible). ROADMAP 업데이트 필요.
- Next Recommendation: Cycle 3134: ROADMAP M8 완료 마킹 + commit + 다음 마일스톤 검토
