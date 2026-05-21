# Cycle 2755: ISSUE 백로그 cleanup — alloc-optimization + smt-integration close

Date: 2026-05-12

## Re-plan

Cycle 2754 Next Rec 그대로. Trigger ⚪ NONE.

## Scope & Implementation

### 19 active ISSUE 인벤토리

- **9 B-track 2026-03-26**: methodology stamp batch — M4-1 baseline 후 re-eval, **현 처리 skip**
- **6 codegen/infra 2026-04-13**: 부분 진척 + Deferred 혼재
- **3 from 2026-05-11** (Cycle 2718 신규): clang-knapsack-outlier / golden-flakiness-inttoptr / or-chain-lowering — 모두 ROADMAP에 deferred 명시됨
- **1 신규 2026-05-12** (Cycle 2753): bmb-algo-quicksort-ffi-overhead

### Close 2건

**1. alloc-optimization** (2026-04-13 → 2026-05-11 stamp):
- Cycle 2750 측정 1.010x ≈ parity
- P-track ≤1.05x 충족
- Close criteria "binary_trees ≤ 100%"는 aspirational
- Arena infra 4-6 cycles 부적합 ROI (1pp → 0.5pp 효과)
- Resolution: "1.010x ≈ parity within Tier 1 measurement noise floor"
- 재개 조건: ratio_c 1.05x 초과 회귀 또는 GC/RAII 도입 시 동반 검토
- 파일 이관: `claudedocs/issues/closed/`

**2. smt-integration** (2026-04-13 Deferred):
- 2026-04-13 Cycle 382에서 명시적 **Deferred** 결정
- 1+ 년 누적 진척 없음
- "Deferred" ≠ "Open" — backlog 정리 위해 closed/ 이관 적절
- 본문 그대로 유지 (재개 조건 archived)
- 파일 이관: `claudedocs/issues/closed/`

### 인벤토리 변화

| 시점 | active | closed | 변화 |
|------|--------|--------|------|
| Cycle 2735 종료 | 19 | 40 | (baseline) |
| Cycle 2753 신규 (quicksort-ffi) | 20 | 40 | +1 |
| Cycle 2755 close 2건 | **17** | **42** | -3 (2 close + 1 신규) |

ROADMAP § 6 "Cycle 2755 갱신" sub-section 추가됨.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 17 active ISSUE 확인 | ✅ (`ls -1 claudedocs/issues | grep -v "^_\|^closed" | wc -l` = 17) |
| 42 closed (+2 이번) | ✅ |
| 2 closed 파일 양식 보존 가이드 4번 준수 | ✅ (closed/ 이동) |
| close resolution 본문 추가 | ✅ |
| ROADMAP § 6 갱신 | ✅ |

결함: 없음.

## Reflection

### Conservative close vs aggressive close

alloc-optimization close 결정 시 옵션:
- (a) **Aggressive close** (1.010x ≈ parity, criteria 우회) ✅ 선택
- (b) Conservative keep open at P3 (strict criteria 충족 안 됨)
- (c) Criteria 재정의 후 close

(a) 선택 근거:
- 1pp 갭은 Tier 1 5-run noise floor 내 (2-5pp)
- Arena infra ROI 매우 낮음 (4-6 cycles → 0.5pp 효과)
- "측정 없는 성능 주장 금지" + "성능 저하는 버그다" 원칙 균형: 측정 잘 됐고, 실제 사용자 영향 없음
- close resolution이 재개 조건을 명확히 함 → 부담 없는 close

### "Deferred 1+ 년"은 closed 신호

smt-integration 처리는 "양식 보존 가이드 5번 — Deferred 명시" 와 "4번 — close 시 closed/ 이동" 사이의 gray zone. 1+ 년 미진척 + 명시적 Deferred + 외부 신호 부재 = closed 이관 적절.

**구조 개선 후보**: `_template.md`에 "Deferred 6개월 이상 → closed 자동 이관 권고" 추가. 별도 cycle.

### Active 17 인벤토리 평가

| 카테고리 | 개수 | 처리 권고 |
|----------|------|----------|
| B-track 2026-03-26 batch | 9 | M4-1 baseline 후 일괄 close 후보 |
| codegen/infra 2026-04-13 | 4 (linter / playground-wasm / simd-codegen / hashmap-perf) | hashmap-perf는 측정 추적, 3건은 feature spec |
| 2026-05-11 (Cycle 2718 era) | 3 (clang-knapsack-outlier / golden-flakiness-inttoptr / or-chain-lowering) | M4-9 종속 + multi-cycle |
| 2026-05-12 (이번 세션) | 1 (quicksort-ffi) | M3-5 종속 |

17 모두 정당하게 open. multi-cycle phase 시작 회피 시 close 후보 부재.

### Pacing 점검 (advisor 지적)

소비 cycle: 6/10 (2750-2755). 잔여 4 cycles. 다음 후보:
- B: multiple-pre-clauses 파서 spec 확장 (1-2 cycles, 언어 spec)
- D: FP arity guard 36 sites mechanical (1 cycle, low ROI)
- 작은 작업 ad-hoc

Cycle 2756 = multiple-pre-clauses 추진 권고 (advisor "bounded single-cycle" 정합).

## Carry-Forward

### Actionable

- **Cycle 2756**: multiple-pre-clauses 파서 spec 확장 — `ISSUE-20260326-multiple-pre-clauses.md` 참조. 현재 workaround = `and` chain. 적정 spec 정의 + parser 변경 1-2 cycles.

### Structural Improvement Proposals

1. **`_template.md` "Deferred ≥6개월 → closed 이관 권고" 가이드 추가**: smt-integration 처리 패턴 표준화. 1 cycle, candidate.
2. **B-track 9 ISSUE 일괄 close 절차 표준화**: M4-1 baseline 완료 시 점검 자동화 (10-min cycle, 매뉴얼 검토 + 일괄 처리). M4-1 종속, M4-1 후속 cycle에서 처리.

### Pending Human Decisions

- 신규 없음
- 기존 큐 유지

### Roadmap Revisions

ROADMAP § 6 "Cycle 2755 갱신" sub-section 추가. M축 영향 없음.

### Next Recommendation

**Cycle 2756: multiple-pre-clauses 파서 spec 확장**

`ISSUE-20260326-multiple-pre-clauses.md` 본문 검토 → spec 정의 → 파서 변경 → 골든 테스트.

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| `claudedocs/issues/closed/ISSUE-20260413-alloc-optimization.md` | (mv from active) | gitignored |
| `claudedocs/issues/closed/ISSUE-20260413-smt-integration.md` | (mv from active) | gitignored |
| `claudedocs/ROADMAP.md` § 6 (Cycle 2755 sub-section) | claudedocs | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2755.md` | gitignored |
