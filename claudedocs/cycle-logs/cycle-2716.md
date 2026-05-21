# Cycle 2716: ISSUE 백로그 triage (40개)
Date: 2026-05-11

## Re-plan
인계 (Cycle 2715): ISSUE 40+ triage. Trigger ⚪ NONE.

## Scope & Implementation

### 분류 카테고리

| 카테고리 | 수 | 의미 |
|---------|----|------|
| ✅ **Resolved** | 15 | 과거 또는 이번 cycle로 해소 — closed/ 이동 후보 |
| ⏳ **HUMAN-locked** | 13 | M4-1 (B 공식 측정) 의존 — `BMB_BENCH_API_KEY` 필요 |
| ⚠️ **Actionable** | 9 | 자율 cycle 진척 가능, 우선순위 평가 필요 |
| 🧊 **Stale 후보** | 3 | 1개월+ 무진척, 재평가 권고 |

### 상세 분류

#### ✅ Resolved (15개)

**2026-05-01 Track 시리즈 (7) — 모두 M2/M3 완료로 흡수**
- `ISSUE-20260501-track-m-machine-output.md` — Cycle 2575+ ✅
- `ISSUE-20260501-track-n-mcp-server.md` — Cycle 2557 ✅
- `ISSUE-20260501-track-o-context-pack.md` — Cycle 2549+ ✅
- `ISSUE-20260501-track-q-ambiguity-audit.md` — Cycle 2592 ✅ (~92%)
- `ISSUE-20260501-track-r-llm-bench.md` — Cycle 2592 ✅ (~95%)
- `ISSUE-20260501-track-t-external-bindings.md` — Cycle 2565 ✅
- `ISSUE-20260501-track-s-ecosystem-bmb-rewrite.md` — Cycle 2611 ✅ (~99%)

**2026-05-10 M5 언어 갭 (5)**
- `ISSUE-20260510-let-tuple-destructuring.md` — Cycle 2621 ✅ (M4-3)
- `ISSUE-20260510-static-method-call.md` — Cycle 2620 ✅ (M4-4)
- `ISSUE-20260510-option-expr-position.md` — Cycle 2633 ✅ (M5-1)
- `DESIGN-M5-1-payload-enum.md` — Cycle 2633 ✅
- `DESIGN-M5-3-multi-field-enum.md` — Cycle 2637 ✅

**2026-05-11 최근 (3)**
- `ISSUE-20260511-set-field-index.md` — Cycle 2690-2692 ✅ (M5-5g)
- `ISSUE-20260511-golden-regression-3.md` — Cycle 2701 ✅ (0 FAIL)
- `ISSUE-20260511-golden-manifest-audit.md` — Cycle 2693+2698 ✅

**2026-04-13 Bootstrap Fixed Point**
- `ISSUE-20260413-bootstrap-fixed-point.md` — **Cycle 2711-2714 ✅ 회복** (token packing 5M scale + builtin arity guard)

#### ⏳ HUMAN-locked (13개)

**2026-03-26 B축 baseline 의존 — M4-1 잠금**
- `ISSUE-20260326-first-shot-rate-low.md`
- `ISSUE-20260326-integration-category-weakness.md`
- `ISSUE-20260326-crosslang-reference-asymmetry.md`
- `ISSUE-20260326-external-problem-validation.md`
- `ISSUE-20260326-llvm-name-conflicts.md` — **부분 해소** (Cycle 2697/2703/2712/2714 — arity guard + lint 11)
- `ISSUE-20260326-if-else-early-return-codegen.md`
- `ISSUE-20260326-context-overflow-prevention.md`
- `ISSUE-20260326-multi-model-validation.md`
- `ISSUE-20260326-multiple-pre-clauses.md`
- `ISSUE-20260326-problem-difficulty-bias.md`
- `ISSUE-20260326-recursive-function-codegen.md`
- `ISSUE-20260326-statistical-testing.md`
- `ISSUE-20260326-type-d-failure-analysis.md`

→ B축 공식 측정이 잠금 해소 시 자동 재평가 가능. `BMB_BENCH_API_KEY` 필요.

#### ⚠️ Actionable (9개)

**2026-04-13 트랙** (priority 추정):
1. **High** `ISSUE-20260413-hashmap-perf.md` — HashMap 성능 (P축)
2. **High** `ISSUE-20260413-string-builder-opt.md` — StringBuilder 최적화 (P축)
3. **Medium** `ISSUE-20260413-alloc-optimization.md` — 메모리 할당 (P축)
4. **Medium** `ISSUE-20260413-compare-inline.md` — 비교 인라인 (P축)
5. **Medium** `ISSUE-20260413-linter-enhancement.md` — lint 12+ (Track Q)
6. **Medium** `ISSUE-20260413-match-jump-table.md` — match jump (P축)
7. **Low** `ISSUE-20260413-simd-codegen.md` — SIMD 1급 타입
8. **Low** `ISSUE-20260413-simd-vectorization.md` — SIMD 자동 벡터화

**Deferred (clang outlier)**:
9. `ISSUE-20260511-clang-knapsack-outlier.md` — clang -O3 anti-pattern (BMB 측 작업 없음, M4-9 deferral)

#### 🧊 Stale 후보 (3개)

- `ISSUE-20260413-playground-wasm.md` — playground 트랙 침체, M1-3 우선순위 낮음
- `ISSUE-20260413-roadmap-sync.md` — `docs/ROADMAP.md` vs `claudedocs/ROADMAP.md` sync. Drift D 해소 (Cycle 2521+)로 부분 해소
- `ISSUE-20260413-smt-integration.md` — Z3 IPC는 Cycle 2603+ 있으나 SMT 파이프라인 본격 활성화는 별도 트랙

→ 다음 세션에서 재평가 또는 archived/ 폴더 이동

### 액션

이 사이클은 **분류 보고서만** 작성. 파일 이동 (closed/, archived/)은 위험이 적지만 사이클 부담. **Cycle 10 또는 다음 세션 Carry-Forward**로.

근거: `Cycle log = 권고, file 이동 = 다음 액션`. 보고서 내 STATUS 마크가 충분.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 40개 ISSUE 분류 | ✅ |
| ✅ Resolved 15개 정확성 | ✅ (cycle log 참조로 확정) |
| Bootstrap Fixed Point ISSUE — 이번 세션 회복 표기 | ✅ |

결함: 없음.

## Reflection

### 외부 관찰자 관점

1. **HUMAN-locked 13개의 정체**: B축 baseline 측정 1번이 13개 이슈를 일괄 재평가 트리거. M4-1 진척이 큰 ROI.

2. **Actionable 9개의 우선순위 모호**: P축 (HashMap/StringBuilder/Alloc/Compare/match-jump)이 5개로 클러스터링. 다음 세션 1개 cycle = 1개 fix 페이스로 풀 수 있는 분량.

3. **Resolved 15개의 정체**: closed/ 폴더로 이동하면 활성 백로그 40 → 25로 감소. 다음 세션 triage 부담 절반.

4. **B축 정체의 비대칭**: B축 미선언 (Drift B, ROADMAP § 2)이 자율 cycle 13개를 묶음. 메인테이너 액션 1개 = 13 ISSUE 해소 잠재.

### Roadmap impact

- M4-1 (B 공식 측정) **권장 우선순위 강화** — 1 액션 → 13 ISSUE 잠금 해소
- Cycle 10 commit 시 closed/ 이동 가능 (또는 다음 세션)

## Carry-Forward

- Actionable (Cycle 10 = 2717):
  - **HANDOFF + ROADMAP 갱신** — 이번 세션 매우 큰 변곡점 (Stage 2 회복)
  - **memory note 정정** — OOM 가설 (Cycle 2708-2712 정정 누적)
  - 종합 commit
  - (선택) Resolved 15 ISSUE → closed/ 이동
- Structural Improvement Proposals:
  - **M4-1 B 측정 일순 우선화**: 13개 이슈 잠금 해소 ROI
  - **closed/ + archived/ 폴더 구조**: 향후 triage 부담 감소
- Pending Human Decisions: 변경 없음 (M3-3, M3-4, M3-5, M4-1)
- Roadmap Revisions: Cycle 10에서 일괄
- Next Recommendation: Cycle 10 = HANDOFF/ROADMAP/memory 갱신 + 종합 commit
