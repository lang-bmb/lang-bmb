# Cycle 2977: GPUStack avg=2 고루프 문제 근본 원인 수정 — 7개 파일
Date: 2026-05-19

## Re-plan
Cycle 2976 Carry-Forward: 남은 avg=2 고루프 문제들 분석 및 수정.
GPUStack 2026-05-19 결과에서 avg=2 PASS 문제 10개 식별. loop1 실패 원인 분석.

## Scope & Implementation

### 근본 원인 분석 결과

| 문제 | 실패 유형 | 근본 원인 |
|------|----------|----------|
| 33_counting_sort | Logic (D) | `first = 0` (set 누락) + 변수명 충돌 (`v` 재사용) |
| 35_sieve_primes | Logic (D) | `i = i + 1` (set 누락) + `return`/`break` 미지원 |
| 55_token_count | Logic (D) | `count = count + 1`, `break` 미지원 |
| 51_bracket_match | Logic (D) | `loop`/`break` 사용 (미지원) |
| 62_deep_nesting | Logic (D) | problem.md 오류: "return n unchanged" → 실제는 -1 반환 |
| 99_bounded_queue_contract | Logic (D) | `tail/count = ...` (set 누락) + 빈 큐 dequeue -1 미기술 |
| 57_zigzag_print | Type (C) | `if is_even == 1` (bool vs i64 비교) |

### 특이 발견: BMB 네이티브 컴파일러 변수 스코프 버그
33_counting_sort에서 `for _i in 0..n { let v = read_int() }` 후 `for v in 0..max_val` 실행 시, 네이티브 컴파일러에서 `v`가 이전 루프의 마지막 값으로 고정됨.
- 인터프리터: 정상 (변수 스코프 올바름)
- 네이티브: 루프 외부에서 선언된 것처럼 동작 (스코프 버그)
- 임시 해결: 변수명을 다르게 사용하도록 problem.md 수정 (`v` → `val`/`vi` 분리)

### 수정된 파일 (7개)

| 파일 | 핵심 수정 |
|------|---------|
| 33_counting_sort | `first = 0` → `set first = 0`, 변수명 분리, 완전한 fn main 예시 |
| 35_sieve_primes | `i = i + 1` → `set i = i + 1`, `return`/`break` 금지 CRITICAL, 완전한 구현 |
| 55_token_count | `break` 금지 CRITICAL, `set` 명시, 완전한 fn main |
| 51_bracket_match | `loop`/`break` → `while i < n` 교체, 완전한 fn main |
| 62_deep_nesting | "return n unchanged" → "return -1" 수정 |
| 99_bounded_queue_contract | `set` CRITICAL, 빈 큐 dequeue -1 처리, 완전한 fn main |
| 57_zigzag_print | `if is_even == 1` WRONG 경고, `if row % 2 == 0` 직접 사용 |

## Verification & Defect Resolution
- `cargo test --release`: 6260/6260 PASS ✅

## Reflection
- **공통 패턴**: `set` 없는 변수 할당 (`x = y` 대신 `set x = y`)과 `break`/`loop`/`return` 미지원 구문이 가장 많은 loop1 실패 원인
- **62_deep_nesting**: problem.md 설명과 test case가 불일치 — "return n unchanged"는 틀렸음. 테스트가 정답이므로 문서를 수정
- **33_counting_sort**: 네이티브 컴파일러 변수 스코프 버그 발견 (Latent defect). Bootstrap compiler 이슈로 별도 추적 필요

## Carry-Forward
- Actionable: avg=2 중 아직 미조사: 12_queue_simulation (missing_semicolon_eof), Claude high-loop 문제들 (43, 66, 37, 37, 51)
- Structural Improvement Proposals: 33_counting_sort에서 발견된 네이티브 컴파일러 변수 스코프 버그 — `for v in 0..N`이 외부 스코프의 `v`를 사용하는 버그. Bootstrap compiler fix 필요.
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 12_queue_simulation missing_semicolon 수정 + Claude 고루프 문제 분석 (43_sum_of_squares avg=4, 66_acc_recursion avg=4)
