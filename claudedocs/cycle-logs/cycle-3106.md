# Cycle 3106: Track B lower_* 계약 배치 추가 (63개)
Date: 2026-05-25

## Re-plan
Cycle 3105 Carry-Forward: lower_* 63개 계약 추가. 계획 유효.

## Scope & Implementation

**lower_* 계약 63개 추가 (전체 처리)**:

파라미터 패턴별 분류 및 배치 적용:

| 패턴 | 개수 | 계약 |
|------|------|------|
| idx+count before temp+block | 3 | `temp_id >= 0 and block_id >= 0 and idx >= 0 and count >= 0` |
| total before temp+block | 1 | `temp_id >= 0 and block_id >= 0 and idx >= 0 and total >= 0` |
| temp+block+idx (after) | 3 | `temp_id >= 0 and block_id >= 0 and idx >= 0` |
| temp+block only | 52 | `temp_id >= 0 and block_id >= 0` |
| field_idx+n_children | 1 | `field_idx >= 0 and n_children >= 0 and cur_temp >= 0 and cur_block >= 0` |
| idx+safe | 1 | `idx >= 0 and safe >= 0` |
| safe only | 2 | `safe >= 0` |

**결과**: 1132 → 1069 (63개 감소)

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3231 warnings, 0 errors)
- `bmb verify bootstrap/compiler.bmb`: ✅ total:1417, verified:1417, failed:0
- lower_* 잔여: 0개 ✅

## Reflection

- Scope fit: 100% (lower_* 완전 처리)
- 파라미터 패턴이 parse_*/llvm_*보다 다양 — 카테고리별 분류 전략 효과적
- safe: i64는 boolean 플래그 (0/1), safe >= 0 계약은 trivially true지만 의미 있음

## Carry-Forward

- Actionable: Cycle 3107 — 다음 그룹 (step_/emit_/check_/type_ 등) 계약 추가
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: 나머지 1069개 중 pos/idx/temp_id 패턴 가진 그룹 계속
