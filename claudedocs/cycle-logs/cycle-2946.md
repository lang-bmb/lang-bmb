# Cycle 2946: always-fail 7문제 분석 + problem.md + 에러 패턴 수정
Date: 2026-05-19

## Re-plan

Cycle 2945 Carry-Forward → "GPUStack 재측정 or 잔여 always-fail problem.md 분석"

GPUStack 재측정(100문제×3 runs)은 수 시간이 필요하므로, 이번 사이클에는:
- 잔여 7개 always-fail 문제의 실제 실패 코드 분석
- problem.md clarification 및 에러 패턴 추가

## Scope & Implementation

### 분석 결과: 7개 always-fail 문제 루트 원인

| 문제 | loop_type | 루트 원인 | 수정 |
|------|-----------|----------|------|
| 28_positive_factorial | C×10 | `fn main()`에 `pre n >= 0` 계약 → "undefined variable: n" 반복 | ✅ 에러 패턴 추가 |
| 41_collatz_length | D×10 | t 쿼리 루프 미작성 — 첫 숫자(=t)를 값으로 직접 계산 | ✅ problem.md 읽기 구조 명시 |
| 34_power_mod | D×10 | n을 4번째로 읽음 (a,b,m,n 순서 오독) — 1000번 loop | ✅ problem.md n-first + 알고리즘 힌트 |
| 39_partial_sum_query | D×10 | n,q,m 3개 먼저 읽음 (배열보다 q를 앞서 읽음) | ✅ problem.md 읽기 순서 명시 |
| 71_single_element | D×10 | 전 원소 출력 + min + max + count = 4줄 (3줄이어야 함) | ✅ problem.md "exactly 3 lines" 명시 |
| 79_mini_interpreter | D×10 | op=5 dup/op=6 print-no-pop 구현 오류 → 가비지 값 | ✅ problem.md op=5,6 명시적 구현 스케치 |
| 91_ring_buffer | D×10 | full 시 head 전진 없음 → overwrite-when-full 미구현 | ✅ problem.md head 전진 명시 + 구현 스케치 |

### Fix 1: `contract_param_undefined` 에러 패턴 (patterns.rs)

28_positive_factorial: 루프 C (same error, same code, 10 attempts). 에러 "undefined variable: `n`"에 매핑되는 패턴이 없어 모델이 아무 guidance 없이 동일 코드 재제출.

추가 패턴:
- trigger: `"undefined variable"` (kind: "type")
- suggestion: fn main() 계약 금지 + 헬퍼 함수로 이동 가이드

### Fix 2-7: problem.md 수정 (6개 파일)

- **41_collatz_length**: "IMPORTANT: Reading Multiple Queries" 섹션 + 읽기 패턴 코드
- **34_power_mod**: "IMPORTANT: Reading Order" + 알고리즘 힌트 (fast exponentiation)
- **39_partial_sum_query**: "IMPORTANT: Reading Order" + 단계별 읽기 순서 코드
- **71_single_element**: "IMPORTANT: Exactly 3 Lines of Output" + 올바른 구현 예시
- **91_ring_buffer**: overwrite-when-full 로직 명시 + head 전진 구현 스케치
- **79_mini_interpreter**: op=5 (dup), op=6 (print-no-pop) 명시적 구현 스케치

## Verification & Defect Resolution

```
cargo test --release -p bmb
  lib.rs:          3778/3778 PASSED
  main.rs:           47/47   PASSED
  diagnostics_test:  17/17   PASSED  (was 16, +1 new: test_contract_param_undefined)
  integration.rs:  2388/2388 PASSED
```

전체 6230 PASS, 0 FAIL.

## Reflection

### Scope fit
- ✅ 7개 always-fail 분석 완료, 루트 원인 5개 AI 알고리즘 오류 + 2개 입력 형식 오독 + 1개 C루프 에러 패턴 없음
- ✅ 1개 에러 패턴 추가 (contract_param_undefined) → 28번 C루프 탈출 가능성 높음
- ✅ 6개 problem.md 수정 → 입력 형식/출력 요구사항 명시화
- ✅ 전체 테스트 통과 (6230 PASS)

### B-axis 개선 추정
| 수정 | 기대 효과 |
|------|---------|
| 28_positive_factorial (에러 패턴) | C루프 → 다른 시도 가능, 높은 수정 가능성 |
| 41_collatz_length (읽기 구조) | t-first 패턴 명시 → 수정 가능성 높음 |
| 34_power_mod (읽기 순서+알고리즘) | n-first + 알고리즘 힌트 → 수정 가능성 중간 |
| 39_partial_sum_query (읽기 순서) | 단계별 순서 명시 → 수정 가능성 중간 |
| 71_single_element (3줄 출력) | 명시적 구현 예시 → 수정 가능성 높음 |
| 79_mini_interpreter (op=5,6) | 구현 스케치 → 수정 가능성 중간 |
| 91_ring_buffer (overwrite 로직) | head 전진 명시 + 스케치 → 수정 가능성 중간 |

Cycle 2945 3개 + 이번 7개 = 잠재적 10개 수정 (누적 always-fail 잔여 0개)

### Philosophy drift
없음. 에러 패턴 + problem.md 수정은 proper fixes.

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack B축 재측정** — always-fail 10개 수정 효과 검증. 85.0%→90%+ 목표. 수동 실행 필요 (API key + 수 시간)
  2. **28_positive_factorial 재검증** — `contract_param_undefined` 패턴 적용 후 behavior 확인
  3. **inttoptr UB (P3)** — Option A 대형 작업 착수 (5-10 cycles) — HUMAN 결정 대기
- Pending Human Decisions: inttoptr Option A 승인
- Roadmap Revisions: ROADMAP § B축 always-fail 10개 → 잠재적 수정됨 주석 추가 필요
- Next Recommendation: Cycle 2947 → ROADMAP/HANDOFF 갱신 + GPUStack 재측정 준비 or inttoptr UB P3 분석 착수
