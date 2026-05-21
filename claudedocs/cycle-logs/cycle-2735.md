# Cycle 2735: Compiler-bug ISSUE 2건 v0.98 재현 시도 → 2 close

Date: 2026-05-11

## Re-plan

인계 (Cycle 2734 carry-forward + advisor): v0.98 재현 시도 — `if-else-early-return-codegen` + `recursive-function-codegen`. Trigger: ⚪ NONE.

## Scope & Implementation

### `if-else-early-return-codegen.md` — v0.98 재현 시도

| 테스트 | 패턴 | 결과 |
|--------|------|------|
| 기본 | `if n<2 { 0 } else { println(42); 0 }` | 5/5 정답 `42` |
| Substantial work | `count_primes(100)` else branch | 5/5 정답 `25` |

→ v0.51.22 era 버그 **v0.98에서 재현 불가**. Close.

### `recursive-function-codegen.md` — v0.98 재현 시도

heapify on `[5,3,8,1,9,2,7,4,6,0]`, root=0:
- 1차 swap (v[0]<>v[2]): 8 → v[0]
- 2차 swap (v[2]<>v[6]): 7 → v[2]
- 예상 `8\n7\n`
- **5/5 deterministic** = 정답

→ v0.51.22 "garbage values" 버그 **v0.98에서 재현 불가**. Close.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 2 ISSUE v0.98 재현 시도 | ✅ 둘 다 정상 동작 |
| 2 ISSUE close | ✅ |
| Active ISSUE 카운트 | 21 → **19** (-2) |
| 누적 close (세션) | 5 |
| 재현 파일 정리 | ✅ test_*.bmb 삭제 |

결함: 없음.

### 백그라운드 진척

- Golden full: **1042/2862 (36%)** — **0 FAIL** so far
- Tier all bench: ~200 (Tier 1 후반)

## Reflection

### advisor의 정확성 검증

> "stamp says '30-50% 이미 fix됨'. 단순 grep + 1 test = 1 cycle. resolved 시 close."

advisor 권고 100% 정확:
- 두 ISSUE 모두 v0.98에서 재현 불가
- 각 1 cycle 내 검증 + close
- 1년 stale ISSUE가 actionable result로 전환

이는 양식 표준화의 진짜 leverage: **stale 데이터 명시 → 단순 재검증 가능 → 빠른 close**.

### 측정 가능한 변화

- Active ISSUE: 21 → **19** (-2)
- Closed ISSUE: 38 → **40** (+2)
- 누적 close (세션 2728-2735): 5 (llvm-name-conflicts, simd-vectorization, roadmap-sync, if-else-early-return, recursive-function-codegen)

### 패턴 인식

bmb-ai-bench era ISSUE (2026-03-26 작성, v0.51.22 측정) 중 compiler-bug 카테고리는 codegen 광범위 변경으로 대부분 해소 추정. 잔여 13 ISSUE 중 LLM-bench methodology (10건)는 M4-1 baseline 실행 시 일괄 갱신, compiler-bug (1건 `multiple-pre-clauses`)는 spec 변경 필요.

## Carry-Forward

- Actionable (다음 cycle):
  - **C10 (final)**: 백그라운드 작업 결과 분석 (~36% golden, ~70% bench Tier1) + 최종 commit
- Structural Improvement Proposals:
  - 양식 표준화 phase **COMPLETE** — 추가 작업 없음
  - 백그라운드 작업 완료 시 lcs_three flakiness rate 실측 갱신 (ISSUE observed_rate)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음
- Next Recommendation: **C10 = 최종 cycle (결과 분석 + commit)**

## 세션 요약 (Cycles 2728-2735, 8 cycles)

| Cycle | 산출 |
|-------|------|
| 2728 | lcs_three 회귀 가설 기각 (4 fail environmental UB 진단) |
| 2729 | 풀 골든 + Tier all 백그라운드 재실행 시작 |
| 2730 | `_template.md` 신규 + 11 ISSUE stamp + 2 close |
| 2731 | 9 B-track batch reference pointer 적용 |
| 2732 | hashmap-perf + alloc-optimization 양식 정규화 |
| 2733 | roadmap-sync close (v0.98 재측정으로 모든 claim resolved) |
| 2734 | strict 6-field 검증 + HANDOFF 정정 |
| 2735 | if-else-early-return + recursive-function v0.98 재현 → 2 close |

- Active ISSUE: 25 → **19** (-6, 28 close 시 -3, 신규 +1 = -5, 7 close total + 1 new -1 = net -6)
- 양식 표준화 100% coverage (12 직접 + 9 batch reference + 신규 `_template.md`)
- HANDOFF 정정 (lcs_three "1 FAIL 회귀" → "4 fail environmental UB")
