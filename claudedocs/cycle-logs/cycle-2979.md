# Cycle 2979: Claude 고루프 문제 추가 수정 — 9개 파일
Date: 2026-05-19

## Re-plan
Cycle 2978 Carry-Forward: 34_power_mod, 74_majority_element, 76_multi_function, 90_nth_prime, 79_mini_interpreter, 30_contract_chain 분석 및 수정.

## Scope & Implementation

### loop1 실패 원인 분석

| 문제 | avg loops | 실패 유형 | 근본 원인 |
|------|----------|----------|----------|
| 34_power_mod | 2.3 | D (logic) | `i=i+1`, `result=...`, `b=...`, `e=...` (set 누락) |
| 74_majority_element | 2.0 | D (logic) | `let arr: i64 = vec_new()` 잘못된 타입 + 모델이 t 무시 |
| 76_multi_function | 2.0 | C (compile) | `sum=sum+x`, `mn=x`, `mx=x` (set 누락) → 잘못된 LLVM IR |
| 90_nth_prime | 2.0 | D (logic) | `bool` 타입 + `d=d+1`, `count=count+1` (set 누락) |
| 79_mini_interpreter | 2.7 | D (logic) | `i=i+1` (set 누락) → 무한루프 → timeout → 빈 출력 + op=1에 i+=2 (잘못된 로직) |
| 30_contract_chain | 2.7 | A (contract) | bound에 `pre x >= 0` (limit >= 0 누락) → Z3 반례 |

### 수정된 파일 (9개)
| 파일 | 핵심 수정 |
|------|---------|
| 34_power_mod | CRITICAL `set` 추가, 완전한 fn main+pow_mod |
| 74_majority_element | `let arr = vec_new()` (타입 어노테이션 제거), t 무시 CRITICAL |
| 76_multi_function | CRITICAL `set` + 완전한 fn main |
| 90_nth_prime | `bool` → `i64` + `set` 추가, 완전한 구현 |
| 79_mini_interpreter | CRITICAL `set i = i + 1`, op=1에 i+=1 (not +2) 수정 |
| 30_contract_chain | bound `pre x >= 0 and limit >= 0` 강화 |
| + 이전 cycle에서 41_collatz_length | `set` + `n % 2 == 0` 짝수 체크 |
| + 이전 cycle에서 71_single_element | "first/last NOT min/max" CRITICAL |
| + 이전 cycle에서 43_sum_of_squares | `set` + 완전한 fn main |

## Verification & Defect Resolution
- `cargo test --release`: 6260/6260 PASS ✅

## Reflection
- **`set` 누락 패턴**이 압도적으로 많은 loop1 실패 원인. problem.md 코드 예시에 `set` 없는 코드가 있으면 모델이 그대로 복사
- **90_nth_prime**: `bool` 타입은 지원되지만 BMB 특유의 방식이 필요. `i64 (1/0)` 패턴이 더 안전
- **79_mini_interpreter**: op=1에서 i+=2는 잘못된 로직 (n은 op count, not token count)
- **30_contract_chain**: Z3 contracts 문제는 단순한 코드 오류가 아닌 논리적 이해 문제

## Carry-Forward
- Actionable: 남은 avg=2 문제들 확인 (76_multi_function 실제로는 linker error), 더 조사 필요한 문제들
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 남은 Claude 고루프 문제들 조사 (avg=2: multi_function 재확인) + 전체 커밋
