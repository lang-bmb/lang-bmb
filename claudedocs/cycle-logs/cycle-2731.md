# Cycle 2731: ISSUE 양식 표준화 — 9 B-track batch pointer 추가

Date: 2026-05-11

## Re-plan

인계 (Cycle 2730 carry-forward C4): 9 LLM-bench ISSUE에 `_b_track_methodology_stamp.md` 1-line pointer. Trigger: ⚪ NONE. Light mechanical cycle.

## Scope & Implementation

9개 ISSUE 첫 줄 `**Status: OPEN**`에 batch reference + STALE 표시 추가:

| ISSUE | edited |
|-------|--------|
| `ISSUE-20260326-context-overflow-prevention.md` | ✅ |
| `ISSUE-20260326-crosslang-reference-asymmetry.md` | ✅ |
| `ISSUE-20260326-external-problem-validation.md` | ✅ |
| `ISSUE-20260326-first-shot-rate-low.md` | ✅ |
| `ISSUE-20260326-integration-category-weakness.md` | ✅ |
| `ISSUE-20260326-multi-model-validation.md` | ✅ |
| `ISSUE-20260326-problem-difficulty-bias.md` | ✅ |
| `ISSUE-20260326-statistical-testing.md` | ✅ |
| `ISSUE-20260326-type-d-failure-analysis.md` | ✅ |

**Pattern**: `**Status: OPEN** — Cycle 2730 양식: 측정 stamp는 [_b_track_methodology_stamp.md] 참조 (**STALE** since 2026-04-26)`

### 백그라운드 진행 (~중간 점검)

- Golden full: ~495/2862 (17%)
- Tier all bench: ~50/N (Tier 1 진행 중)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 9 ISSUE batch pointer | ✅ all 9 edited |
| Active ISSUE measurement stamp coverage | 19/22 명시적 + 3 obsolete metadata = ≈100% |

결함: 없음.

## Reflection

ISSUE 양식 표준화 완성도:
- **신규 표준**: `_template.md` 제공
- **stamp 적용**: 직접 11 ISSUE + batch reference 9 ISSUE = 20/22 (91%)
- **잔여 2 ISSUE** (`hashmap-perf`, `alloc-optimization`): 이미 informal stamp 보유, 양식 형식 통일 가능하나 informational 충분

표준화 leverage 첫 효과 (Cycle 2730 기록):
- 1년 stale `roadmap-sync` 데이터 명시적 식별
- llvm-name-conflicts ISSUE 해결 사실 발견 (Lint 11)
- simd-vectorization superseded 상태 정리

## Carry-Forward

- Actionable (다음 cycle):
  - **C5**: Cycle 2729 백그라운드 작업 (~50% 완료 추정) 모니터링 + 결과 분석
  - **C6**: cycle 2728 발견 ISSUE 양식 leverage 첫 검증 — golden 결과로 flakiness rate 정량화 (4 failing tests 재확인)
- Structural Improvement Proposals:
  - `hashmap-perf` / `alloc-optimization` 양식 정리 (Optional, 측정 추이 명확)
  - `_template.md` README mention (다음 cycle)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음
- Next Recommendation: **C5 = 백그라운드 완료 모니터링** → C6 = 결과 분석
