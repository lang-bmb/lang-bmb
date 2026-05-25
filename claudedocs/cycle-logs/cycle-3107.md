# Cycle 3107: Track B 대규모 배치 계약 추가 (400개+)
Date: 2026-05-25

## Re-plan
Cycle 3106 Carry-Forward: 잔여 그룹 계속. 패턴 파악 후 전체 pos/idx/start 등 일괄 처리.

## Scope & Implementation

**총 추가: 403개 (step_62 + 71 + 270개)**

1. **step_* (62개)**: `cur_temp >= 0 and cur_block >= 0`
2. **ifs_/fmt_/check_/pfcse_/licm_/build_/trl_/cf_ 위치 파라미터 (71개)**: pos/idx/branch_pos/end
3. **전체 함수 pos/idx/start/offset >= 0 (270개)**: 글로벌 regex 적용
4. **depth/count/cur_temp/paren_pos/target_pos/n_pure (23개)**
5. **버그 수정**: safe_divzero_check/safe_bounds_check에 cur_block 없는 상태에서 cur_block 계약 추가됨 → 수정 (pre cur_temp >= 0만 유지)
6. **counter/temp/fn_start/n/i 등 추가 (20개)**

**결과**: 1342 → 623 (-719개, 53.6% 감소)

## Verification & Defect Resolution

- **P0 버그 수정**: `cur_temp >= 0 and cur_block >= 0` 계약이 cur_block 없는 함수(safe_divzero_check, safe_bounds_check)에 잘못 추가 → 발견 즉시 수정
- `bmb check bootstrap/compiler.bmb`: ✅ (3212 warnings, 0 errors)
- `bmb verify bootstrap/compiler.bmb`: ✅ total:1417, verified:1417, failed:0
- 미계약: 1342 → 623 (-719)

## Reflection

- Scope fit: 100%
- 글로벌 패턴 적용(pos/idx/start) 매우 효율적 — 단일 regex로 260개 처리
- cur_block 의존 패턴은 반드시 파라미터 목록 확인 후 적용 필요 (다행히 2개만 오류)
- TK_* (106), cf_* (52), is_* (47) 등 non-numeric 함수 그룹은 별도 전략 필요

## Carry-Forward

- Actionable: Cycle 3108 — 잔여 623개 분석 (TK_* 제외 517개 중 contractable 찾기)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: 1342 → 623 (53.6% 감소) — M8-A Track B 급진전
- Next Recommendation: TK_* 상수 함수 제외 후 실제 contractable 패턴 계속
