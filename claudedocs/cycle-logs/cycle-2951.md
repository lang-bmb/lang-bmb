# Cycle 2951: t-first 다중 쿼리 problem.md 일괄 보강 (7종)
Date: 2026-05-19

## Re-plan

Cycle 2950 Carry-Forward → 잔여 problem.md 개선 탐색.

t-first 다중 쿼리 패턴이 있는 모든 문제를 grep으로 식별 (17종) → 이미 경고가 없는 7종 발견.
이번 사이클에서 일괄 보강.

## Scope & Implementation

### 식별된 누락 7종

| 문제 | 특이사항 |
|-----|---------|
| 61_mutual_recursion | t 후 n 하나 |
| 62_deep_nesting | t 후 n 하나 |
| 66_acc_recursion | t 후 n 하나 |
| 68_boundary_values | t 후 (value, lo, hi) 순서 |
| 73_palindrome_check | t 후 (n, n values) — 복잡 |
| 74_majority_element | t 후 (n, n values) — 복잡 |
| 81_dispatch_table | t 후 (op, a, b) |

### 수정 내용

모든 7종에:
1. **t 설명을 "(number of test cases)"로 명시**
2. **"IMPORTANT: t is the query count" 섹션 추가**
3. **BMB 구현 스케치 추가**

복잡한 케이스(73/74): 내부 루프에서 n 읽기 + n values 읽기 전체 스케치 제공.

## Verification & Defect Resolution

```
cargo test --release -p bmb --test diagnostics_test
  22/22 PASSED — 회귀 없음
```

## Reflection

### Scope fit
- ✅ t-first 다중 쿼리 problem.md 전수 조사 + 일괄 보강
- ✅ 특히 73/74 (t + n + n values 이중 루프) 복잡 케이스에 전체 스케치 제공

### 누적 수정 현황 (Cycles 2945-2951)
- 에러 패턴: 6개 신규 + 2개 개선
- problem.md: 24개 파일 수정 (이번 사이클 +7)
- diagnostics 테스트: 13 → 22 (+9)

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack B축 재측정** — 7 cycles분 fix 효과 검증
  2. **51_bracket_match `||` 지원** — BMB 언어에 `||` 추가
  3. **inttoptr UB (P3)** — HUMAN 결정 대기
- Pending Human Decisions: inttoptr Option A 승인
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2952 → 나머지 문제 탐색 or 추가 에러 패턴 발굴
