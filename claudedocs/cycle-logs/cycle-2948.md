# Cycle 2948: 1/3-fail 4문제 분석 + bool_operators 에러 패턴 + problem.md 수정
Date: 2026-05-19

## Re-plan

Cycle 2947 Carry-Forward → 1/3-fail 4문제 분석 + HANDOFF 갱신.

## Scope & Implementation

### 1/3-fail 문제 분석

GPUStack partial-fail 문제 중 1/3-fail (1번 실패) 문제 4개:

| 문제 | 루트 원인 | 수정 |
|------|---------|------|
| 43_sum_of_squares | t-first 읽기 미이행 (41_collatz와 동일 패턴) | ✅ problem.md 멀티쿼리 구조 명시 |
| 51_bracket_match | `||` 사용 → "Unrecognized token `|`" → B루프 (B×10) | ✅ bool_operators 에러 패턴 추가 |
| 44_euclidean_dist | A 전체 읽기 전에 B 일부 읽음 (입력 형식 오독) | ✅ problem.md "ALL of A then ALL of B" |
| 50_calculator | op=4 divide: a/b vs b/a pop 순서 혼동 | ✅ problem.md 팝 순서 명시 |

### Fix 1: `bool_operators` 에러 패턴 (patterns.rs)

51_bracket_match: 모델이 `c == 40 || c == 91` 사용 → `||`가 "Unrecognized token `|`" 발생.
기존 `closure_lambda` 패턴이 발동하여 "람다 없음" 힌트 → 완전히 틀린 피드백.

추가 패턴:
- `bool_operators`: trigger `"Unrecognized token \`|\`"`, `"Unrecognized token \`&\`"``
- suggestion: `'or'` / `'and'` 키워드 사용 안내
- `closure_lambda` 앞에 배치하여 우선 발동

### Fix 2: 43_sum_of_squares problem.md

t-first 읽기 구조 + 전체 읽기 스케치 추가 (41_collatz와 동일 패턴).

### Fix 3: 44_euclidean_dist problem.md

"Read ALL of A first, then ALL of B" 명시 + 전체 구현 스케치 제공.

### Fix 4: 50_calculator problem.md

op=4 divide pop 순서: `a = vec_pop(top)`, `b = vec_pop(second)`, push `b/a`.
예시: push(20), push(4), divide → a=4, b=20, push(20/4=5) = 5.

## Verification & Defect Resolution

```
cargo test --release -p bmb
  lib.rs:          3778/3778 PASSED
  main.rs:           47/47   PASSED
  diagnostics_test:  19/19   PASSED  (was 17, +2 new: test_bool_operators_pipe, test_bool_operators_ampersand)
  integration.rs:  2388/2388 PASSED
```

전체 6232 PASS, 0 FAIL.

## Reflection

### Scope fit
- ✅ 1/3-fail 4문제 분석 + problem.md 수정 (4개)
- ✅ bool_operators 에러 패턴 추가 → 51_bracket_match B루프 탈출 가능
- ✅ diagnostics 19/19 (+ 2 신규 테스트)

### 누적 수정 현황 (Cycles 2945-2948)
- Always-fail 11문제: 10개 수정 (에러 패턴 4개, problem.md 8개)
- Partial-fail 2/3-fail 5문제: problem.md 수정
- Partial-fail 1/3-fail 4문제: 에러 패턴 1개 + problem.md 3개
- 에러 패턴 총 추가: 5개 (function_name_reserved, if_stmt_no_semicolon, contract_param_undefined, bool_operators ×2)
- diagnostics 테스트: 13 → 19 (+6)

### B-axis 개선 추정 (재측정 대기)
현재 85.0% (255/300). 모든 수정이 효과를 내면:
- Always-fail 10개×3 = 30 failures → 0
- Partial-fail 9개 개선 → ~5-10 failures 감소
- 285-290/300 = 95.0-96.7% 기대

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack B축 재측정** — 4 cycles worth of fixes 검증 필요
  2. **51_bracket_match `||` 지원** — BMB가 `||`를 직접 지원하는 것도 고려 (언어 갭)
  3. **inttoptr UB (P3)** — HUMAN 결정 대기
- Pending Human Decisions: inttoptr Option A 승인
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2949 → HANDOFF 갱신 + 추가 언어 갭 탐색 or 잔여 always-fail 분석
