# Cycle 2959: 불변 변수 버그 일괄 수정 + problem.md 계속 개선
Date: 2026-05-19

## Re-plan

Cycle 2958 완료. 추가 스캔에서 심각한 패턴 발견:
- 7개 problem.md 스케치가 `let i = 0` (mut 없음) + `i = i + 1` 사용 → 불변 변수 수정 에러
- 일부 problem.md에 완전한 알고리즘 스케치 누락 (96, 97, 98, 100번 계약 기반 문제들)
- `if_without_else_unit` 진단이 Cycle 2822 이후 잘못된 안내 제공

## Scope & Implementation

**불변 변수 버그 수정 (7개 파일)**:
- 44_euclidean_dist: `let i = 0`, `let j = 0`, `let k = 0`, `let dist = 0` → `let mut`
- 39_partial_sum_query: `let i = 0`, `let qi = 0` → `let mut` + 완전한 prefix sum 구현 추가
- 34_power_mod: `let i = 0` → `let mut i: i64 = 0`
- 35_sieve_primes: `let i = 0` → `let mut i: i64 = 0`
- 41_collatz_length: `let i = 0` → `let mut i: i64 = 0`
- 24_sorted_insert: `let i = 0` → `let mut i: i64 = 0` + `if i > 0` else { () } 패턴 수정
- 99_bounded_queue_contract: `let i = 0` → `let mut i: i64 = 0`
- 71_single_element: `let i = 0` → `let mut i: i64 = 0` + 전체 스케치 수정

**추가 problem.md (6개)**:
- 70_empty_input: BMB Notes 추가 (for 0..n handles n=0 gracefully)
- 96_safe_matrix_mul: 완전한 matrix multiply 스케치 추가
- 97_range_sum_contract: prefix sum 스케치 추가
- 98_sorted_merge_contract: two-pointer merge 스케치 추가
- 100_verified_binary_search: binary search 스케치 추가
- 52_base_convert: string building (str_concat + format) 스케치 추가

**diagnostic 수정**:
- `if_without_else_unit`: Cycle 2822의 `if cond { body }` 지원 반영 — 단순 문장에는 else 불필요, 값으로 사용 시에만 else 필요로 수정

## Verification & Defect Resolution

- `cargo test --release diagnostics` → exit code 0 ✅
- `cargo test --release` (full suite) → exit code 0 ✅ (background)

## Reflection

- `let i = 0; while i < n { ... i = i + 1 }` 패턴이 7개 파일에 퍼져 있었음
- LLM이 스케치를 그대로 복사하면 "immutable variable assignment" 에러 발생
- Cycle 2822 이후 if-without-else가 동작하는데 진단이 여전히 "add else { () }" 안내 → 수정
- 계약 기반 문제들(96-100)이 BMB 스케치 없이 방치됨

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: Cycle 2960 → 나머지 BMB 스케치 검토 + 언어 패턴 갭 탐색
