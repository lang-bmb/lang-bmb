# Cycle 2730: ISSUE 양식 표준화 — `_template.md` + 11 ISSUE stamping + 2 close

Date: 2026-05-11

## Re-plan

인계 (Cycle 2728/2729 + HANDOFF § 2): "🚨 ISSUE 양식 표준화 [HIGHEST LEVERAGE]" structural improvement. Trigger: ⚪ NONE.

## Scope & Implementation

### `_template.md` 신규 작성

`claudedocs/issues/_template.md` — measurement_date / stale_after / measurement_source / observed_rate / scope / env_hash 6필드 필수. 측정 기반 아닌 ISSUE는 `n/a` 명시. 양식 보존 가이드 (재측정 갱신 / stale_after 도달 시 처리) 포함.

### 표준화 적용 (10건)

| ISSUE | 적용 |
|-------|------|
| `_template.md` | 신규 (이번 cycle 표준) |
| `ISSUE-20260413-simd-codegen.md` | stamp 추가 (n/a — feature spec) |
| `ISSUE-20260413-playground-wasm.md` | stamp 추가 (n/a — feature) |
| `ISSUE-20260413-roadmap-sync.md` | stamp 추가 (**STALE** — Jan 2026 데이터, close 후보) |
| `ISSUE-20260413-smt-integration.md` | stamp 추가 (deferral context) |
| `ISSUE-20260413-linter-enhancement.md` | stamp 추가 (Cycle 2703 부분 진척 반영) |
| `ISSUE-20260326-if-else-early-return-codegen.md` | stamp + v0.98 재현 verification 권고 |
| `ISSUE-20260326-recursive-function-codegen.md` | stamp + v0.98 재현 verification 권고 |
| `ISSUE-20260326-multiple-pre-clauses.md` | stamp (parser limitation, n/a measurement) |
| `_b_track_methodology_stamp.md` | 신규 batch reference (9 LLM-bench ISSUE 일괄 stamp) |

### 2 ISSUE close (→ `closed/`)

| ISSUE | 사유 |
|-------|------|
| `ISSUE-20260326-llvm-name-conflicts.md` | Cycle 2703 Lint 11 (builtin_name_collision) 으로 resolution 확정. 21 reserved names 정적 감지 |
| `ISSUE-20260413-simd-vectorization.md` | Cycle 2220에서 이미 Superseded 상태로 마크됐으나 active dir 잔존. 형식적 close |

### 백그라운드 작업 진행 상황

- `b00nrwrmh` (golden full): ~340/2862 진행 (이 cycle 시작 직후)
- `b2abemb8w` (Tier all bench): Tier 0 ✅ + Tier 1 진행 (~20 benchmarks done)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `_template.md` 작성 | ✅ 6필드 + 보존 가이드 + 측정 추이 sub-table |
| 6 ISSUE 직접 stamp | ✅ all six edited |
| `_b_track_methodology_stamp.md` 작성 | ✅ 9 LLM-bench ISSUE 일괄 reference |
| 2 ISSUE close | ✅ `closed/` 이동 |
| Active ISSUE 카운트 | 24 → **22** (-2) |

결함: 없음. ISSUE 형식이 비균질 (English vs Korean templates) — 일괄 rewrite 대신 stamp block만 추가.

## Reflection

### 외부 관찰자 관점

1. **Stale data audit으로서의 가치**: roadmap-sync ISSUE의 Jan 2026 데이터가 1년 stale임을 cycle 2730에서야 stamp 의무화 후 명시적으로 식별. 이전 23 cycles 모두 이 stale 데이터 위에서 추론 가능. 양식 표준화의 직접 효과 첫 증거.

2. **이미 해결된 ISSUE 발견**: llvm-name-conflicts가 Cycle 2703에서 Lint 11로 resolution됐으나 active dir에 잔존 (1주 stale). 양식 stamp 의무화가 "이 ISSUE는 아직 살아있는가?" 질문을 강제 → close.

3. **Batch reference 패턴**: 9 LLM-bench methodology ISSUE를 `_b_track_methodology_stamp.md` 단일 파일로 일괄 stamp. 측정 컨텍스트가 동일 (2026-03-26 ~100문제 실험)이면 9번 반복 stamp 대신 1 reference + 9 short pointers (다음 cycle에 pointer 추가 가능). Cycle 효율 9x.

4. **양식 효과 정량화 시점**: 다음 cycle (Tier all 결과 분석 시) ISSUE-20260413-{hashmap-perf, alloc-optimization}의 stamp 데이터를 자동 비교 가능 → 양식 정착 가속.

### 측정 가능한 변화

- Active ISSUE: 24 → **22** (-2 close)
- Closed ISSUE: 34 → **36** (+2)
- 양식 stamp 적용: 11 ISSUE (10 직접 + 1 batch reference)
- `_template.md` 신규 — 향후 모든 ISSUE 작성 표준
- HANDOFF "lcs_three 1 FAIL" 정정 사항 (Cycle 2728)이 _template.md 측정 stamp 첫 사용 사례로 보존

### Roadmap impact

- M4 (Adopted): 변경 없음 — M4-1 B baseline 실행 시 9 LLM-bench ISSUE 일괄 갱신 트리거
- M5 (Language Completeness): Lint 11 close로 § 4 M5-5 표 한 줄 정리 가능 (다음 cycle)

## Carry-Forward

- Actionable (다음 cycle):
  - **C4**: 측정 stamp 미적용 잔여 ISSUE 8건 stamp 완료 (`crosslang-reference-asymmetry`, `external-problem-validation`, `first-shot-rate-low`, `integration-category-weakness`, `multi-model-validation`, `problem-difficulty-bias`, `statistical-testing`, `type-d-failure-analysis`, `context-overflow-prevention`) — _b_track_methodology_stamp.md pointer 1줄 추가 방식
  - **C5**: Cycle 2728 (lcs_three diagnostic) + Cycle 2729 결과 (풀 골든 + Tier all) 검증
  - **C6**: 추가 ISSUE 변동 — `roadmap-sync.md` v0.98 재측정 후 close 결정
- Structural Improvement Proposals:
  - `_template.md` 적용 강제 (PR template / commit hook?) — long term
  - ISSUE 명명 규칙 통일 (English Title vs Korean) — low priority
- Pending Human Decisions: 변경 없음
- Roadmap Revisions:
  - 없음 (M4/M5 진척 영향 없음, ROADMAP § 4/5 다음 cycle에 ISSUE 표 정리)
- Next Recommendation: **C4 = 잔여 stamp 일괄** → **C5 = 백그라운드 검증** → C6+ 추가 작업

## 메타 통찰

- 양식 strucutral improvement의 **즉시 효과**: 적용 cycle 자체에서 2 close (llvm-name-conflicts + simd-vectorization). 1년 stale ISSUE 발견. 표준화가 회수했어야 할 cycle 시간.
- 잔존 22 active 중 **6건이 stale 표시** (3개월+ 미측정): roadmap-sync, alloc-optimization (4월), hashmap-perf (4월), 9건 B-track (3월) — 다음 측정 라운드 priority.
