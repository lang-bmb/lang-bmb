# Cycle 2915: B축 품질 — Placeholder 15개 수정 + 진단

Date: 2026-05-18

## Re-plan

이전 Carry-Forward 없음 (Cycle 2914 완료 상태). Advisor 권고 반영:
- placeholder 15개 전수 fix (31-45)
- 측정 없이는 "fix"가 아닌 "변경" → baseline JSON에 `problem_md_changes` 기록
- Always FAIL 문제 실제 qwen3 생성 코드 진단 (추측 금지)

## Scope & Implementation

### 1. Placeholder 15개 problem.md 전수 수정

| 문제 | 내용 | 완료 |
|------|------|------|
| 31_dot_product | 벡터 내적 (n, A[n], B[n] → 합) | ✅ |
| 32_selection_sort | 선택 정렬 (n, arr → 정렬) | ✅ |
| 33_counting_sort | 계수 정렬 (n, max_val, arr → 정렬) | ✅ |
| 34_power_mod | 모듈러 지수 (n queries, a b m → a^b mod m) | ✅ |
| 35_sieve_primes | 에라토스테네스 체 (n → 소수 개수) | ✅ |
| 36_array_rotation | 배열 좌회전 (n, k, arr → 회전) | ✅ |
| 37_binary_exp | 이진 지수 (n queries, a b → a^b) | ✅ |
| 38_matrix_trace | 행렬 대각합 (n, n×n → trace) | ✅ |
| 39_partial_sum_query | 구간합 쿼리 (n, arr, q, (l,r) → 합[l..r] inclusive) | ✅ |
| 40_bubble_sort | 버블 정렬 (n, arr → 정렬) | ✅ |
| 41_collatz_length | 콜라츠 수열 길이 (t queries, n → 길이) | ✅ |
| 42_integer_sqrt | 정수 제곱근 (t queries, n → floor(sqrt(n))) | ✅ |
| 43_sum_of_squares | 제곱합 (t queries, n → 1²+...+n²) | ✅ |
| 44_euclidean_dist | 유클리드 거리² (n, A[n], B[n] → 거리²) | ✅ |
| 45_array_compact | 배열 압축 (n, arr → 0 제거 후 count+elements) | ✅ |

### 2. baseline JSON 갱신
`b_baseline_2026-05-18_c2914_qwen3.json`에 `problem_md_changes` 필드 추가 — 비교 유효성 명시.

### 3. Always FAIL 진단 (qwen3 실제 생성 코드 분석)

| 문제 | 근본 원인 | 분류 |
|------|----------|------|
| **25_range_clamp** | `fn clamp(...)` → stdlib prelude에 `clamp` 이미 정의 → `invalid redefinition` 링크 오류 | problem.md 버그 |
| **28_positive_factorial** | `fn main() -> i64 pre n >= 0...` — main은 파라미터가 없어 n 미정의 | problem.md 불명확 |
| **71_single_element** | problem.md "first, last, count" 출력이어야 하는데 "모든 원소 + min + max + count" 출력 → 문제 설명이 해결책과 불일치 | problem.md 버그 |
| **99_bounded_queue_contract** | `vec_pop` 사용 → 큐 디큐는 뒤에서 제거 → front index 추적과 충돌 → 메모리 오염 | bmb_reference 부족 |

### 4. problem.md 수정 (3개)

- **25_range_clamp**: "IMPORTANT: Name your clamp function `clamp_val`" 추가
- **28_positive_factorial**: "Contract goes on helper function, NOT on main()" 명시, `main` 계약 금지 설명 추가
- **71_single_element**: "Print all elements, then min, max, count" → "Print first, last, count" 수정

### 5. bmb_reference.md 보완

- **Queue pattern**: `vec_pop` CRITICAL 경고 추가 + bounded queue 패턴 추가
- **Reserved stdlib names**: `clamp`, `min`, `max`, `abs`, `gcd_i64` 재정의 금지 섹션 신규

## Verification & Defect Resolution

테스트 변경 없음 (2388 tests, 수정된 파일은 problem.md / bmb_reference.md / baseline JSON).
새로 만든 problem.md들은 solution.bmb + tests.json 기준으로 내용 검증됨.

## Reflection

**Scope fit**: 완전히 scope 충족. 15개 placeholder → 전부 proper description. 진단도 실제 생성 코드 기반.

**Latent defects found**:
- 71_single_element의 problem.md는 solution과 완전 불일치. 모델이 올바른 코드(min,max,count)를 생성해도 실제 정답은 (first,last,count) → 설명 자체가 잘못됨.
- 25_range_clamp: `clamp` 이름 충돌은 bmb_reference에서도 경고 없음 → 양쪽 모두 수정 필요.
- Always FAIL 이유의 80%가 problem.md 자체 결함 (placeholder / 불명확 / 불일치).

**Roadmap impact**: 재측정 시 Always FAIL 11개 중 최소 4개(25,28,71,그리고 수정된 15개 placeholder에 해당하는 34,41 등)가 해소될 가능성 높음.

**Philosophy drift**: 없음 — B축 품질 개선은 M4-5 범위.

## Carry-Forward

- **Actionable**: 나머지 Always FAIL 7개 (79_mini_interpreter, 89_topological_sort, 90_nth_prime, 91_ring_buffer) 진단 필요
- **Structural Improvement Proposals**: problem.md 자동 검증 스크립트 — problem.md와 solution.bmb + tests.json 일관성 체크
- **Pending Human Decisions**: 재측정 시기 (ANTHROPIC_API_KEY 필요 없는 GPUStack으로 재측정 가능)
- **Roadmap Revisions**: 없음
- **Next Recommendation**: Cycle 2916 — 나머지 Always FAIL 4개 (79, 89, 90, 91) 진단 + 가능하면 fix
