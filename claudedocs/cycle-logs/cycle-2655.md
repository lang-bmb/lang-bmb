# Cycle 2655: M3-2 정식 측정 시도 — knapsack BMB vs C
Date: 2026-05-11

## Re-plan
Cycle 2654 Carry-Forward: HANDOFF 갱신 또는 M3-2 C 베이스라인. M3-2 우선 — knapsack 1개 정식 측정.

## Scope & Implementation

**측정 조건** (Rule 4 준수):
- BMB: `bmb build --release` + opt -O2 (기본 파이프라인)
- C: `gcc -O2 -march=native`
- 워크로드: N=2000, CAP=5000, **ITERS=500** (startup overhead amortize)

**소스**: `ecosystem/benchmark-bmb/benches/compute/knapsack/{bmb,c}/main.{bmb,c}` (ITERS=500 수정 변형)

**5-run wall-clock 결과**:

| 시스템 | Run 1 | Run 2 | Run 3 | Median |
|--------|-------|-------|-------|--------|
| BMB | 1417 ms | 1393 ms | 1450 ms | 1417 ms |
| C | 1198 ms | 1140 ms | 1152 ms | 1152 ms |

**비율**: BMB / C ≈ 1.23x (BMB **slower**)

**체크섬 동일**: 17874646 (정확성 ✅)

**중요 발견 — README 주장과 모순**:
- `ecosystem/bmb-algo/src/lib.bmb` 헤더: "knapsack: 6.8x faster than C"
- 본 cycle 정식 측정: knapsack BMB 1.23x slower than C
- 가능한 원인:
  - 비공식 결과의 측정 환경/방법 다름
  - 컴파일러 변경으로 인한 회귀
  - bmb-algo 라이브러리 (FFI export)와 standalone 빌드 차이

## Verification & Defect Resolution

**`cargo test --release`**: ✅ 6210 passed (변경 없음)

**M3-2 진척**:
- ✅ Python 비교 (Cycle 2654, 7/7 BMB faster)
- 🔄 C 비교 1/7 완료 (knapsack 1.23x slower)
- ⏳ C 비교 6/7 잔여 (lcs, floyd, fibonacci, sieve, nqueens, edit_distance)

**P축 영향**:
- Tier 1 ≤1.05x 목표 — knapsack 1.23x = **NOT PASS** (Tier 1 정의에 따라)
- 그러나 P축 표 (ROADMAP § 5)는 16/16 ≤1.05x로 명시 — knapsack은 별도 측정 (M3 showcase)
- ROADMAP Tier 1 16개 도메인 벤치 ≠ M3 showcase 7개

## Reflection

**Scope fit**: knapsack 1개 정식 측정 ✅. README 주장 검증 갭 발견.

**측정 방법 신뢰성**:
- wall-clock 측정 (process-level, startup overhead 포함)
- in-process timing 측정으로 더 정확한 결과 가능 (별도 cycle)
- 5-run min-of-3 또는 median 권장 (현재 3-run median 사용)

**Latent defects**:
- README "6.8x faster than C" 주장 = unverified (검증 또는 수정 필요)
- bmb-algo의 7개 알고리즘 정식 C 비교 = M3-2 잔여

**Philosophy 점검**:
- "측정 없는 성능 주장은 허용하지 않음" → 현재 README 주장 = unverified ❌
- 본 cycle은 *정직한 측정*을 수행 — 결과가 가설(빠름)에 부합하지 않더라도 기록 ✅
- Decision Framework 1순위 (언어 스펙) — 1.23x 차이는 **컴파일러 최적화 영역** (3순위)으로 추정. lib.bmb 알고리즘 자체는 동일 ✅

**Roadmap impact**:
- M3-2 부분 완료. README "6.8x faster" 주장 검증 = 명시적 후속 작업 필요
- 잔여 C 비교 6개 (lcs, floyd, etc) — M3-2 잔여 작업

## Carry-Forward
- Actionable: Cycle 2656 — 잔여 5 cycles 남음. 후보:
  - M3-2 잔여 6개 알고리즘 C 비교 (~3-4 cycles)
  - README 주장 검증 (in-process timing 추가)
  - HANDOFF 종합 갱신
- Structural Improvement Proposals:
  - bmb-algo README 주장 검증 — 측정 갭 해소 또는 결과 갱신
  - in-process timing 인프라 (`time_ns()` 사용) → 정확도 향상
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M3-2 부분 진척 명시 필요
- Next Recommendation: Cycle 2656 — fibonacci/sieve/nqueens 빠른 3개 추가 측정 (각 1개씩 quick) + HANDOFF 갱신
