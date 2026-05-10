# Cycle 2654: M3-2 부분 시도 (bmb-algo Python 벤치) + ROADMAP 갱신
Date: 2026-05-11

## Re-plan
Cycle 2653 Carry-Forward: M3-2 시작 또는 ROADMAP 갱신. 결정 = Python 벤치 실행 + ROADMAP M5-5 진척 반영.

## Scope & Implementation

**bmb-algo Python 벤치마크 결과** (`benchmarks/bench_algo.py`):

| 함수 | BMB (μs) | Python (μs) | Speedup | 평가 |
|------|---------|------------|---------|------|
| knapsack | 4.72 | 17.63 | 3.73x | FAST |
| fibonacci(30) | 0.22 | 0.48 | 2.14x | FAST |
| prime_count(10000) | 10.11 | 331.82 | 32.81x | FAST |
| nqueens(10) | 1776.99 | 6918.36 | 3.89x | FAST |
| quicksort(15) | 1.94 | 3.37 | 1.73x | OK |
| merge_sort(15) | 2.12 | 6.54 | 3.09x | FAST |
| edit_distance | 1.38 | 8.85 | 6.39x | FAST |

**결과**: 7/7 BMB faster, 6/7 FAST (≥2x), 1/7 OK (≥1x).

**측정 방법**:
- BMB: `bmb_algo.dll` (--release + opt -O2 빌드, ctypes FFI 호출)
- Python: 순수 Python 구현 (CPython 3.12)
- 워밍업 10회 + 실행 100-500회 평균

**중요한 카비엇**:
- BMB 측정은 **ctypes FFI 오버헤드 포함** — 실제 native 성능은 더 우수
- ROADMAP Rule 4의 "C -O2 -march=native vs BMB" 정식 비교는 미완 (C 베이스라인 부재)

**ROADMAP 갱신**:
- 헤더: Cycles 2650-2654 — M5-5 dispatch + bmb-algo Python 벤치
- M5 진척바: █████░░░ → ██████░░ (M5-5 핵심 추가)
- M5-5 항목 분할: literal/alias/iter/mut ✅ + repeat/fn-return/struct-field ❌

## Verification & Defect Resolution

**`cargo test --release`**: ✅ 6210 passed (변경 없음)

**Python 벤치마크**: ✅ 7/7 BMB faster

**M3-2 잔여**:
- C 베이스라인 (knapsack/fibonacci/sieve/nqueens 등) 작성 + 동일 입력으로 측정
- 권장: 별도 cycle (3-5 cycles)에서 7개 알고리즘 C 구현 + 정식 비교

## Reflection

**Scope fit**: Python 비교 결과 공식 기록 + ROADMAP M5-5 갱신. 부분 진척.

**M3-2 진척**:
- ✅ Python 비교 결과 (이미 우수: 7/7)
- ⏳ C 비교 베이스라인 (별도 cycle)

**Latent defects**: 없음.

**Philosophy 점검**:
- bmb-algo는 도그푸딩 활동 — Python 대비 우수 결과는 가설 검증의 한 데이터포인트 ✅
- "Performance > Everything" — 7/7 우위 = 가치 증명 ✅
- 단, C 베이스라인 비교 = 완전한 가설 검증 필요. M3-2 잔여로 carry.

**Roadmap impact**:
- M3 ~90% → ~92% (Python 벤치 공식 기록 추가)
- M5-5 핵심 ✅ 명시 (4/7 매트릭스)

## Carry-Forward
- Actionable: Cycle 2655 — HANDOFF.md 종합 갱신 + 다음 사이클 (M3-2 C 베이스라인 또는 M5-4-A tuple String)
- Structural Improvement Proposals:
  - M3-2 C 베이스라인: knapsack/fibonacci/sieve/nqueens C로 작성, gcc -O2 -march=native 비교 (3-5 cycles)
  - M6 lower-time type registry (M5-5 잔여 + M4 확장 기반)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 매트릭스 4/7 ✅ 명시
- Next Recommendation: Cycle 2655 — HANDOFF 갱신 + 그 이후 cycle 2656부터 M3-2 C 베이스라인 또는 다른 영역
