# Cycle 2956: 복잡 알고리즘 + 출력 패턴 problem.md 개선
Date: 2026-05-19

## Re-plan

Cycle 2955 완료. 추가 개선 대상: 82/83/86/95 복잡 알고리즘 + 57 출력 패턴.

## Scope & Implementation

**복잡 알고리즘 힌트 추가** (5개 문제):
- 82_producer_consumer: front-pointer 큐 패턴
- 83_pipeline: abs 빌트인 + negate `0-val` + reverse two-pointer + 출력 패턴
- 86_heap_sort: insertion sort 대안 완성 코드 제공 (O(n²) for n≤1000)
- 95_memory_pool: parallel vecs (sizes + active) 패턴

**출력 패턴 힌트 추가** (1개):
- 57_zigzag_print: `print(x)` + `print_str(" ")` + `println_str("")` 패턴, 방향 체크

## Verification & Defect Resolution

problem.md 변경만 → 테스트 suite 영향 없음. diagnostics_test 22/22 PASS.

## Reflection

- 누적 개선: Cycles 2954-2956에서 총 14개 문제 힌트 추가
- 86_heap_sort에 insertion sort 대안 제공 — LLM이 복잡한 heap 구현 없이도 통과 가능
- 83_pipeline의 `-val` 문법 (BMB에서 안됨) 및 `abs()` 빌트인 존재 안내
- 82_producer_consumer front-pointer 패턴 (vec 앞에서 제거 vs 포인터 전진)

## Carry-Forward

- Actionable: 추가 문제들 (54_text_wrap, 55_token_count, 58_spiral_order, 60_checksum 등)
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: Cycle 2957 → 나머지 50-99 문제들 점검 + print_str 패턴 이슈 진단
