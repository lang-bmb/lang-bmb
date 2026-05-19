# Cycle 2958: 추가 problem.md 개선 + diagnostic 버그 수정
Date: 2026-05-19

## Re-plan

Cycle 2957 완료. 전체 테스트 6235 PASS. 나머지 문제 스캔:
- 1-49 범위 중 BMB 힌트 미제공 문제: 03, 06, 33, 45 → 복잡한 알고리즘 패턴 필요
- diagnostic `unknown_function` 패턴이 존재하지 않는 `i64_min`/`i64_max` 를 안내 → 즉시 수정

## Scope & Implementation

**추가 problem.md (7개)**:
- 03_merge_sort: 바텀업 반복 병합 정렬 (bottom-up iterative merge sort) + 공백 분리 출력 패턴
- 06_matrix_multiply: 평탄 행렬 인덱싱 (`a[i*n+k]`) + 삼중 루프 + 행별 출력
- 33_counting_sort: count vec (size=max_val+1) + 역방향 출력 패턴
- 45_array_compact: vec_push로 비영 수집 + count 먼저 출력
- 13_linked_list_sum: 리스트 시뮬레이션 불필요 — 단순 합산으로 충분
- 20_matrix_transpose: 평탄 행렬 전치 (`v[r*cols+c]`)
- 17_histogram: count vec + `format("{} {}", i, cnt)` 출력
- 56_char_frequency: 삽입 정렬 + 런 카운팅 + distinct count

**diagnostic 수정 (2개)**:
- `unknown_function` 패턴: `i64_min(a,b)`, `i64_max(a,b)` → `min(a,b)`, `max(a,b)` (실제 내장 함수명)
- `84_accumulator_pattern` problem.md: 동일 수정

**type annotation 개선**:
- 87_longest_increasing, 88_knapsack_01: `let arr: i64 = vec_new()` → `let arr = vec_new()` (타입 추론으로 정리, 기능 동일)

## Verification & Defect Resolution

- 전체 테스트 PASS (6235+ 테스트)
- `min(a, b)`, `max(a, b)` builtin 확인: `eval.rs` 에 `builtin_min`, `builtin_max` 등록 확인 ✅
- `i64_min`/`i64_max` 는 존재하지 않음 확인 (grep no match) ✅

## Reflection

- `unknown_function` diagnostic이 `i64_min`/`i64_max` 를 안내하면 LLM이 존재하지 않는 함수를 시도해 B-loop 유발
- Cycles 2954-2958: diagnostic 4개 수정 (bool_operators, tuple_destruct, match_wildcard, unknown_function)
- problem.md: 25+개 문제에 BMB 힌트 추가

## Carry-Forward

- Actionable: 전체 테스트 결과 최종 확인
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: Cycle 2959 → 남은 problem.md 점검 + 추가 언어 패턴 발견 탐색
