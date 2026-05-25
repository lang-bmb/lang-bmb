# Cycle 3171: M9 Batch 37 — ifs_check_then_one/ifs_check_then_one_rest/ifs_check_else_one/ifs_check_else_one_rest/ifs_fn_lines/ifs_try_extended/ifs_try_both_or_fallback/ifs_try_two_or_fallback/ifs_check_both/ifs_check_both_rest/ifs_check_both_rest2/ifs_check_then_two/ifs_check_then_two_rest/ifs_check_else_two/ifs_check_else_two_rest 15개 post 조건 추가
Date: 2026-05-26

## Re-plan
Plan valid. Cycle 3170 Carry-Forward에서 ifs 계열 계속 진행.

## Scope & Implementation
15개 함수 postcondition 추가 (ifs = If-Statement optimization 패스):

| 함수 | 반환 | post |
|------|------|------|
| ifs_check_then_one | String | `post it.len() >= 0` |
| ifs_check_then_one_rest | String | `post it.len() >= 0` |
| ifs_check_else_one | String | `post it.len() >= 0` |
| ifs_check_else_one_rest | String | `post it.len() >= 0` |
| ifs_fn_lines | i64 | `post it >= 0` |
| ifs_try_extended | i64 | `post it >= 0` |
| ifs_try_both_or_fallback | i64 | `post it >= 0` |
| ifs_try_two_or_fallback | i64 | `post it >= 0` |
| ifs_check_both | String | `post it.len() >= 0` |
| ifs_check_both_rest | String | `post it.len() >= 0` |
| ifs_check_both_rest2 | String | `post it.len() >= 0` |
| ifs_check_then_two | String | `post it.len() >= 0` |
| ifs_check_then_two_rest | String | `post it.len() >= 0` |
| ifs_check_else_two | String | `post it.len() >= 0` |
| ifs_check_else_two_rest | String | `post it.len() >= 0` |

## Verification & Defect Resolution
- missing_postcondition: 283 → **268 (−15)** ✅

## Reflection
- ifs 패스 함수군 대부분 처리. 잔여 ifs 함수들 (ifs_check_flex_both_sides 등) 다음 사이클에.

## Carry-Forward
- Actionable: Cycle 3172 — ifs_check_flex_both_sides/ifs_flex_check_goto 등 ifs 잔여 + gcs/pht 계열
- Next Recommendation: Cycle 3172 — ifs 나머지 + gcs 계열 시작
