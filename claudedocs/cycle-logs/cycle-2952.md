# Cycle 2952: unwrap_bang not 추가 + t-first 3종 + 알고리즘 힌트 2종
Date: 2026-05-19

## Re-plan

Cycle 2951 Carry-Forward → 잔여 problem.md + 에러 패턴 개선.

발견:
1. `unwrap_bang` 패턴이 `!x` → `not x` 연결 누락 (boolean negation 안내 없음)
2. t-first 문제 중 52/59/64 미개선
3. 87_LIS, 88_knapsack 알고리즘 구현 힌트 없음

## Scope & Implementation

### Fix 1: unwrap_bang suggestion 개선

기존: "BMB has no ! operator (no macros, no unwrap)."
수정: boolean negation → `not x`, macro → 함수, unwrap → match 각각 안내.

```
"BMB has no ! operator.
- For boolean negation: use 'not x' (NOT '!x')
- For macros (println!, format!): use plain functions: println(x), format(\"{}\", x)
- For unwrap: use match or change type to plain T"
```

### Fix 2: t-first problem.md 3종

| 문제 | 추가 내용 |
|-----|---------|
| 52_base_convert | t 설명 + 루프 스케치 + str_concat 힌트 |
| 59_calendar_day | t 설명 + 루프 스케치 + 누적 일수 배열 힌트 |
| 64_many_params | t 설명 + 12개 read_int 명시 + 즉시 계산 |

### Fix 3: 알고리즘 힌트 2종

| 문제 | 추가 내용 |
|-----|---------|
| 87_longest_increasing | O(n²) DP + 전체 BMB 구현 스케치 |
| 88_knapsack_01 | 읽기 순서(interleaved) + 1D DP + 전체 BMB 구현 스케치 |

## Verification & Defect Resolution

```
cargo test --release -p bmb --test diagnostics_test (in progress)
```

## Reflection

### Scope fit
- ✅ 에러 패턴 개선 1종 (unwrap_bang)
- ✅ t-first 문제 잔여 3종 완료 → 총 17/17 t-first 문제 경고 완비
- ✅ 복잡 알고리즘 힌트 2종 (knapsack, LIS)

### 누적 수정 현황 (Cycles 2945-2952)
- 에러 패턴: 6개 신규 + 3개 개선
- problem.md: 30개 파일 수정
- diagnostics 테스트: 13 → 22 (+9)

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack B축 재측정** — 8 cycles분 fix 효과 검증
  2. **51_bracket_match `||` 지원** — BMB 언어에 `||` 추가
  3. **inttoptr UB (P3)** — HUMAN 결정 대기
- Pending Human Decisions: inttoptr Option A 승인
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2953 → 최종 정리 + HANDOFF 갱신 + 커밋
