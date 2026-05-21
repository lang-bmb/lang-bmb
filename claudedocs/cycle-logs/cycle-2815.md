# Cycle 2815: C baseline 전수 검증 + 3개 버그 수정

Date: 2026-05-13

## Re-plan

ISSUE-20260326-external-problem-validation 처방 #1: "C baselines verified for all 100 problems". 자율 가능.

## Scope & Implementation

**C baseline 전수 검증 실행**: `gcc -O2 baseline.c` → 12 tests per problem → 100개 전체.

**발견 결함 3개**:

| 문제 | 결함 | 원인 |
|------|------|------|
| `46_csv_parser` | `baseline.c` 오작동 | 현재 문제는 "integer field count + sum" 인데, `baseline.c`가 "CSV 텍스트 쉼표 카운트"를 구현 — 전혀 다른 알고리즘 |
| `47_word_count` | `baseline.c` 오작동 | 문제가 Cycle 2811에서 "Longest Consecutive Run"으로 재작성됐는데 `baseline.c`는 여전히 "단어 카운트" 구현 |
| `49_roman_to_int` | `tests.json` 오입력 | Test 4: stdin=`"1 4 1000 1000 1000"` (n=4 이지만 값 3개) → C는 uninitialized memory 읽어 4785215 출력. BMB는 EOF=0 읽어 3000 — test 자체가 UB에 의존 |

**수정**:
1. `46_csv_parser/baseline.c` → integer 기반 field_count + sum 구현으로 전면 교체
2. `47_word_count/baseline.c` → longest consecutive run 구현으로 전면 교체
3. `49_roman_to_int/tests.json` test 4 → `"1 4 ..."` → `"1 3 1000 1000 1000"` (n=3으로 수정)

**재검증**: C baseline 100/100 PASS ✅ / BMB solution 3개 모두 12/12 PASS ✅

## Verification & Defect Resolution

- C baseline 100개 문제 전수: **100/100 PASS** ✅
- BMB solution 46/47/49 재검증: **36/36 PASS** ✅

## Reflection

**Scope fit**: 완전 부합 (ISSUE 처방 #1 달성).
**Latent defects**: 없음. 49_roman_to_int test 4 UB → deterministic로 수정 (실제 테스트 케이스 의미 보존: MMM=3000).
**Philosophy drift**: 없음. 테스트 정합성 = 벤치마크 신뢰도의 근간.
**Roadmap impact**: 없음 (P-track 수치 변동 없음).

## Carry-Forward

- Actionable: Cycle 2816 — ISSUE-external-problem-validation 처방 #2 (external-sourced problems) 또는 통계 테스트 구현
- Structural Improvement Proposals: `baseline.c`와 `solution.bmb` 정합성 검사를 CI에 통합 제안 (현재 수동)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 통계 검정 구현 (scipy 기반) 또는 BMB reference 문서 개선
