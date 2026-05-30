# Cycle 3318: P1 Phase 2 — contracts_check + lint_effects violations 형식 통일
Date: 2026-05-30

## Re-plan
P1 Phase 2: contracts_check와 lint_effects warnings의 `"rule":` → `"type":` 통일.

## Scope & Implementation
- `count_rule_entries`: `{"rule":` → `{"type":` 패턴 변경 (count_caller_entries와 동일해짐)
- lint_effects 3종: `{"rule":"effect_pure_violation"` → `{"type":"effect_pure_violation"`, `{"rule":"effect_propagation"` → `{"type":"effect_propagation"`, `{"rule":"missing_effect_annotation"` → `{"type":"missing_effect_annotation"`
- contracts_check 5종: max_params/require_postcondition/forbid_effect/forbid_function/require_effect_annotation 모두 `"rule":` → `"type":`

## Verification & Defect Resolution
- cargo test: 3800+47+22+2390+23 = 6282 PASS, 0 FAILED ✅
- Stage 1 build (compiler_s1d.exe): 성공 ✅
- contracts-check 동작 확인: `{"type":"max_params","function":"..."}` 형식 ✅
- Within-gen Fixed Point: fp3318a.ll == fp3318b.ll ✅

## Reflection
- 모든 `"rule":` 완전 제거 완료 — `"type":` 으로 통일
- count_rule_entries와 count_caller_entries가 이제 동일한 패턴 사용 (중복)
- 향후: count_rule_entries/count_caller_entries 중 하나로 통합 가능

## Carry-Forward
- Actionable: P1 Phase 3 — semantic_duplicate violations 형식 통일 (`{"fn_a":"...","fn_b":"..."}` → `{"type":"semantic_duplicate","function":"...","similar_to":"..."}`)
- Structural Improvement Proposals: count_rule_entries + count_caller_entries 통합 (이름은 count_viol_entries)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3319 — P1 Phase 3 semantic_duplicate + 카운터 통합
