# Cycle 3150: M9 Batch 16 — step_binop/unary/if/let/mut/call 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 598개 잔여. step_* MIR 함수들 2차 배치.

## Scope & Implementation
15개 post 조건 추가 (모두 `post it.len() >= 1` — make_step 또는 make_step_leaf 반환):
- `step_binop_start` — make_step(cur_temp, ...) or/and 분기
- `step_binop_right` — make_step(cur_temp, ...) 단일 경로
- `step_binop_final` — make_step(t+1, ...) 또는 make_step_leaf(t+1) (safe 분기)
- `step_unary_start` — make_step(cur_temp, ...) 단일 경로
- `step_unary_final` — make_step_leaf(cur_temp + 1) 단일 경로
- `step_if_start` — make_step(...) 두 경로 (IZ/IT 분기)
- `step_if_select` — make_step_leaf(else_temp + 1)
- `step_if_then` — make_step(cur_temp, my_block + 1, ...)
- `step_if_else` — make_step(cur_temp, cur_block, ...)
- `step_if_final` — make_step(cur_temp + 1, cur_block, merge_label, "")
- `step_let_start` — make_step(cur_temp, ...)
- `step_let_body` — make_step(cur_temp, ...)
- `step_mut_start` — make_step(cur_temp, ...)
- `step_mut_body` — make_step(cur_temp, ...)
- `step_call_start` — make_step_leaf(cur_temp + 1) 또는 make_step(...) (args 분기)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2949 → 2949 (0 net; missing_postcondition 598→583 = −15)
  - semantic_duplication: +15 (정상 — net 상쇄)
- bmb verify: 745/745 → 730/730 (0 failed, total −15: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- step_binop_final: make_step(t+1, ...) 또는 make_step_leaf(t+1) 두 경로 — 둘 다 post it.len() >= 1 성립
- step_if_select: lower_expr_sb 결과를 통해 else_temp 계산 → make_step_leaf(else_temp + 1)
- make_step/make_step_leaf의 post it.len() >= 1 계약이 모든 step_* 함수 검증의 기반
- 남은 missing_postcondition: 583개

## Carry-Forward
- Actionable: missing_postcondition 583개 계속 분석
  - 다음 배치: step_call_arg/final, step_method_start/arg/final, step_nullable_result/or, step_unit, step_seq_start/second, step_assign_start/final, step_array_index_start/idx 등
  - 나머지 step_* 함수들 (step_break, step_continue, step_return, step_return_value, step_cast_*, step_array_*, step_set_*, step_field_*)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→583 (−231 총계, 28.4% 감소)
- Next Recommendation: Cycle 3151: step_call/method/nullable/seq/assign/array 계열 15개
