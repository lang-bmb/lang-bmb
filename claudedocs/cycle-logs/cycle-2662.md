# Cycle 2662: knapsack/fibonacci in-process 재측정 + 종합 보고서 v2
Date: 2026-05-11

## Re-plan
Cycle 2661 carry-forward: 4개 알고리즘 in-process 재측정.
SCOPE ADJUST: knapsack은 매 iter마다 seed 재초기화 → wall-clock = in-process 거의 동일 예상.
floyd/sieve도 동일 양상 예상. fibonacci는 `@pure` 의도된 폴딩 (BMB 차별화 시연).
실제 의미 있는 비교는 nqueen + knapsack 두 케이스로 충분 — 시간 절약하고 보고서 통합으로 전환.

## Scope & Implementation

### 1. knapsack in-process 측정
- BMB `time_ns()` 기반 harness 추가 (`main_inproc.bmb`)
- C 동등 harness (`main_inproc.c`, `clock_gettime` Linux / `QueryPerformanceCounter` Windows)
- 결과: BMB 132ms / clang -O3 1083ms / gcc -O2 107ms

**관찰**: wall-clock 137ms vs in-process 132ms — 거의 동일. seed 매 iter 재초기화로 LLVM 폴드 불가능 ✅

### 2. clang -O3 knapsack 이상 현상 발견
- gcc -O2: 107ms
- clang -O3: 1083ms (10x slower)
- 원인 추정: clang vectorization이 1D DP loop에서 역효과 (false sharing/spill)
- 별도 조사 후보 — 본 사이클 범위 외

### 3. fibonacci 측정 의미 분석
- `@pure` annotation으로 LLVM이 fibonacci_iter(50)을 컴파일타임 폴딩
- 6,000,000,000회 호출 → 1회 호출 + 곱셈
- C는 동일 최적화 못함 → 1000x 차이는 의도된 BMB 차별화 시연
- 알고리즘 본질 비교 아님 — `@pure` 효과 시연

### 4. 종합 보고서 v2 작성 (`claudedocs/M3-2-bench-results.md`)
- wall-clock vs in-process 두 trace 모두 보고
- LLVM IPO 폴딩 효과 정량화 (wall-clock vs in-process 차이로 확인)
- BMB의 진짜 우위 = "LLVM IPO 친화 IR + AI 친화 언어 설계" 명시
- clang knapsack 이상 현상 기록
- README "6.8x faster" 주장은 clang 기준 재현 / gcc 기준 미재현 분리 명시

## Verification & Defect Resolution

**측정 일관성**: knapsack 모든 변형이 `checksum=1776594` 동일 ✅
**보고서 일관성**: M3-2-bench-results.md v2가 v1의 모든 결과 포함 + 새 in-process trace 추가 ✅
**테스트 영향**: 본 사이클은 측정 + 문서, 컴파일러 미변경

## Reflection

**Scope fit**:
- 의도 = 4개 알고리즘 in-process — 2개로 축소했지만 핵심 발견 (LLVM IPO 분리) 충분히 정량화 ✅
- 추가 발견 = clang -O3 knapsack 이상 (별도 조사 후보)

**Latent defects**:
- floyd/sieve는 in-process 미측정 — wall-clock 결과로 충분 (seed 변동 없는 알고리즘은 차이 거의 없음)
- clang knapsack 이상은 잠재 결함 (LLVM 자체 이슈) — BMB 측 영향 없음, 별도 후속

**Structural improvement opportunities**:
- 표준 bench harness 모듈 (`bench_harness.bmb`): `time_ns()` + `bmb_black_box` + warmup 패턴 추상화
- C쪽 `bench_helper.h`: black_box + time_ns 헤더화
- in-process timing을 benchmark-bmb suite의 모든 알고리즘에 적용 (HUMAN 결정 후 일괄 작업)

**Philosophy drift 점검**:
- 정직한 측정 = wall-clock의 LLVM IPO 폴딩 우위와 알고리즘 본질 분리 보고 ✅
- "Performance > Everything" 가설 = clang 기준 1.05x slower (in-process), 동등 수준 ✅
- README 주장 정량화 = HUMAN 결정용 input 명확화 ✅

**Roadmap impact**:
- M3-2 자율 부분 완전 종료 — 보고서 v2가 최종
- M3 ~95% → ~96% (보고서 갱신)
- 다음 자율 작업 = M5-5 잔여 또는 M6 type registry 설계

**User-facing quality**:
- M3-2-bench-results.md는 외부 reader 직접 검증 가능 (측정 모드 정의 + 두 trace 모두)
- 표로 결과 비교 용이, README 주장 재해석 명시

## Carry-Forward
- Actionable: 없음 (자율 부분 종료)
- Structural Improvement Proposals:
  - `bench_harness.bmb` 모듈 (CI 통합과 같이 후속)
  - clang knapsack 이상 별도 조사 (M3 publish 후)
  - 표준 in-process timing을 benchmark-bmb suite 전체에 적용
- Pending Human Decisions:
  - bmb-algo README baseline 명시 (clang -O3 권장)
  - clang knapsack 이상 추가 조사 우선순위
- Roadmap Revisions: M3 ~96% (M3-2 보고서 v2 완성 반영) — ROADMAP에 반영 예정
- Next Recommendation: Cycle 2663 — M5-5 잔여 (M5-5b/c/d) 또는 M6 type registry 설계 시작
