# Cycle 2765: Tier 3 workload amplification POC + 한계 발견

Date: 2026-05-12

## Re-plan

진입: ROADMAP cycle-logs Phase A (Tier 3 workload amp, 2 cycles 예정).

**Mid-cycle 🟠 RE-PLAN trigger** (advisor 자문 호출):

ISSUE-tier3-spawn-overhead Option A 추정 "1-2 cycles"가 비현실. BMB의 LLVM optimizer는 `bmb_black_box(pure_fn(invariant_input))` 패턴에서 fn을 invariant로 hoist하여 outer-loop amplification DCE. 단일 input scaling은 작동하나 bench 별 다른 패턴 필요. 5-10 cycles scope.

**대응**: Phase A POC 축소 (lexer + brainfuck 두 bench만) → HashMap (P1 측정 갭) 우선 진행 권고 적용. Plan shift:
- 원: Cycle 2765-2766 Phase A → Cycle 2767-2770 HashMap
- 신: Cycle 2765 Phase A POC + ISSUE 갱신 → Cycle 2766 HashMap 진단 (한 칸 앞당김)

## Scope & Implementation

### Step 1: 진단 + design

7 Tier 3 benches 조사:
- brainfuck: outer iter 9 (BMB+C)
- http_parse: outer iter 10000 (C), N/A (BMB)
- json_parse: outer iter 100000 (C)
- json_serialize: outer iter 10000 (C)
- lexer: single count_tokens(large) 호출, large=gen_large(100)
- sorting: 이미 200× outer loop (workload-dominant 가능성)
- csv_parse: outer iter 없음

### Step 2: lexer 시도 (POC)

**시도 1 — outer-loop amplification (BMB+C)**: 100× outer loop  
- C amp100: 5.5→21.3ms (workload +16ms, 0.16ms/iter)
- BMB amp100 plain: 6.2ms (DCE 완전)
- BMB amp100 + `bmb_black_box(count_tokens(large))`: 8.2ms (+1.7ms, hoist 부분 차단)

→ BMB outer loop은 LLVM CSE/purity inference가 hoist. `bmb_black_box`를 return에만 걸어서는 input invariance 차단 불충분.

**시도 2 — input scaling**: gen_large(100) → gen_large(1000), single call, C 버퍼 100000→500000
- C amp1000: 5.5→12.9ms (workload +7ms)
- BMB amp1000: 6.5→6.8ms (변화 거의 없음)

→ C는 효과 있음. BMB는 **별도 0-token 버그** 발견 (이하 step 3).

### Step 3: BMB lexer 0-token 버그 (pre-existing)

`count_tokens()` 가 모든 입력에 대해 0 반환. small source도 모든 카운터 0. C는 정상 ident=20 / kw=12 출력. git stash 검증으로 my 변경 이전부터 존재 확인.

새 ISSUE 등록: `claudedocs/issues/ISSUE-20260512-bmb-lexer-bench-zero-tokens.md` (P2, measurement fairness 영향)

### Step 4: brainfuck POC

- BMB 9→99 (10×): 6→7.6ms (+1.6ms)
- C 9→99 (10×): 6→6.9ms (+0.9ms)

inner work <0.1ms/iter → 10× amp가 workload dominance 달성 불충분. 100× 이상 필요. 본 cycle 변경은 그대로 유지 (modest improvement).

### Step 5: ISSUE 갱신

`ISSUE-20260512-tier3-spawn-overhead-methodology.md` 에 Cycle 2765 empirical findings 추가:
- Option A 단독 비현실 (BMB CSE 문제)
- scope 1-2 → 5-10 cycles 정정
- Option A + Option B (inproc) 결합 권고

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210/6210 PASS (BMB compiler 변경 없음) |
| 빌드 sanity (lexer C/BMB, brainfuck C/BMB) | ✅ 4/4 모두 빌드 |
| 출력 baseline (변경 후 회귀 없음) | ✅ C 측 정상 token counts, BMB 측 0-token 버그는 pre-existing |
| ISSUE 양식 (`_template.md` 정합) | ✅ 두 ISSUE 모두 measurement_date / stale_after / source 등 포함 |

**Defects 발견** (이번 cycle):
- BMB lexer count_tokens 0-token 버그 (pre-existing, 별도 ISSUE 등록)
- Option A scope 추정 비현실 (ISSUE 갱신)

**Defect resolution**: 두 가지 모두 본 cycle scope 외 (latent + scope correction). 양쪽 모두 ISSUE 등록 + carry-forward.

## Reflection

### 외부 관점 — 6 dimensions

1. **Scope fit**: 부분 — Phase A 2 cycle plan을 1 cycle POC로 축소. brainfuck/lexer 두 bench 변경 유지하나 broader scope 미완.
2. **Latent defects**: BMB lexer 0-token 버그 발견 (pre-existing). 별도 ISSUE.
3. **Structural improvement opportunities**:
   - Tier 1 inproc 패턴(`bmb_black_box(input_seed)` 매 iter) 그대로 Tier 3 포팅 — Option B 정합. 5-10 cycles 분리 phase 후보.
   - bench output verification CI gate (BMB ↔ C 출력 diff 자동) — `_template.md` 옆에 메소드 정합 점검 도구로 정착 가능.
4. **Philosophy drift**: 없음 — methodology 정합/measurement integrity는 BMB 철학 정렬 ("측정 없는 성능 주장 금지").
5. **Roadmap impact**:
   - Phase A 2 cycles → 실제 5-10 cycles. cycle-logs ROADMAP Phase A 축소 + 신규 carry-forward 명시 필요
   - HashMap 진단을 Cycle 2766으로 한 칸 앞당김 (당초 2767)
6. **User-facing quality**: N/A — 내부 measurement methodology만 영향.

### advisor leverage

cycle 중반 advisor 호출 (Option A 측정 미스매치 발견 시점)이 결정적. "Phase B 4-cycle budget의 현실성" entry 권고가 또 한 번 검증됨 — **ISSUE 본문의 cycle 추정은 실측 검증 전까지 가설**. 일반화: 매 phase 시작 시 첫 cycle을 가설 검증 cycle로 운영 권고.

### Plan shift 정당화

advisor 권고 옵션 ① (POC 축소 → HashMap 사수) 선택 근거:
- HashMap = P1 측정 4% 갭 (정량 ROI 명확)
- Tier 3 methodology = P2 awareness (정성 ROI)
- 옵션 ② (Option B inproc 전환)은 10-cycle session 전체 소비 → HashMap + or-chain 모두 미완

## Carry-Forward

### Actionable (다음 cycle)

- **Cycle 2766 (당초 2767 → 한 칸 앞당김)**: HashMap 소스 진단 (stdlib/collections/ 또는 runtime). 해시 함수 식별 + collision strategy + IR diff (BMB vs C hash_table)

### Structural Improvement Proposals

- **Tier 3 inproc 포팅 (5-10 cycles, 분리 phase)**: ISSUE-tier3-spawn-overhead Option A+B 결합. 본 세션 잔여 cycles 부족, 다음 세션 또는 별도 세션 후보. 추정 5-10 cycles per 7 benches.
- **bench output verification CI**: BMB ↔ C 출력 diff 자동 검사. lexer 0-token 같은 fairness 버그 회귀 방지. 1-2 cycles.

### Pending Human Decisions

- M3-3/M3-4 publish dispatch (HUMAN, 이전 세션부터 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 이전 세션부터 누적)
- **신규**: Tier 3 inproc 전환 시점 결정 (multi-cycle phase, HUMAN scope confirmation)

### Roadmap Revisions

cycle-logs `ROADMAP.md` Phase A 갱신:
- 원: Cycles 2765-2766 (2 cycles, Option A workload amp)
- 신: Cycle 2765 (1 cycle, POC + ISSUE 갱신 + 한계 발견)
- HashMap 진단 (당초 2767) → Cycle 2766으로 한 칸 앞당김
- 잔여 cycles 2766-2770 HashMap (5 cycles), 2771-2773 or-chain (3), 2774 종료 (1)

### Next Recommendation

**Cycle 2766**: HashMap 소스 진단 + 해시 함수 식별. budget realism 점검 (advisor 권고): 첫 cycle에서 scope 추정 실측 검증.

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| lexer C: 100x→1000x scaling + buffer 100000→500000 | `ecosystem/benchmark-bmb/benches/real_world/lexer/c/main.c` | submodule modified |
| lexer BMB: 100x→1000x scaling | `ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main.bmb` | submodule modified |
| brainfuck C: outer 9→99 | `ecosystem/benchmark-bmb/benches/real_world/brainfuck/c/main.c` | submodule modified |
| brainfuck BMB: outer 9→99 | `ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main.bmb` | submodule modified |
| ISSUE 갱신 (tier3-spawn-overhead) | `claudedocs/issues/ISSUE-20260512-tier3-spawn-overhead-methodology.md` | tracked |
| 새 ISSUE (bmb-lexer-bench-zero-tokens) | `claudedocs/issues/ISSUE-20260512-bmb-lexer-bench-zero-tokens.md` | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2765.md` | gitignored |
