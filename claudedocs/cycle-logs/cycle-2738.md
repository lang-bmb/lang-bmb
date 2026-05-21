# Cycle 2738: ISSUE backlog deep audit — v0.98 close 후보 부재 확인

Date: 2026-05-11

## Re-plan

인계: Cycle 2737 carry-forward "ISSUE backlog deep audit". Trigger: ⚪ NONE. 19 active ISSUE 전수 검토하여 v0.98 추가 close 가능성 평가.

## Scope & Implementation

### 19 active ISSUE 카테고리 분류

| 카테고리 | 카운트 | ISSUE | close 가능성 |
|---------|-------|-------|-------------|
| **B-track methodology** (M4-1 종속) | 10 | ISSUE-20260326-* | ❌ HUMAN gate (BMB_BENCH_API_KEY) |
| **P-track multi-cycle** | 4 | hashmap-perf / alloc-optimization / or-chain-lowering / clang-knapsack-outlier | ❌ multi-cycle scope |
| **Feature / Tool 진척** | 3 | linter-enhancement / playground-wasm / simd-codegen | ❌ multi-cycle, partial 진척 |
| **Deferred (정식)** | 1 | smt-integration | △ "Open Deferred" 유지 권고 |
| **언어 spec 변경 필요** | 1 | multiple-pre-clauses | ❌ compiler.bmb + bootstrap 필요 |

### 핵심 검증 — 언어 갭 ISSUE 회수율

ROADMAP § M4 prep에 명시된 3개 언어 갭 (let-tuple / static-method / Option-expr) 모두:

| ISSUE | 상태 |
|-------|------|
| `ISSUE-20260510-let-tuple-destructuring.md` | ✅ closed/ (Cycle 2621) |
| `ISSUE-20260510-static-method-call.md` | ✅ closed/ (Cycle 2620) |
| `ISSUE-20260510-option-expr-position.md` | ✅ closed/ (Cycle 2633) |

→ ROADMAP과 ISSUE 백로그 정합성 확인.

### 추가 close 후보 부재 결론

각 카테고리별 사유:
1. **B-track methodology** (10건): M4-1 baseline 미실행 — 갱신 불가, 본질적으로 valid
2. **P-track multi-cycle** (4건): 실측 갭 존재 (hashmap 1.027x / alloc 1.043x), 측정 stamp 갱신은 Tier all 결과 후 (Cycle 2741+ 예정)
3. **Feature 진척** (3건): linter (11/N), playground-wasm (미구현), simd (Phase 1 스캐폴딩만)
4. **smt-integration**: Deferred 명시, 재개 조건 보존 — 형식적 close보다 Open 유지가 visibility 측면 우수
5. **multiple-pre-clauses**: v0.98 재현 확인 (parser error 동일), fix는 Rule 6에 따라 compiler.bmb + bootstrap 필요. 현재 bench 동시 실행으로 Stage 1-3 헤비 작업 회피

### multiple-pre-clauses v0.98 재현 확인

```bmb
fn safe_get(idx: i64, len: i64) -> i64
    pre idx >= 0
    pre idx < len
= idx;
```
→ parser error: `Unrecognized token 'pre' found...` (line 105 col 5)

→ ISSUE 유효, fix 보류 (cycle scope 적합 시 분리 phase)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 19 active ISSUE 전수 검토 | ✅ |
| closed/ 언어 갭 ISSUE 회수율 | ✅ 3/3 |
| multiple-pre-clauses v0.98 상태 | ✅ 재현 확인 (parser error 동일) |
| close 후보 부재 결론 정당화 | ✅ 카테고리별 사유 명시 |

결함: 없음. (audit-only cycle, 변경 0)

## Reflection

### 양식 표준화 효과 누적 분석

| 세션 | 백로그 변화 |
|------|-------------|
| Cycles 2728-2736 시작 | 23 active |
| Cycles 2728-2736 종료 | 19 active (-4 net) |
| **Cycle 2738 (현재)** | **19 active (변화 없음)** |

→ 양식 표준화 leverage 1단계 완료. 잔여 19건은 모두 본질적 multi-cycle 또는 HUMAN 종속. **추가 doc-only close 가능성 소진**.

### 다음 leverage 영역

ISSUE 백로그 자체 cleanup은 saturation. 추가 leverage는:
1. M4-1 baseline (HUMAN unlock 시) → 10 B-track ISSUE 일괄 갱신
2. Tier all 측정값 → P-track 4 ISSUE 추이 stamp
3. multi-cycle phase 시작 → 각 ISSUE에 직접 work

### audit-only cycle의 가치 검증

audit cycle은 "추가 close 없음"을 결론지었지만:
- ISSUE-ROADMAP 정합성 확인 (3/3 언어 갭 closed/)
- multiple-pre-clauses v0.98 재현 확인 (Cycle 2735 패턴 적용 — v0.98 verify-or-close)
- 양식 표준화 saturation 측정

→ "추가 진척 없음" 확인 자체가 다음 cycle을 명확하게 함.

## Carry-Forward

- Actionable: 없음 (audit-only)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음
- Next Recommendation: **Cycle 2739** — context-overflow-prevention 스크립트 수정 (`run_experiment.py` 슬라이딩 윈도우, 자율, ~30 LOC, bench 무관)
