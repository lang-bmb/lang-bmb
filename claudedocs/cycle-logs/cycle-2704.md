# Cycle 2704: M4-9 clang knapsack outlier IR diff
Date: 2026-05-11

## Re-plan
인계받은: knapsack에서 clang -O3가 BMB 대비 6.7x 느린 이유. 1 cycle 한정 (advisor 권고). Trigger ⚪ NONE.

## Scope & Implementation

### IR 분석
- Clang -O3: knapsack inline → outer unroll-by-2, inner loop는 select-phi 기반 dynamic bound + smax intrinsic + **unconditional store**
- BMB --release + opt -O2: 단일 inner loop, **conditional store** (then/merge), 정적 j > w[i] bound

### Root cause
**Clang의 두 anti-patterns**:
1. `if (take > dp[j]) dp[j] = take;` → `dp[j] = smax(take, dp[j])` (unconditional store) — store buffer pressure + cache line write
2. select-phi dynamic loop bound — loop termination dependency chain 길어짐, ILP 감소

GCC는 conditional store 유지 → BMB와 유사 패턴 (124 vs 171 μs)

### 결론
BMB가 빠른 게 아니라 **clang -O3 outlier**. BMB의 단순 lowering이 LLVM opt가 잘못된 transformation을 회피하게 함 (운 좋은 효과).

### Issue 등록
`claudedocs/issues/ISSUE-20260511-clang-knapsack-outlier.md`:
- 측정 데이터, IR diff, root cause 분석
- 권장: README 라벨 명시 (clang vs gcc), upstream 보고 후보

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Clang -O3 IR 추출 | ✅ unconditional store + select-phi 확인 |
| BMB opt -O2 IR | ✅ conditional store 확인 |
| GCC IR (간접) | ✅ 1.39x 결과로 conditional store 추정 정합 |

결함: BMB 측 결함 없음 (오히려 BMB의 lowering이 이 케이스에서 유리)

## Reflection

**핵심 통찰**:
- "BMB 6.7x faster" 주장은 **clang outlier에 한정** — gcc 기준 BMB는 1.39x 느림
- BMB의 naive lowering이 LLVM opt의 over-optimization을 회피하는 부수 효과 — 의도된 design 아님
- 이는 BMB IR generation의 stability를 보여주는 데이터 포인트 (단순한 IR이 항상 더 안정적인 perf 보장은 아니지만, 이 케이스에서는 ✓)

**도그푸딩 가치**:
- IR 비교가 측정 결과의 root cause를 명확히 함 (vs 마이크로벤치 측정만으로 추측)
- BMB의 lowering 전략 평가: "최적화 회피"보다 "예측 가능성" 우선이 결과적으로 좋음

**Roadmap impact**:
- M4-9 분석 종결 (별도 이슈로 deferral, BMB 측 작업 없음)
- README 라벨 명시는 HUMAN 결정 (M3-5)

## Carry-Forward
- Actionable:
  - Cycle 2705: Option C (compiler dynamic 우선화) 또는 다른 M4/M5 갭 — flexible
- Structural Improvement Proposals:
  - **README/HANDOFF**: clang vs gcc baseline 라벨 명시 (현재 "C" 모호함)
  - **벤치마크 인프라**: 기본 baseline을 clang+gcc dual로 통일 (이미 inproc 패턴, 표 표시만 정리)
- Pending Human Decisions:
  - **M3-5 [HUMAN]**: bmb-algo README "knapsack 6.7x faster than C" 라벨 정정 (clang -O3 outlier 명시)
- Roadmap Revisions: M4-9 → ISSUE-20260511-clang-knapsack-outlier로 deferral
- Next Recommendation: Cycle 2705 Option C (compiler dynamic 우선화) 또는 다른 갭 작업
