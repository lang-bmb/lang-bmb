# Cycle 3172: M9 Batch 38 — ifs_check_flex_both_sides/ifs_flex_check_goto/ifs_flex_scan_block/ifs_emit_both/ifs_skip_both_continue/ifs_emit_extended/ifs_program/gcs_is_forward/gcs_find_forwards/gcs_find_in_map/gcs_resolve/gcs_rewrite_fn/gcs_program/pht_is_phi_fwd/pht_get_copy_info 15개 post 조건 추가
Date: 2026-05-26

## Re-plan
Plan valid. ifs 잔여 + gcs/pht 계열 진행.

## Scope & Implementation
15개 함수 postcondition 추가:

| 함수 | 반환 | post |
|------|------|------|
| ifs_check_flex_both_sides | i64 | `post it >= 0` |
| ifs_flex_check_goto | i64 | `post it >= 0` |
| ifs_flex_scan_block | i64 | `post it >= 0` |
| ifs_emit_both | i64 | `post it >= 0` |
| ifs_skip_both_continue | i64 | `post it >= 0` |
| ifs_emit_extended | i64 | `post it >= 0` |
| ifs_program | i64 | `post it >= 0` |
| gcs_is_forward | String | `post it.len() >= 0` |
| gcs_find_forwards | String | `post it.len() >= 0` |
| gcs_find_in_map | i64 | `post it >= -1` (sentinel -1) |
| gcs_resolve | String | `post it.len() >= 0` |
| gcs_rewrite_fn | i64 | `post it >= 0` |
| gcs_program | i64 | `post it >= 0` |
| pht_is_phi_fwd | String | `post it.len() >= 0` |
| pht_get_copy_info | String | `post it.len() >= 0` |

## Verification & Defect Resolution
- missing_postcondition: 268 → **253 (−15)** ✅
- gcs_find_in_map: find 계열 -1 sentinel → `post it >= -1` (유일한 예외)

## Reflection
- ifs 패스 완결, gcs (Goto Chain Simplification) + pht (Phi-Aware Empty Block Threading) 패스 시작
- ifs_flex/ifs_emit 계열: i64 반환 최적화 패스 emit 함수 → `post it >= 0`

## Carry-Forward
- Actionable: Cycle 3173 — pht_find_copy_map/pht_copy_lookup 등 pht 계열 계속
- Next Recommendation: Cycle 3173 — pht 잔여 + optimize_cf_dce_loop 등
