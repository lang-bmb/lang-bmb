# Cycle 2732: 잔여 2 P-track ISSUE 양식 정규화

Date: 2026-05-11

## Re-plan

인계 (Cycle 2731 carry-forward): `hashmap-perf` + `alloc-optimization` 양식 정리. Trigger: ⚪ NONE. Light mechanical cycle.

## Scope & Implementation

| ISSUE | 변경 |
|-------|------|
| `ISSUE-20260413-hashmap-perf.md` | informal stamp → 표준 양식 (6필드 + 측정 추이 표). 1.027x BMB slower, 8.3 pp 개선 |
| `ISSUE-20260413-alloc-optimization.md` | informal stamp → 표준 양식 (6필드 + 측정 추이 표). 1.043x BMB slower, 1.7 pp 개선 |

이제 active 22/22 ISSUE 모두 양식 적용.

### 백그라운드 진척

- Golden full: ~578/2862 (20%)
- Tier all: Tier 1 진행 중 (~80 benchmarks complete)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 2 ISSUE 표준 양식 적용 | ✅ |
| Active ISSUE 양식 coverage | **22/22 (100%)** |

결함: 없음.

## Reflection

C3-C4 (Cycle 2730-2731-2732)를 통해 ISSUE 양식 표준화 phase 완료:
- `_template.md` 신규 (표준 양식 reference)
- `_b_track_methodology_stamp.md` 신규 (9 ISSUE batch reference)
- 22 active ISSUE 모두 양식 적용
- 2 ISSUE close (llvm-name-conflicts + simd-vectorization)
- 1 신규 ISSUE 등록 (golden-flakiness-inttoptr — 양식 first application)

다음 cycle은 백그라운드 작업 완료 대기 + 결과 분석.

## Carry-Forward

- Actionable (다음 cycle):
  - **C7**: 골든 결과 분석 (lcs_three / cholesky_trace / crc32_simple / assortativity 재현율 확정)
  - **C8**: Tier all bench 분석 (P-track 변화)
- Structural Improvement Proposals:
  - `_template.md` README mention (선택)
  - Roadmap-sync ISSUE close 결정 (v0.98 재측정 확정 후)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음
- Next Recommendation: **C7 = 백그라운드 작업 모니터링 + 결과 분석**
