# Cycle 3317: P1 violations 형식 통일 Phase 1 — effect_verify
Date: 2026-05-30

## Re-plan
P1 violations 형식 통일 시작. Phase 1 = effect_verify 3가지 위반 유형 통일.

## Scope & Implementation
- `count_caller_entries`: `{"caller":` → `{"type":` 패턴 변경 (카운터 통일)
- `eff_emit_viol_pair`: `{"caller":"...","callee":"...","caller_effect":"...","callee_effect":"..."}` → `{"type":"effect_mismatch","function":"...","callee":"...","caller_effect":"...","callee_effect":"...","message":"..."}`
- `eff_collect_pure_calls`: `{"caller":"...","callee":"...","caller_effect":"pure",...}` → `{"type":"pure_violation","function":"...","callee":"...","callee_effect":"...","message":"..."}`
- `eff_collect_missing_annot`: `{"caller":"...","callee":"","caller_effect":"missing",...}` → `{"type":"missing_annotation","function":"...","missing_effect":"...","message":"..."}`

## Verification & Defect Resolution
- cargo test: 3800+47+22+2390+23 = 6282 PASS, 0 FAILED ✅
- Stage 1 build (compiler_s1c.exe): 성공 ✅
- effect-verify 동작 확인: pure_violation 형식 `{"type":"pure_violation","function":"..."}` ✅
- violations_count 카운터 정확성 유지 ✅
- Within-gen Fixed Point: fp3317a.ll == fp3317b.ll ✅

## Reflection
- effect_verify 3종 위반이 모두 `type` + `function` 필드로 통일됨
- contracts_check, semantic_duplicate는 다음 사이클에서 통일
- count_caller_entries 이름은 레거시이지만 기능은 범용적으로 업데이트됨

## Carry-Forward
- Actionable: P1 Phase 2 — contracts_check violations 형식 통일 (`{"rule":"..."}` → `{"type":"...","function":"..."}`)
- Structural Improvement Proposals: count_caller_entries 이름을 count_viol_entries로 리네이밍 (레거시)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3318 — P1 Phase 2 contracts_check
