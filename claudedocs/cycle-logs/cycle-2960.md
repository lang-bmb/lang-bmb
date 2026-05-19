# Cycle 2960: problem.md 광범위 개선 — 스캔 완결
Date: 2026-05-19

## Re-plan

Cycle 2959 완료. 나머지 problem.md 파일 전체 스캔 — 힌트 미제공 + 잘못된 패턴 식별.

## Scope & Implementation

**새 BMB 스케치 추가 (12개 파일)**:
- 36_array_rotation: k번 왼쪽 회전 = 인덱스 k..n 후 0..k 출력
- 54_text_wrap: 행 너비 w로 합산
- 96_safe_matrix_mul: 완전한 삼중 루프 + 공백 분리 출력 (already done previous cycle — confirmed)
- 97_range_sum_contract: prefix sum 스케치
- 98_sorted_merge_contract: 2-포인터 merge
- 100_verified_binary_search: binary search 스케치
- 07_max_subarray: Kadane's (첫 요소로 초기화 — all-negative 처리)
- 09_insertion_sort: vec_set 기반 삽입 정렬 + 출력 패턴
- 10_reverse_array: 역방향 출력 패턴
- 11_stack_operations: vec_push/vec_pop/vec_len 스택 구현
- 12_queue_simulation: `vec_remove(q, 0)` 로 dequeue 구현 발견
- 52_base_convert: str_concat + format으로 숫자 문자열 구성

**주요 발견**:
- `vec_remove(v, idx)` 가 실제로 존재함 (Cycle 2853 추가) → queue dequeue에 직접 활용 가능
- `max(a, b)`, `min(a, b)` builtin 확인 → Kadane's에서 직접 사용

## Verification & Defect Resolution

진단 테스트 exit 0 ✅
전체 테스트 exit 0 ✅ (background)

## Reflection

- 총 Cycles 2954-2960: ~45개 problem.md 파일 개선, 5개 diagnostic 수정
- `vec_remove`, `min/max` builtin 재발견 → problem.md 스케치에 반영
- 잘못된 스케치가 LLM의 B-loop 직접 유발 — 수정으로 성공률 직접 향상 가능

## Carry-Forward

- Actionable: 전체 테스트 최종 확인 + commit
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: Cycle 2961 → 마지막 스캔 (19_prefix_sum, 19-30 범위) + commit 준비
