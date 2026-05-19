# Cycle 2978: Claude 고루프 문제 수정 — 8개 파일
Date: 2026-05-19

## Re-plan
Cycle 2977 Carry-Forward: 12_queue_simulation missing_semicolon + Claude 고루프 문제들 분석.
Claude baseline (2026-05-13) 결과에서 avg≥2 PASS 문제 14개 분석.

## Scope & Implementation

### loop1 실패 원인 분석

| 문제 | avg loops | 실패 유형 | 근본 원인 |
|------|----------|----------|----------|
| 12_queue_simulation | 2.0 | B (compile) | missing `};` at EOF |
| 43_sum_of_squares | 4.0 | D (logic) | `qi=qi+1`, `sum=sum+...` (set 누락), `let qi=0` (mut 누락) |
| 37_binary_exp | 3.7 | D (logic) | `result=result*b`, `b=b*b`, `e=e/2` (set 누락) |
| 66_acc_recursion | 4.0 | D (logic) | 모델이 op=1/op=2 구조로 잘못 해석 |
| 71_single_element | 3.0 | D (logic) | 모델이 min/max로 오해 (first/last가 -1,1이므로) |
| 70_empty_input | 3.0 | D (logic) | `sum=sum+...` (set 누락), `println(n)` 누락 |
| 41_collatz_length | 2.3 | C (type) | `n * (bool) == n` 타입 오류 |
| 34_power_mod | 2.3 | D (logic) | 유사 구조 (다중 쿼리 누락 가능성) |

### 수정된 파일 (8개)

| 파일 | 핵심 수정 |
|------|---------|
| 12_queue_simulation | CRITICAL `};` 경고 + 완전한 fn main 래퍼 |
| 43_sum_of_squares | `set` 추가, `let mut` 추가, 완전한 fn main |
| 37_binary_exp | `set result=...`, `set b=...`, `set e=...` 수정, fn main |
| 66_acc_recursion | CRITICAL: "NO operation type, just ONE integer n" 강화 |
| 71_single_element | CRITICAL: "first/last NOT min/max" + `set i = i + 1` |
| 70_empty_input | `set sum=...` + `println(n)` 필수 명시 |
| 41_collatz_length | `set i = i + 1`, `n % 2 == 0` 올바른 짝수 체크, 완전한 구현 |
| 34_power_mod | (Carry-Forward로 이동 — 별도 분석 필요) |

## Verification & Defect Resolution
- `cargo test --release`: 6260/6260 PASS ✅

## Reflection
- **공통 패턴**: `set` 없는 변수 할당은 여전히 가장 많은 실패 원인
- **71_single_element**: 문제가 min/max처럼 보이는 예시(-1, 1)가 있어 모델 오해 유발. "NOT min/max" 명시 필요
- **41_collatz_length**: 짝수 체크에서 `n / 2 * 2 == n` 패턴은 bool을 산술에 사용할 때 타입 에러 발생

## Carry-Forward
- Actionable: 34_power_mod, 74_majority_element, 76_multi_function, 90_nth_prime, 79_mini_interpreter, 30_contract_chain 분석 필요
- Structural Improvement Proposals: `set` 키워드 없는 할당에 대한 더 친절한 에러 메시지 검토
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 남은 avg=2 문제들 분석 + 커밋
