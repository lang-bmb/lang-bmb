# Cycle 2695: Mandelbrot inproc 측정
Date: 2026-05-11

## Re-plan
Carry-Forward (Cycle 2694): mandelbrot inproc 측정. Trigger 없음.

## Scope & Implementation

### inproc 변환 신규 작성
- `bmb/main_inproc.bmb` — BMB 버전, time_ns() harness
- `c/main_inproc.c` — Clang/GCC 비교 베이스, QueryPerformanceCounter

### 측정 환경 (동일)
- BMB: `--release`
- Clang: `-O3 -march=native`
- GCC: `-O3 -march=native`

### Mandelbrot inproc 결과 (size=2000, max_iter=100)

| Runtime | min (μs) | max (μs) | median (μs) | vs BMB |
|---------|----------|----------|-------------|--------|
| **BMB** | 145202 | 150172 | 147972 | 1.00x |
| Clang -O3 -march=native | 135553 | 142838 | 137626 | BMB 1.075x slower |
| GCC -O3 -march=native | 131746 | 138175 | 133332 | BMB 1.110x slower |

checksum 모두 3612362 일치.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| BMB 빌드 (release) | ✅ |
| C clang/gcc 빌드 | ✅ |
| checksum 일치 (3 × 5 run) | ✅ 3612362 |

결함: 없음.

## Reflection

**측정 의의**:
- mandelbrot은 fixed-point 정수 산술 + 이중 while 루프
- BMB ≈ Clang (1.075x) — LLVM 백엔드 한계 내 parity
- BMB ≈ GCC (1.110x) — knapsack/nqueen의 큰 갭(1.27-1.34x) 대비 양호
- 두 baseline 모두 ≤1.15x 범위 — Tier 1 측정 정합성 검증

**도그푸딩**:
- inproc timing 패턴 (Cycle 2685 가이드) 재현성 확인 — 변환 작업 5분 이내
- 표준화의 가치 입증

**Roadmap impact**:
- Tier 1 inproc 측정 2/N 완료 (knapsack + mandelbrot)
- 누적 측정 표:
  - knapsack: BMB vs clang 0.149x (BMB faster), vs gcc 1.39x
  - mandelbrot: BMB vs clang 1.075x, vs gcc 1.110x
  - fibonacci: BMB vs clang 1.04x, vs gcc 0.38x (BMB 2.6x faster)
  - nqueen: BMB vs clang 1.06x, vs gcc 1.27x

**핵심 관찰**: BMB는 Clang과 매우 근접 (LLVM parity). GCC와의 갭은 도메인별 양극 (fibonacci BMB faster, knapsack/nqueen BMB slower) — LLVM 백엔드 특성.

## Carry-Forward
- Actionable: Cycle 2696 — 측정 데이터 종합 + ROADMAP § 5 갱신
- Structural Improvement Proposals:
  - 측정 자동화 스크립트 (`bench-inproc-suite.sh`) — 모든 변환된 bench 일괄 실행
  - clang knapsack outlier 분석 (장기 cycle)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음 (Cycle 2696에서 ROADMAP § 5 갱신 예정)
- Next Recommendation: Cycle 2696 — Tier 1 측정 종합 정리 + ROADMAP § 5 갱신
