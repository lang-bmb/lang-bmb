# Cycle 2768: ISSUE-hashmap close + _template.md meta-improvement

Date: 2026-05-12

## Re-plan

진입 — cycle 2767 carry-forward (ISSUE close + 양식 강화). Trigger ⚪ NONE.

## Scope & Implementation

### Step 1: ISSUE-hashmap-perf close

`claudedocs/issues/ISSUE-20260413-hashmap-perf.md` → `claudedocs/issues/closed/`:
- 우선순위 P3 (이미 cycle 2767에서 강등)
- 상태 "Closed (Cycle 2768) — 실측 1.020x ≈ parity, P-track 기준 1.05x 내부. compiler fix ROI 부정 (5-7 cycles → 0.2pp)"
- Active ISSUE 17 → 16, Closed 누적 44 → 45

### Step 2: `_template.md` 메타 필드 추가

cycle 2765/2766/2767 3 연속 패턴: ISSUE 본문 cycle estimate (1-2)이 실측 (3-7) 차이 ≥1.5x. 회귀 방지로 양식 강화.

추가 사항:
- **양식 변경 이력**: 2026-05-12 (Cycle 2768) 추가
- **해결 방안 섹션 헤더**: "모든 `scope: N cycles` 추정은 검증 전까지 가설" 명시
- Option A/B 항목: `scope` → `estimated_cycles` (이전과 의미 동일하나 명명 일관) + "(hypothesis — verify via 진단 cycle)" 부착
- **양식 보존 가이드 #6 추가**: 1 cycle 진단 cycle 먼저 → 가설 적합성 확인 후 implementation. 추정과 실측 차이 ≥1.5x 시 ISSUE 본문 갱신 + 우선순위 재평가

### Step 3: 기존 ISSUE 배치 점검 (적용 회피)

17 active ISSUEs 모두 새 `estimated_cycles` 필드 없음. **Backfill 회피**:
- 각 ISSUE re-estimate은 데이터 없이 추정 반복 — noise
- 양식 변경은 forward-looking. 새 ISSUE / next revision 시 적용
- 기존 ISSUE는 `scope: N cycles` 표기 유지 (의미 동일)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| ISSUE 이동 (`closed/` 디렉토리) | ✅ |
| Active ISSUE 카운트 17 → 16 | ✅ |
| `_template.md` 변경 일관성 (3 sections) | ✅ |
| `cargo test --release` | ✅ |

**Defects**: 없음 (문서/양식 변경 only).

## Reflection

### advisor leverage (메타-개선)

cycle 2767 advisor 권고:
> "세 번째 같은 패턴이 나타나면 (이번 세션 두 번째 cycle만에 두 번 확인), `claudedocs/issues/_template.md` 자체에 'cycle estimate은 검증 전까지 가설' 필드 추가 권고. 이건 메타-개선."

세 번째 패턴 확인 (cycle 2767) + cycle 2768에서 즉시 메타-개선 적용. advisor "패턴 → 양식 강화" 권고 leverage.

### 외부 관점 — 6 dimensions

1. **Scope fit**: 충족 — cycle 2767 carry-forward 그대로 처리.
2. **Latent defects**: 없음.
3. **Structural improvement opportunities**:
   - 잔여 카리포워드: bench output verification CI (cycle 2765-2767 누적). 1-2 cycles 실행 가능
   - measurement framework Tier 1 default 10-run 변경
4. **Philosophy drift**: 없음. ISSUE close는 측정 우선 (Verification Principle) + 양식 강화는 measurement integrity 강화.
5. **Roadmap impact**:
   - cycle-logs `ROADMAP.md` Phase B' 완료 (cycles 2766-2768 = 3 cycles, 당초 5 cycles plan)
   - 잔여 5 cycles (2769-2773) + 종료 (2774). plan: bench output CI + 잔여 carry-forward 처리
6. **User-facing quality**: N/A.

### 메타-개선의 leverage

3 cycle 연속 ISSUE estimate-vs-실측 갭 패턴 (cycle 2765 lexer DCE, cycle 2766 HashMap, cycle 2767 bootstrap stack overflow) → 패턴화 → 양식 변경. 이번 변경으로 향후 ISSUE 작성 시 estimate verification 의무화 → meta-cycle (cycle 자체의 cycle).

## Carry-Forward

### Actionable (다음 cycle)

**Cycle 2769**: bench output verification CI (cycle 2765에서 lexer 0-token 같은 unfairness 회귀 방지)
- BMB ↔ C 출력 diff 자동 검사 도구 작성
- Tier 1/3 모든 bench에 적용 가능한 형태
- CI 통합은 별도 cycle

### Structural Improvement Proposals

- **measurement framework Tier 1 default 10-run**: cycle 2725 Tier 3 5-run noise 문제 재확인. 1 cycle.
- **bootstrap stack limit 증가** (`-Wl,--stack=...`): hash_table-style deep nesting 회피. 1 cycle.
- **기존 ISSUE backfill `estimated_cycles`**: noise risk. carry-forward (다음 ISSUE revision 시 자연 적용).

### Pending Human Decisions

- M3-3/M3-4 publish (HUMAN, 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 누적)
- Rule 6 / Rule 7 충돌 (Rust 잔존 `should_no_inline_for_licm`)
- bootstrap parser stack 한계 fix 시점

### Roadmap Revisions

cycle-logs `ROADMAP.md`:
- Phase B' 완료 (cycles 2766-2768, ISSUE close + meta-improvement). 당초 plan 4 cycles → 실제 3 cycles.
- 잔여 plan: 
  - Cycle 2769-2770: bench output verification CI
  - Cycle 2771-2773: 카리포워드 (Tier 3 amp 5 benches OR or-chain OR bootstrap stack OR clang knapsack)
  - Cycle 2774: 세션 종료

### Next Recommendation

**Cycle 2769**: bench output verification CI 도구 작성 시작. scope:
1. 도구 위치: `scripts/verify_bench_outputs.py` 또는 BMB로 작성
2. 입력: bench dir (BMB exe + C exe pairs)
3. 출력: diff matrix (per bench, BMB vs C 출력 일치/불일치)
4. CI 통합 위치: `.github/workflows/`

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| ISSUE-hashmap-perf 이동 + 상태 "Closed" | `issues/closed/ISSUE-20260413-hashmap-perf.md` | tracked (이동) |
| `_template.md` 양식 변경 이력 + estimated_cycles 강화 (3 sections) | `claudedocs/issues/_template.md` | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2768.md` | gitignored |
