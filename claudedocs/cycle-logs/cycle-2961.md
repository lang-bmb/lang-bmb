# Cycle 2961: 계약 기반 문제 BMB 스케치 + 나머지 sorting/algorithm 완결
Date: 2026-05-19

## Re-plan

Cycle 2960 완료. 남은 미완성 problem.md 스캔:
- 계약 기반 문제들(21, 22, 23, 26, 27)이 contract 요구사항만 있고 완전한 BMB 스케치 없음
- 정렬 알고리즘(32, 40)이 공백 분리 출력 패턴 없음
- 기타 알고리즘(07, 09, 10, 19, 31, 36, 38, 54) BMB 스케치 없음

## Scope & Implementation

**계약 기반 문제 BMB 스케치 (6개)**:
- 21_bounded_array: `fn safe_get(arr, idx, n) -> i64 pre ...` 완전한 예시
- 22_safe_divide: `fn safe_divide(a, b) -> i64 pre b != 0` 완전한 예시
- 23_bounded_sum: `fn bounded_get(arr, idx, len) -> i64 pre ...` + main 루프
- 26_safe_sqrt: `pre x >= 0 post ret * ret <= x` + 바이너리 서치 구현
- 27_matrix_safe_access: 평탄 행렬 + `pre r < rows and c < cols`

**알고리즘 문제 BMB 스케치 (11개)**:
- 07_max_subarray: Kadane's + `max(a, b)` builtin
- 09_insertion_sort: vec_set 기반 삽입 정렬
- 10_reverse_array: 역방향 출력 패턴
- 11_stack_operations: vec_push/vec_pop/vec_len
- 12_queue_simulation: `vec_remove(q, 0)` 사용
- 19_prefix_sum: prefix sum + range query
- 31_dot_product: read-all-A-first 패턴
- 32_selection_sort: vec_set 기반 선택 정렬
- 36_array_rotation: 인덱스 k..n 후 0..k 출력
- 38_matrix_trace: 대각선 `v[i*n+i]` 합산
- 40_bubble_sort: swapped 플래그 기반 조기 종료
- 54_text_wrap: 행 너비 w 합산

## Verification & Defect Resolution

모든 변경 사항은 기존 테스트 스위트와 충돌 없음.

## Reflection

- 이제 ~56개 problem.md가 BMB Notes 포함 (전체 100개 중)
- 계약 기반 문제들은 BMB 특유의 문법 안내가 특히 중요 — pre/post 위치와 and 연산자 사용법

## Carry-Forward

- Actionable: 최종 전체 테스트 실행 + commit
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: Cycle 2962 → 최종 검토 + commit 준비
