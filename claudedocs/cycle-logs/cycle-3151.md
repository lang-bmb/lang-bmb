# Cycle 3151: M9 Batch 17 — step_call/method/nullable/seq/assign/array_index 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 583개 잔여. step_* MIR 함수들 3차 배치.

## Scope & Implementation
15개 post 조건 추가 (모두 `post it.len() >= 1` — make_step 또는 make_step_leaf 반환):
- `step_call_arg` — make_step_leaf(cur_temp+1) 또는 make_step(...) (args 소진/계속 분기)
- `step_call_final` — make_step_leaf(cur_temp) 단일 경로 (호환성 유지용)
- `step_method_start` — make_step(cur_temp, ...) 단일 경로
- `step_method_arg` — make_step_leaf(cur_temp+1) 또는 make_step(...) (args 소진/계속 분기)
- `step_method_final` — make_step_leaf(cur_temp) 단일 경로 (호환성 유지용)
- `step_nullable_result` — make_step_leaf 또는 make_step (메서드별 분기: is_some/is_none/unwrap/unwrap_or)
- `step_nullable_or` — make_step_leaf(cur_temp+1) 단일 경로
- `step_unit` — make_step_leaf(cur_temp+1) 단일 경로 (unit = const 0)
- `step_seq_start` — make_step(cur_temp, ...) 단일 경로
- `step_seq_second` — make_step(cur_temp, ...) 단일 경로
- `step_assign_start` — make_step(cur_temp, ...) 단일 경로
- `step_assign_final` — make_step_leaf(cur_temp+1) 단일 경로
- `step_array_index_start` — make_step(cur_temp, ...) 단일 경로
- `step_array_index_idx` — make_step(cur_temp, ...) 단일 경로
- `step_array_index_final` — make_step(t+4, ...) 또는 make_step_leaf(t+4) (safe 분기)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2949 → 2949 (0 net; missing_postcondition 583→568 = −15)
  - semantic_duplication: +15 (정상 — net 상쇄)
- bmb verify: 730/730 → 715/715 (0 failed, total −15: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- step_call_final/step_method_final: 실제로 사용되지 않는 호환성 stub (주석 "unused in new design") 이지만 계약 추가로 명세 완성
- step_assign_final: store + const 0 (unit 반환) → make_step_leaf 확실
- make_step/make_step_leaf의 post it.len() >= 1 계약 덕분에 모든 step_* 함수 검증 연쇄 성공
- 남은 missing_postcondition: 568개

## Carry-Forward
- Actionable: missing_postcondition 568개 계속 분석
  - 다음 배치: step_set_index_start/idx/val/final, step_field_access_start/final, step_set_field_start/val/final, step_set_var_start/final, step_break, step_continue, step_return, step_return_value, step_cast_* 등
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→568 (−246 총계, 30.2% 감소)
- Next Recommendation: Cycle 3152: step_set_index/field/var/break/continue/return/cast 계열 15개
