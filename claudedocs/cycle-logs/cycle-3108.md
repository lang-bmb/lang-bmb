# Cycle 3108: Track B 잔여 그룹 계약 + TK_* post 계약 (270개+)
Date: 2026-05-25

## Re-plan
Cycle 3107 Carry-Forward: 잔여 그룹 계속. 계획 유효.

## Scope & Implementation

**총 추가 (이 Cycle): ~270개**

1. **글로벌 pos/idx/start (270개)**: 모든 함수에 `\bpos: i64\b`, `\bidx: i64\b`, `\bstart: i64\b` 패턴 일괄 적용
2. **depth/count/cur_temp/paren_pos/target_pos/n_pure (23개)**: 의미 있는 수치 파라미터
3. **c/kind 파라미터 (14개)**: `c >= 0` (byte 값), `kind >= 0` (token kind)
4. **단일 라인 body 패턴 (6개)**: pos/start 계약 — 같은 줄 body (= expr) 처리
5. **TK_* 상수 (106개)**: `post it > 0` — 토큰 ID 값이 항상 양수
6. **iter/n/temp/gcs_from/p3/p6 (11개)**: 특정 파라미터 계약
7. **버그 수정 (2개)**: cur_block 없는 함수에 잘못된 계약 추가 → 수정

**세션 전체 결과**: 1342 → 487 (-855, 63.7% 감소)

## Verification & Defect Resolution

- `bmb check`: ✅ (3211 warnings, 0 errors)  
- `bmb verify`: ✅ total:1417, verified:1417, failed:0
- 미계약 경로: 1342 → 1187 → 1132 → 1069 → 936 → 666 → 623 → 603 → 617 → 617 → 603 → 494 → 487

## Reflection

- Scope fit: 100%
- `missing_postcondition` 경고 3개 신규 — pre만 있고 post 없는 함수에서 정상 발생
- 487개 잔여: cf_*(49), is_*(37), optimize_*(18) 등 대부분 String 파라미터 — 수치 계약 어려움
- TK_* `post it > 0`은 컴파일러 최적화에 유용 (token kind 비교 최적화 가능)

## Carry-Forward

- Actionable: Cycle 3109 — 잔여 487개 중 추가 가능한 것 (post 계약 후보, is_* 분석)
- Structural Improvement Proposals: None
- Pending Human Decisions: M8 계획 공식 확정 (M8-A Track B)
- Roadmap Revisions: M8-A Track B 급진전 (1342→487)
- Next Recommendation: commit → 3-Stage Fixed Point 검증 → 잔여 계속 또는 M8 계획 수립
