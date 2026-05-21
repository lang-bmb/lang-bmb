# Cycle 2694: Knapsack inproc 측정 (Phase 2 진입)
Date: 2026-05-11

## Re-plan
Carry-Forward (Cycle 2693): Phase 2 — Tier 1 inproc 측정. Trigger 없음.
Discovery: `main_inproc.bmb` + `main_inproc.c` 이미 존재 (Cycle 2661-2662 시점) — 측정만 미실행.

## Scope & Implementation

ROADMAP M5 매트릭스에 M5-5g 추가 (set field-index + nested chain).

### 측정 환경

| 컴파일러 | 플래그 |
|---------|--------|
| BMB | `--release` (target/release/bmb.exe) |
| Clang | `-O3` / `-O3 -march=native` |
| GCC | `-O3` / `-O3 -march=native` |

### Knapsack inproc 결과 (50 iter, n=2000, cap=5000)

| Runtime | min (μs) | max (μs) | median (μs) | vs BMB |
|---------|----------|----------|-------------|--------|
| **BMB** | 162168 | 179633 | 171009 | 1.00x |
| Clang -O3 | 1121859 | 1149912 | 1146804 | **BMB 6.71x faster** |
| Clang -O3 -march=native | 1129271 | 1144354 | 1137806 | **BMB 6.65x faster** |
| GCC -O3 | 123738 | 134173 | 127600 | BMB 1.34x slower |
| GCC -O3 -march=native | 122583 | 126935 | 123361 | BMB 1.39x slower |

checksum 모두 1776594 일치 (correctness OK).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| BMB 빌드 (release) | ✅ |
| C clang -O3 빌드 | ✅ |
| C gcc -O3 빌드 | ✅ |
| checksum 일치 (3 runtimes × 5 runs) | ✅ 1776594 모두 동일 |

결함: 없음 (BMB 정확성). 단, **외부 관찰**: clang -O3가 knapsack을 GCC 대비 9x 느리게 컴파일 — clang 자체의 최적화 갭 (BMB의 강점이 아니라 clang의 약점). README "6.8x faster than C" 주장은 clang baseline에서만 성립.

## Reflection

**핵심 발견**:
- BMB knapsack은 GCC -O3 대비 ~1.34-1.39x slower — fibonacci(0.38x)/nqueen(1.27x)와 비슷한 LLVM vs GCC 갭
- Clang -O3가 knapsack을 매우 느리게 — 별도 LLVM 최적화 갭 (BMB와 동일 백엔드 사용)
- BMB ≈ clang 수준 (LLVM parity 가설 일관) — 정확성보다 측정-방법론 강조 필요

**철학 점검**:
- Performance > Everything ✅ — GCC 갭은 LLVM 백엔드 한계 (외부 요인, BMB ≈ Clang 일관성)
- README baseline 라벨 (clang vs gcc) 명시 필요 — HUMAN 결정 사항 데이터 보강

**Roadmap impact**:
- Tier 1 inproc 측정 1/N 완료 (knapsack)
- 다음: mandelbrot inproc 측정 (Cycle 2695)
- Latent: clang knapsack IR 갭 분석 (별도 cycle, 장기)

## Carry-Forward
- Actionable: Cycle 2695 — mandelbrot inproc 측정
- Structural Improvement Proposals:
  - benchmark 비교 자동화 스크립트 (`bench-compare.sh` median + 분산)
  - clang -O3 knapsack IR vs BMB IR diff 분석
- Pending Human Decisions:
  - README "6.8x faster than C" 라벨 (clang 기준 명시)
- Roadmap Revisions: ROADMAP M5 매트릭스에 M5-5g 추가 ✅ (이 사이클)
- Next Recommendation: Cycle 2695 — mandelbrot inproc + JSON parse inproc (가능시)
