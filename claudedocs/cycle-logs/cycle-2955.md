# Cycle 2955: problem.md 대량 개선 — t-first + 복잡 알고리즘 힌트
Date: 2026-05-19

## Re-plan

Cycle 2954 완료. 나머지 HANDOFF 항목: GPUStack 재측정(HUMAN), inttoptr(HUMAN), 추가 패턴 발굴.
GPUStack 데이터 없이 할 수 있는 최선: 기존 problem.md에서 BMB 힌트 없는 문제들 개선.
SCOPE: 50-99번 문제 중 BMB 힌트 없는 것들 → t-first 패턴 + 알고리즘 접근법 추가.

## Scope & Implementation

**t-first 패턴 추가** (5개 문제):
- 65_chain_calls: `t` 루프 + `x*x*2+1` 계산 스케치
- 67_nested_loops: `t` 루프 + 3중 중첩 for 스케치
- 72_alternating: `t` 루프 + `n % 2` 패턴
- 75_longest_plateau: `t` 루프 + vec/plateau 알고리즘 스케치
- 90_nth_prime: `t` 루프 + trial division + nth_prime 스케치 (완성 코드)

**복잡 알고리즘 힌트** (4개 문제):
- 80_array_of_arrays: flat 1D vec + `row*cols+col` 인덱싱 (nested vec 불필요 안내)
- 93_sparse_array: parallel arrays 패턴 (BMB에 int→int hash map 없음 명시)
- 94_lru_simulate: vec으로 LRU 시뮬레이션 + shift 패턴 스케치
- 51_bracket_match (Cycle 2954): 스택 스케치 이미 추가

## Verification & Defect Resolution

```
cargo test --release -p bmb --test diagnostics_test: 22/22 PASSED
```
(problem.md는 컴파일 불필요, 테스트 suite 영향 없음)

## Reflection

- 스코프 적합도: ✅ 9개 problem.md 개선 (65/67/72/75/90/80/93/94 + 51)
- 영향 예상: LLM이 BMB-특이적 패턴 (parallel arrays, flat matrix, t-first) 학습
- 철학: problem.md 힌트는 LLM에게 "BMB에서 어떻게" 보여주는 핵심 도구
- 미비: 95_memory_pool, 82_producer_consumer, 83_pipeline 등 아직 힌트 없음

## Carry-Forward

- Actionable: 추가 problem.md 개선 (82/83/95/86 heap_sort 등)
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정, inttoptr Option A
- Roadmap Revisions: None
- Next Recommendation: Cycle 2956 → 86_heap_sort, 95_memory_pool, 82/83 힌트 추가 + 잔여 패턴 탐색
