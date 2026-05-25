# Cycle 3155: M9 Batch 21 — lower_float/bool/string/var/binop/unary/if/let/call_sb 15개 post it >= 0 추가
Date: 2026-05-25

## Re-plan
Plan valid. lower_*_sb 계열 기본 표현식 핸들러 계속 처리.

## Scope & Implementation
15개 `post it >= 0` 추가 (모두 pack_ids 반환):
- `lower_float_sb` — pack_ids(temp_id+1, block_id)
- `lower_bool_sb` — pack_ids(temp_id+1, block_id)
- `lower_string_sb` — pack_ids(temp_id+1, block_id)
- `lower_var_sb` — pack_ids(temp_id+1, block_id)
- `lower_binop_sb` — lower_if_branch_sb / pack_ids(t+1, right_block)
- `lower_unary_sb` — pack_ids(operand_temp+1, operand_block)
- `lower_if_select_sb` — pack_ids(else_temp+1, cond_blk)
- `lower_if_sb` — lower_if_select_sb / lower_if_branch_sb
- `lower_if_branch_sb` — pack_ids(else_temp+1, else_block)
- `lower_let_sb` — lower_expr_sb(body, val_temp, ...)
- `lower_let_mut_sb` — lower_expr_sb(body, val_temp, ...)
- `lower_call_sb` — lower_call_args_sb(...)
- `lower_call_args_sb` — pack_ids(temp_id+1, block_id) / 재귀
- `lower_method_sb` — lower_nullable_method_sb / lower_method_args_sb
- `lower_nullable_method_sb` — pack_ids(...) 다중 분기

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- missing_postcondition: 523 → **508** (−15)

## Reflection
- 기본 표현식 MIR lowering 핸들러 계열 완결
- lower_expr_sb (최상위 dispatch) 아직 미처리 — 남은 lower_*_sb 처리 후 추가 예정
- 로드맵 영향: 없음

## Carry-Forward
- Actionable: missing_postcondition 508개 — lower_method_args/extra_args/block/unit/seq/assign/while/loop + lower_break/continue/return + lower_field/set_field/set_var + lower_index/* + lower_for_* + lower_expr_sb
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→508 (−306 총계, 37.6% 감소)
- Next Recommendation: Cycle 3156: lower_method_args/block/unit/seq/assign/while/loop/break/continue/return 계열 15개
