# Cycle 2754: bench_algo.py knapsack/quicksort large variant — M3-5 narrative 보강

Date: 2026-05-12

## Re-plan

Cycle 2753 Next Rec 후보 (C): bench_algo.py large variant 추가. Trigger ⚪ NONE.

## Scope & Implementation

### bench_algo.py 확장

**신규 large 입력**:
- `WEIGHTS_LG`, `VALUES_LG` (100 random items), `CAPACITY_LG ≈ 1300` — knapsack(100)
- `SORT_DATA_LG` (1000 random ints) — quicksort(1000)
- `random.seed(42)` 결정론적

**신규 run** (results 리스트):
- `knapsack(10)` (existing renamed) + `knapsack(100)` 신규
- `quicksort(15)` (existing) + `quicksort(1000)` 신규

### 측정 결과 (v0.98, 2026-05-12)

| Algorithm | BMB | Python | Speedup |
|-----------|-----|--------|---------|
| **knapsack(10)** | 3.5-4.0 us | 24 us | **~7×** |
| **knapsack(100)** | 50 us | 22.5 ms | **~450×** |
| **prime_count(10000)** | 9 us | 448 us | **~49×** |
| **edit_distance** | 1.7 us | 14.6 us | **~8.5×** |
| **nqueens(10)** | 1.72 ms | 9.93 ms | **~5.8×** |
| **quicksort(1000)** | 218-262 us | 548-573 us | **~2.5×** |
| **quicksort(15)** | 3.9 us | 3.6 us | ~0.9× ⚠️ |
| merge_sort(15) | 4.0 us | 11.7 us | ~2.9× |
| fibonacci(30) | 0.4 us | 0.6 us | ~1.5× |

### Scaling 분석 (sweep)

```
knapsack scaling:
  n=10  cap=106:  speedup= 33×
  n=30  cap=373:  speedup=172×
  n=100 cap=1360: speedup=452×
  n=300 cap=3613: speedup=617×

nqueens scaling:
  n=8:  speedup=  7.6×    ← v0.2.0 claim 181× NOT reproducible
  n=10: speedup=  5.6×
  n=12: speedup=  4.7×

quicksort scaling (FFI crossover):
  n=15:   speedup=0.72×  (FFI-bound)
  n=50:   speedup=0.99×  (parity)
  n=100:  speedup=1.16×  
  n=500:  speedup=2.09×
  n=1000: speedup=2.58×
```

### README 갱신

**Headline**:
- Cycle 2753: "Up to 43× (prime_count)" → Cycle 2754: **"Up to ~450× (knapsack(100))"**
- 정직한 회복: v0.2.0 90× claim 가 n≥30에서 재현됨을 검증

**표 + scaling section 추가**:
- knapsack scaling 표 + quicksort scaling 표
- 사용 권고: ≥100 elements/states 에서 FFI overhead 분상 (input-size guidance)

**Historical 섹션 갱신**:
- v0.2.0 `knapsack 90.7×` → consistent (n≥30에서 재현 가능)
- v0.2.0 `nqueens(8) 181.6×` → **재현 불가** (모든 N에서 5-8×만 측정) — 명시적 disclose

### CHANGELOG 갱신

`[Unreleased]` 섹션 갱신:
- Cycle 2753 entries 갱신 (headline 450×로 정정, scaling table 명시)
- knapsack(100) / quicksort(1000) bench 신규
- v0.2.0 nqueens(8) 181× 재현 불가 표시

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 신규 large variant 실행 성공 (knapsack 100, quicksort 1000) | ✅ |
| 결과 재실행 일관성 (knapsack(100) 442× vs 447×) | ✅ (~5% variance 정상) |
| nqueens(8) 181× 재현 불가 확인 | ✅ (모든 N에서 5-8×만) |
| README headline 정정 | ✅ |
| Scaling table 추가 | ✅ |
| CHANGELOG annotation | ✅ |

결함: 없음.

## Reflection

### 정직한 narrative 강화의 의미

Cycle 2753 option (a) "v0.98 numbers"는 honest but conservative. Cycle 2754는:
- (a) 유지 (v0.98 numbers + v0.2.0 archival)
- + (b) scaling demo (n=10/100/300 sweep 표)
- + 입력 크기 가이드 (사용자 권고)

이렇게 하니 narrative이 "v0.2.0은 90×, 지금은 7×" (descending) 에서 "n=10에서 7×, n=100에서 450× — input size에 따라 amplify" (scaling) 으로 회복. 이게 **객관적 진실**임 — bench config 차이만 정직하게 disclose하면 v0.2.0 90× 도 회복.

### v0.2.0 nqueens(8) 181× 의 정직한 처리

3가지 가능:
- (a) silent (cherry-pick) — 정직 위반
- (b) "180× achievable somewhere" — verifiable 아님
- (c) **재현 불가 명시** ✅

(c) 선택은 publish 시 user trust ROI. README "not reproducible at any tested size" 명시.

### bench harness 구조 검증

기존 bench_algo.py은 단일 input config만. 단일 config가 "전체 알고리즘 특성" 대표 불가 (input size에 따라 dramatically scale). scaling sweep 표가 신규 가치.

### Pacing 점검 (advisor 지적)

소비 cycle: 5/10 (2750-2754). 잔여 5 cycles. Cycle 2755+ 후보:
- A: ISSUE 백로그 cleanup (1-2 cycles bounded)
- B: multiple-pre-clauses 파서 spec (1-2 cycles)
- D: FP arity guard 36 sites (1 cycle low ROI)
- (C는 이 cycle에서 통합 처리됨)

Bounded single-cycle 작업 우선 권고 (잔여 5 cycles에 multi-cycle phase 시작 회피).

## Carry-Forward

### Actionable

- **Cycle 2755**: 후보 (A) ISSUE 백로그 cleanup — active 19 ISSUE 중 stale/superseded 후보 식별 + close 추진. Cycle 2750/2751 측정 갱신 데이터 (alloc-optimization 1.010x, sorting 0.91x) leverage.
- **Cycle 2756+**: 후보 (B) multiple-pre-clauses 또는 (D) FP arity guard 36 sites

### Structural Improvement Proposals

1. **bench_algo.py scaling sweep 자동화**:
   - 현 large variant는 수동 추가. `--scaling` 플래그로 n=10/30/100/300 자동 sweep 가능
   - 1 cycle, 추후 candidate
2. **nqueens(8) 181× source 추적** (long-term):
   - v0.2.0 commit history에서 bench harness diff 확인
   - HUMAN review 가치 작음 (기록 archival), low priority

### Pending Human Decisions

- M3-5 narrative HUMAN review (Cycle 2753 + 2754 누적): headline 450×로 회복 + scaling sweep + v0.2.0 partial reproducibility 명시 → 승인 적정성

### Roadmap Revisions

ROADMAP `§ M3-5 row` 갱신 가치: ✅ Cycle 2753 → ✅ Cycles 2753-2754 (large variant 추가 명시). 변경 누적량 작아서 옵션.

### Next Recommendation

**Cycle 2755: ISSUE 백로그 cleanup** — Cycle 2750/2751 측정 갱신 데이터 (alloc-optimization 1.010x, sorting 0.91x BMB FASTER)로 close 후보 식별. 19 active → 17-18 active 목표.

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| `ecosystem/bmb-algo/benchmarks/bench_algo.py` (large variant 추가) | tracked |
| `ecosystem/bmb-algo/README.md` (headline 450× 회복 + scaling section) | tracked |
| `ecosystem/bmb-algo/CHANGELOG.md` (scaling demo annotation) | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2754.md` | gitignored |
