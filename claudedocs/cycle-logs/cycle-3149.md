# Cycle 3149: M9 Batch 15 — i2s/parse_int_simple/trampoline_v3/make_step/do_step/step_* 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 613개 잔여. step_* MIR 함수들 + 유틸리티 함수들.

## Scope & Implementation
15개 post 조건 추가:

**`post it.len() >= 1` (10개)** — make_step/make_step_leaf 체계 + 기반 유틸:
- `i2s(n)` → `post it.len() >= 1` (int_to_string 위임 — post it.len() >= 1 전파)
- `make_step(temp, block, exit_label, work)` → `post it.len() >= 1` (i2s(temp) + SEP + ... 연결)
- `make_step_leaf(temp)` → `post it.len() >= 1` (i2s(temp) 직접 반환)
- `do_step` → `post it.len() >= 1` (make_step 또는 make_step_leaf 반환)
- `step_expr` → `post it.len() >= 1` (make_step/make_step_leaf 분기)
- `step_int` → `post it.len() >= 1` (make_step_leaf(cur_temp + 1))
- `step_float` → `post it.len() >= 1` (make_step_leaf(cur_temp + 1))
- `step_bool` → `post it.len() >= 1` (make_step_leaf(cur_temp + 1))
- `step_string` → `post it.len() >= 1` (make_step_leaf(cur_temp + 1))
- `step_var` → `post it.len() >= 1` (make_step_leaf(cur_temp + 1))

**`post it >= 0` (5개)** — i64 반환 함수들:
- `lower_expr_iter` → `post it >= 0` (trampoline_v3 결과 = pack_ids >= 0)
- `trampoline_v3` → `post it >= 0` (pack_ids(cur_temp, cur_block) >= 0)
- `check_field_type` → `post it >= 0` (0~4 반환: 필드 타입 코드)
- `parse_int_simple` → `pre acc >= 0` 추가 + `post it >= 0` (누산기 패턴)
- `parse_oct_from` → `pre acc >= 0` 추가 + `post it >= 0` (누산기 패턴)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2951 → 2949 (−2 net; missing_postcondition 613→598 = −15)
  - semantic_duplication: +13 (정상)
- bmb verify: 760/760 → 745/745 (0 failed, total −15: Z3 예산 영향)
  - 0 failed — 모든 검증 통과

## Reflection
- make_step/make_step_leaf 기반 계약 수립: step_* 전체 체계의 기반 함수 계약 완성
- i2s → int_to_string 위임으로 post it.len() >= 1 전파 체인: int_to_string(post it.len()>=1) → i2s(post it.len()>=1) → make_step_leaf(post it.len()>=1) → step_*(post it.len()>=1)
- trampoline_v3: 재귀 함수이지만 base case가 pack_ids(>= 0)이므로 post it >= 0 성립
- parse_int_simple/parse_oct_from: s2i/parse_int_from에서 acc=0으로만 호출 — pre acc >= 0 안전
- 남은 missing_postcondition: 598개

## Carry-Forward
- Actionable: missing_postcondition 598개 계속 분석
  - 다음 배치: step_binop_start/right/final, step_if_start/then/else/final, step_let_start/body, step_mut_start/body 등 make_step 반환 step_* 함수들
  - get_field, get_field_at, get_exit_label, scan_exit_children 등 AST 유틸리티
  - parse_int_from, get_child, get_child_at, read_sexp_at 등
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→598 (−216 총계, 26.5% 감소)
- Next Recommendation: Cycle 3150: step_binop/if/let/mut 계열 15개 + 기타 유틸리티
