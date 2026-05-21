# Cycle 2753: M3-5 bmb-algo README 정정 + CHANGELOG annotation

Date: 2026-05-12

## Re-plan

Cycle 2752 Next Recommendation 그대로. Trigger ⚪ NONE. 패턴-따르기 doc update.

## Scope & Implementation

### README 변경 (`ecosystem/bmb-algo/README.md`)

**Headline**:
- Before: "> 90x faster than Python on knapsack. 181x faster on N-Queens."
- After:  "> Up to 43× faster than pure Python (prime_count(10k) at v0.98, 2026-05-12)."

**Benchmark 표**:
- knapsack(100 items) 6.3× → knapsack(10 items, cap 20) **7.5×** (실 측정값 + 라벨 fix)
- nqueens(10) 4.1× → **5.3×**
- prime_count(10k) 32× → **43×**
- edit_distance 6.4× → **7.9×**
- merge_sort(15) 3.3× → **2.9×**
- fibonacci(30) 2× → **1.5×**
- quicksort(15) 2.1× → **0.6× ⚠️** (회귀 disclose, 별도 note)
- 측정 condition: v0.98, 500-iter mean, 10-iter warmup, ctypes FFI overhead 포함

**신규 sub-sections**:
- quicksort regression 인라인 note (ctypes FFI overhead 우위, 입력 크기 권고)
- "Historical measurements (archived)" — v0.2.0 90.7×/181.6× 명시 + 재현 불가 명시

### CHANGELOG 변경 (`ecosystem/bmb-algo/CHANGELOG.md`)

`[Unreleased]` 섹션 신규:
- "README benchmark table re-baselined to v0.98 measurements (2026-05-12). Supersedes v0.2.0 numbers from 2026-03-23."
- "Headline changed from '90x knapsack / 181x N-Queens' (v0.2.0 archival) to 'Up to 43× (prime_count(10k))' (current)."
- "Fixed mis-labeled 'knapsack(100 items)' → 'knapsack(10 items, cap 20)' (matches actual bench_algo.py configuration)."
- "Disclosed quicksort(15) regression: 2.1× → 0.6× — ctypes FFI overhead exceeds algorithmic cost at this input size."

### 신규 ISSUE 등록

`claudedocs/issues/ISSUE-20260512-bmb-algo-quicksort-ffi-overhead.md`:
- 우선순위 P3, scope = bmb-algo bindings 차원 (codegen 아님)
- 3 가설 (A: FFI marshalling / B: BMB 비효율 / C: Python 최적화)
- 3 옵션 (A: 가이드만 / B: ctypes 최적화 BC break / C: BMB quicksort 인라인)
- 종결 기준: quicksort(100) variant 추가 + ≥1.0× 또는 disclose 완료

### ROADMAP 갱신 (`claudedocs/ROADMAP.md`)

- M3 progress bar ~97% → ~98% (M3-5 자율 완결)
- § M3 잔여 표 M3-5 row: ⏳ → ✅ Cycle 2753

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| README headline v0.98 measurement 반영 | ✅ |
| 표 7 알고리즘 모두 v0.98 numbers + 라벨 정정 | ✅ |
| 90x/181x archival note | ✅ |
| quicksort 회귀 disclose | ✅ |
| CHANGELOG [Unreleased] re-baseline | ✅ |
| 신규 ISSUE 양식 6필드 | ✅ |
| ROADMAP M3-5 row 갱신 | ✅ |
| M3 progress bar 갱신 | ✅ |

결함: 없음.

## Reflection

### advisor option (a) 선택의 정확도 확인

advisor 권고: "v0.98 numbers + v0.2.0 archival note" — 정확:
- (a) "Up to" 약화 표현은 confusion 없이 진실하게 작동 (43× 라는 명확한 최대값 cite)
- v0.2.0 archival note는 "거짓이 아니다, 과거에는 그랬다" 인정 — BMB 철학 "측정 없는 성능 주장 금지" 와 정렬

### quicksort 회귀의 disclose 가치

선택지:
- (a) silent (현 README에서 제거하지 않음) — 다른 알고리즘 평균 6×와 일관성 깨짐
- (b) 제거 — bench 결과 censoring, 신뢰 손실
- (c) **disclose + 별도 ISSUE 등록 + 사용자 가이드** ✅ 선택

(c) 선택은 BMB 신뢰성 우선. 작은 입력에서 ctypes overhead가 큰 경우는 일반적이며, 그것을 인정하는 것이 publish 시 user trust 향상.

### M3 ~98%의 의미

| 잔여 | 책임 |
|------|------|
| M3-3 npm publish | HUMAN dispatch (`gh workflow run npm-publish.yml`) |
| M3-4 PyPI publish | HUMAN dispatch (`gh workflow run pypi-publish.yml`) |
| M3-5 narrative review | HUMAN review (헤드라인 + quicksort narrative 승인 또는 추가 정정 지시) |
| M3-7 baseline annotation | M4-1 종속 자동 처리 |

자율 가능 작업 종료. **C dispatch는 HUMAN action**, 다음 cycle은 잔여 자율 백로그 진입.

## Carry-Forward

### Actionable

- **Cycle 2754+**: 잔여 자율 백로그. 후보 (Cycle 2752 advisor 정합 옵션):
  - **A**: ISSUE 백로그 cleanup (stale 재측정 후 close — 1-2 cycles bounded)
  - **B**: `multiple-pre-clauses` 파서 spec 확장 (1-2 cycles, 언어 spec)
  - **C**: bench_algo.py knapsack(100) variant 추가 (1 cycle, M3-5 보강)
  - **D**: FP arity guard 36 sites mechanical (1 cycle, low ROI)
- **HUMAN review 대기**: M3-5 narrative — headline 약화 표현 + quicksort disclose 적절성. quicksort 회귀 해결은 별도 multi-cycle

### Structural Improvement Proposals

1. **bench_algo.py knapsack(100) + nqueens(8) variant 추가**:
   - 위치: `ecosystem/bmb-algo/benchmarks/bench_algo.py`
   - 변경: WEIGHTS_LARGE = [random 100], CAPACITY_LARGE = 500, run("knapsack(100)") 추가
   - 근거: v0.2.0 의 90.7× 가 큰 입력에서 amplify 됨을 demo 가능 → headline 회복 가능성
   - 1 cycle, candidate

2. **Tier 3 inproc 변환 multi-cycle phase**: Cycle 2752 carry-forward 유지 (별도 세션)

### Pending Human Decisions

- 신규 없음. M3-5 narrative HUMAN review 추가 — Cycle 2752 carry-forward와 일관.

### Roadmap Revisions

ROADMAP `§ 4 M3 잔여 표` M3-5 row ⏳ → ✅. `§ 4 M3 progress bar` 97% → 98%. 다른 섹션 변경 없음.

### Next Recommendation

**Cycle 2754: 후보 (C) `bench_algo.py knapsack(100) variant 추가`** — M3-5 narrative 보강 + headline 90×/181× partial 회복 가능성. 1 cycle bounded. 대안: 후보 (A) ISSUE 백로그 cleanup (bounded but variable scope).

Pacing 점검 (advisor 지적): 4 cycles 사용 (2750-2753), 6 cycles 잔여. bounded single-cycle 작업 우선, multi-cycle phase 시작 회피.

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| `ecosystem/bmb-algo/README.md` | 부모 repo directory | tracked |
| `ecosystem/bmb-algo/CHANGELOG.md` | 부모 repo directory | tracked |
| `claudedocs/issues/ISSUE-20260512-bmb-algo-quicksort-ffi-overhead.md` | issues | gitignored |
| `claudedocs/ROADMAP.md` (M3 progress + M3-5 row) | claudedocs | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2753.md` | gitignored |
