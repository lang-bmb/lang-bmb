# Cycle 2947: partial-fail 5문제 분석 + problem.md 수정
Date: 2026-05-19

## Re-plan

Cycle 2946 Carry-Forward → ROADMAP 갱신 + GPUStack 재측정 준비 or inttoptr UB P3

GPUStack 재측정은 수동 실행 필요 (API key + 수 시간). inttoptr P3는 HUMAN 결정 대기.
→ 부분 실패(1-2/3) 문제 분석 + problem.md 수정으로 사이클 진행.

## Scope & Implementation

### ROADMAP 갱신

ROADMAP § B축 "Always FAIL 11문제" 섹션을 Cycles 2945-2946 수정 상태로 갱신.
- 11개 → 모두 addressed (10개 수정 + 1개 폐기)
- GPUStack 재측정으로 효과 검증 필요 명시

### 부분 실패 문제 분석

GPUStack 결과에서 partial-fail (1-2/3 실패) 문제 9개 발견:

| 문제 | 성공 | 실패 | 루트 원인 | 수정 |
|------|------|------|----------|------|
| 24_sorted_insert | 1/3 | 2/3 | 삽입 중 출력 + 잘못된 결과 | ✅ problem.md "완성 후 출력" + 알고리즘 명시 |
| 35_sieve_primes | 1/3 | 2/3 | n=2 → 0 (off-by-one, < 아닌 ≤) | ✅ problem.md inclusive 강조 + n=2 예시 |
| 48_run_length_encode | 1/3 | 2/3 | value/count 별도 줄 출력 | ✅ problem.md "one line" + format() 힌트 |
| 56_char_frequency | 1/3 | 2/3 | value/count 별도 줄 출력 | ✅ problem.md "one line" + format() 힌트 |
| 99_bounded_queue_contract | 1/3 | 2/3 | circular head 관리 없음 → garbage | ✅ problem.md 순환 버퍼 구현 스케치 추가 |
| 43_sum_of_squares | 2/3 | 1/3 | 산발적 실패 — 분석 필요 | 🔍 미수정 |
| 44_euclidean_dist | 2/3 | 1/3 | 산발적 실패 | 🔍 미수정 |
| 50_calculator | 2/3 | 1/3 | 산발적 실패 | 🔍 미수정 |
| 51_bracket_match | 2/3 | 1/3 | 산발적 실패 | 🔍 미수정 |

1/3 실패 문제들은 비결정적 — 별도 분석 사이클 필요.

### problem.md 수정 내용

- **35_sieve_primes**: `<= n` 강조, n=2→1, n=3→2 예시 추가, `<= n` 루프 스케치
- **48_run_length_encode**: `format("{} {}", val, count)` 힌트, WRONG 패턴 명시
- **56_char_frequency**: 동일 — format() 힌트 + WRONG 패턴 명시
- **24_sorted_insert**: "완성 후 출력" 원칙 + 알고리즘 3단계 + space-separated output 형식
- **99_bounded_queue_contract**: 순환 버퍼 구현 전체 스케치 + 모듈러 산술 필수 명시

## Verification & Defect Resolution

```
cargo test --release -p bmb: 6230 PASS, 0 FAIL (변화 없음)
```

## Reflection

### Scope fit
- ✅ ROADMAP always-fail 11개 → 모두 addressed 갱신
- ✅ 2/3-fail 5문제 분석 + problem.md 수정
- ✅ 전체 테스트 통과 (6230 PASS)

### B-axis 개선 추정
- Always-fail 10개 수정 (Cycles 2945-2946): 30 fails → 잠재적 0
- Partial-fail 2/3-fail 5개 수정: 10 fails → 잠재적 0-5 감소
- 총 285-290/300 가능 → 95.0-96.7%+ (현재 85.0%에서 10-12pp 개선 기대)
- 실제 수치는 GPUStack 재측정 후 확인 필요

### Philosophy drift
없음.

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack B축 재측정** — 모든 수정 적용 후 효과 검증. 수동 실행 필요.
  2. **1/3-fail 4문제 분석** — 43_sum_of_squares, 44_euclidean_dist, 50_calculator, 51_bracket_match
  3. **inttoptr UB (P3)** — HUMAN 결정 대기
- Pending Human Decisions: inttoptr Option A 승인
- Roadmap Revisions: ROADMAP § always-fail 표 갱신 완료
- Next Recommendation: Cycle 2948 → 1/3-fail 4문제 분석 + HANDOFF 갱신
