# Cycle 2767: HashMap 측정 검증 — 갭은 noise였음

Date: 2026-05-12

## Re-plan

진입 — cycle 2766 carry-forward 분기 ① (bootstrap-built 측정). Trigger ⚪ NONE.

## Scope & Implementation

### Step 1: 분기 ① bootstrap-built 측정 시도

```
bootstrap/compiler.exe build hash_table/bmb/main.bmb
→ returncode 3221225725 (= 0xC00000FD = STATUS_STACK_OVERFLOW)
```

CLAUDE.md known pattern. 부트스트랩 parser recursive descent의 stack 한계. hash_table bench의 deep nested let/if-else chain이 trigger.

→ 별도 ISSUE 등록: `ISSUE-20260512-bootstrap-parser-stack-overflow.md` (P3, Rule 6 path 차단 요인)

### Step 2: 분기 ① 우회 — A/B test with @inline

@inline 으로 `should_no_inline_for_licm` 우회 가능:
- bench source에 `@inline` 추가 → `explicit_inline=true` → `always_inline=true` → noinline pass excluded

A/B test (10-run min):

| 변종 | min | median | C 대비 |
|------|-----|--------|--------|
| BMB orig (`hm_get` noinline 자동) | 81.5 | 82.2 | **1.020x** |
| BMB `@inline` on `hm_get` | 80.5 | 82.1 | **1.018x** |
| C clang -O3 | 79.8 | 80.6 | 1.000x baseline |

@inline 효과: median 0.1ms 개선 (0.1%). **측정 노이즈 내부**.

### Step 3: 결론 — 갭은 noise

- **실측 갭 1.020x ≈ parity** (Cycle 2750 1.040x는 spurious +1.2pp noise)
- **noinline pass 영향 미미**: 0.2pp 미만
- **bootstrap port ROI 부정적**: 5-7 cycles → 0.2pp 개선 — 명백히 ROI 부정
- **C side fairness**: `static inline` 표기 없는 C 함수도 clang이 자동 inline. BMB에 explicit @inline 추가는 비대칭 → bench 변경 회피 (Principle 2)

### Step 4: ISSUE 갱신

- `ISSUE-20260413-hashmap-perf.md`: P1 → **P3** 강등, **close 후보** (실측 갭 < P-track 기준 1.05x)
- `ISSUE-20260512-bootstrap-parser-stack-overflow.md`: 신규 등록 (분기 ① 차단 원인)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Bootstrap build attempt | ✅ 실패 패턴 확인 (STATUS_STACK_OVERFLOW), 별도 ISSUE 등록 |
| Rust 측 빌드 + A/B 측정 (10-run min) | ✅ 81.5ms (orig) vs 80.5ms (@inline) vs 79.8ms (C) |
| `cargo test --release` | ✅ (변경 없음, 측정만) |
| 출력 정합 (BMB ↔ C) | ✅ 95259 / 100000 / 46445 동일 |
| bench source 변경 회피 | ✅ main_inline.bmb 제거 (Principle 2 정합) |

**Defect resolution**: A/B test 결과로 **HashMap "P1 4% 갭" 가설 기각**. Cycle 2750 측정값이 noise였음. Carry-forward: ISSUE close 권고.

## Reflection

### advisor leverage (세 번째 매칭)

advisor가 entry부터 "1.040x → 0.95x" 같은 expectation 근거 없음을 지적. cycle 2766 호출에서 "**갭 자체가 measurement noise일 가능성**" 가설 명시. cycle 2767 측정으로 가설 확인.

**meta-패턴**: ISSUE 본문 cycle estimate은 가설 — 이번 세션 3 cycle 연속 (2765, 2766, 2767) 같은 패턴. **이건 단발 미스가 아니라 구조적 issue**. `_template.md` 메타 필드 추가 권고 강화.

### 외부 관점 — 6 dimensions

1. **Scope fit**: 충족 — 1 cycle 진단 cycle. 분기 ② (compiler fix) 차단 결정 안전망 제공.
2. **Latent defects**: 
   - bootstrap parser stack overflow (별도 ISSUE)
   - 측정 framework 5-run noise floor 부적합 (Tier 1 8-9 algorithm에 일관)
3. **Structural improvement opportunities**:
   - `_template.md` 에 `estimated_cycles` 필드 + "hypothesis until verified" 명시 — meta-improvement (cycle 2765/2766/2767 패턴화)
   - bench output verification CI (cycle 2765/2766 누적 carry-forward)
   - measurement framework Tier 1 default 10-run (5-run noise floor 부적합)
4. **Philosophy drift**: 없음. bench source 변경 회피 (Principle 2). advisor 권고대로 bootstrap fix 회피 (ROI 부정).
5. **Roadmap impact**:
   - cycle-logs `ROADMAP.md` Phase B' 4-cycle plan을 **단축** (cycle 2766 진단 + cycle 2767 측정 + ISSUE close = 합 3 cycles, 잔여 0 cycles)
   - **잔여 4 cycles 재할당** 필요 (Cycle 2768-2773 6 cycles, 2774 종료)
6. **User-facing quality**: N/A.

### 가설 거부의 leverage

cycle 2766에서 진단 명료히 잡았으나 **expectation을 측정으로 검증하기 전까지 hypothesis**. 측정 결과: hypothesis (4% 갭 → @inline으로 2% 개선) 거부. 5-7 cycles compiler work 비용 회피. advisor의 "측정 우선 권고" + "1.040x 진짜? 측정 noise?" 질문이 정확.

## Carry-Forward

### Actionable (다음 cycle)

**Cycle 2768**: 잔여 cycles 재할당. 후보:
- **(a) ISSUE close + meta-improvement (2-3 cycles)**:
  - `ISSUE-hashmap-perf` close (실측 1.020x ≈ parity)
  - `_template.md` 메타 필드 (`estimated_cycles` + "hypothesis" 명시)
  - cycle-logs ROADMAP 갱신 (Phase B' 단축)
- **(b) or-chain proper fix (3-5 cycles)**: 직전 advisor warning — bootstrap regression risk. semantic correctness only ROI.
- **(c) bench output verification CI (1-2 cycles)**: lexer 0-token 같은 fairness 버그 회귀 방지
- **(d) Tier 3 workload amplification 잔여 5 benches (5-7 cycles, ISSUE에 Option A+B 결합 권고됨)**: 본 세션 budget 부족
- **(e) clang knapsack outlier 분석 (ISSUE-20260511, 장기)**: budget 부족

권고 (advisor 입력 안 받고 결정): **(a) + (c)** 조합 = 3-4 cycles. 잔여 1-2 cycles → 세션 종료. or-chain skip (advisor warning + ROI 부정).

### Structural Improvement Proposals

- **`_template.md` 메타 필드 추가** (`estimated_cycles`, `hypothesis_until_verified`): cycle 2765/2766/2767 3 연속 패턴 → 양식 자체 강화로 회귀 방지. 1 cycle.
- **bench output verification CI**: BMB ↔ C 출력 diff 자동. 1-2 cycles.
- **measurement framework Tier 1 default 10-run** (5-run noise floor 부적합): scripts/benchmark.sh 변경. 1 cycle.
- **bootstrap stack limit 증가 (`-Wl,--stack=...`)**: 단기 해법 1 cycle. 장기 (parser iterative 재작성)은 multi-cycle.

### Pending Human Decisions

- M3-3/M3-4 publish (HUMAN, 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 누적)
- Rule 6 / Rule 7 충돌 해법 (Rust 잔존 `should_no_inline_for_licm` 처리)
- bootstrap parser stack 한계 fix 시점 (Rule 6 path 차단)

### Roadmap Revisions

cycle-logs `ROADMAP.md`:
- Phase B' 4-cycle → **단축 (cycles 2766-2767 = 2 cycles)**. 잔여 5 cycles (2768-2773) + 종료 (2774) = 6 cycles 재할당
- Phase C' (or-chain) → **skip** (advisor warning + 측정 검증 결과 P2 잔류)
- 새 plan: 잔여 5 cycles에 (a) ISSUE close + meta + (c) bench output CI 분배

### Next Recommendation

**Cycle 2768**: 
1. ISSUE-hashmap-perf close (Move to issues/closed/)
2. `_template.md` 메타 필드 추가 (estimated_cycles + hypothesis)
3. Other ISSUE batch review for stale-after 영향 (cycle 2735 양식 100% 적용 확인)

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| ISSUE-hashmap-perf P1→P3 + Cycle 2767 측정 결과 | `claudedocs/issues/ISSUE-20260413-hashmap-perf.md` | tracked |
| 신규 ISSUE: bootstrap parser stack overflow | `claudedocs/issues/ISSUE-20260512-bootstrap-parser-stack-overflow.md` | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2767.md` | gitignored |
