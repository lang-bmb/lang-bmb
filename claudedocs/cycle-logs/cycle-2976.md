# Cycle 2976: high-loop problem.md 개선 — palindrome_check, memory_pool
Date: 2026-05-19

## Re-plan
Cycle 2975 Carry-Forward: 고루프 문제 추가 분석. GPUStack avg=3 loops인 73_palindrome_check, 95_memory_pool 수정.

## Scope & Implementation

### 수정 대상

**73_palindrome_check (avg=3)**:
- 문제: 코드 스니펫만 있고 완전한 `fn main()` 래퍼 없음
- `let arr: i64 = vec_new()` 잘못된 타입 어노테이션
- `println(ok)` 로 끝나면 `()` 반환 → 타입 에러
- 수정: CRITICAL 경고 추가 + 완전한 `fn main() -> i64 = { ... };` 래퍼 + `let _p = vec_push(...)` 패턴

**95_memory_pool (avg=3)**:
- 문제: op=3 stats 출력이 `total_bytes\ncount` (두 줄) 대신 `total_bytes count` (한 줄) 이어야 함
- 코드 예시 없이 의사코드 주석만 있음
- 수정: CRITICAL 경고 추가 (`print(total); print_str(" "); println(count)`) + 완전한 동작 구현 제공

## Verification & Defect Resolution
- `cargo test --release`: 6260/6260 PASS ✅

## Reflection
- 두 문제 모두 "완전한 작동 예시 부재"가 공통 원인
- 95_memory_pool: two-output format은 BMB 특유의 패턴이라 CRITICAL 경고가 효과적
- 73_palindrome_check: `fn main()` return type 누락은 반복 실패 패턴

## Carry-Forward
- Actionable: 남은 고루프 문제 분석 계속 — 12_queue_simulation, 33_counting_sort, 35_sieve_primes, 55_token_count, 57_zigzag_print, 62_deep_nesting, 99_bounded_queue_contract (GPUStack avg=2), Claude high-loop 문제들
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 남은 avg≥2 loop 문제들 root cause 분석 및 수정
