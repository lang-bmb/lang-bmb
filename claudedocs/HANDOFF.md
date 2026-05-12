# BMB Session Handoff — 2026-05-12 (Cycles 2765-2773 — bench verify infrastructure + P0 store_u8 bug)

> **HEAD**: `45a96748` (Cycle 2774 commit — bench verify infrastructure + P0 store_u8 diagnosis)
> **이전 세션 핸드오프**: Cycle 2764 (`e98669fa`) — M3-5 honest re-baseline
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이번 세션 진입점**: Cycle 2765 (Phase A Tier 3 workload amplification 시도)
> **이번 세션 cycle logs**: gitignored (disk only)

---

## 0. 이번 세션 작업 (Cycles 2765-2773, 9 cycles, infrastructure + diagnosis 중심)

### 🚨 중대 발견 — **P0 store_u8 silent correctness bug**

Cycle 2772에서 발견: `store_u8(buf + pos, c)` 패턴이 함수 인자 buf + pos 케이스에서 잘못된 base 선택 (pos을 base ptr로 inttoptr)으로 LLVM이 store를 UB로 간주하여 제거. **compile success + run success + 잘못된 출력**.

- 영향: json_serialize bench `Array: {1,2,3,4,5]` (정상: `[1,2,3,4,5]`)
- 잠재 영향: 다른 bench/bootstrap에서 동일 패턴 사용 시 silent 회귀
- ISSUE: `claudedocs/issues/ISSUE-20260512-store_u8-null-ptr-base.md` (P0, estimated 3-5 cycles, Rule 6 충돌 검토 필요)

### 🛠️ 신규 인프라 — bench output verification

Cycle 2769에서 `scripts/verify_bench_outputs.py` 작성 (240 LOC):
- BMB ↔ C bench 출력 정합 자동 검사 (Tier 1/3 17 benches)
- 1차 측정에서 **6개 결함 즉시 발견** (도구의 가치 입증)
- Cycle 2771: `scripts/full-cycle.sh` 에 Step 3.5로 통합 (non-blocking)

### Cycle-by-cycle 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2765 | Tier 3 workload amplification POC | Option A 한계 발견 (BMB CSE/purity inference로 outer-loop DCE). lexer + brainfuck 부분 적용. ISSUE-tier3-spawn-overhead 갱신 + 새 ISSUE bmb-lexer-0-token 등록 |
| 2766 | HashMap 진단 | `hm_get` 만 noinline (Cycle 2532 MemoryEffectAnalysis pass). Rule 6 충돌 발견 (bootstrap에 동일 pass 없음) |
| 2767 | HashMap 측정 검증 | **갭은 noise였음** (advisor 가설 우월). 실측 1.020x ≈ parity. bootstrap-built 시도 → STATUS_STACK_OVERFLOW (별도 ISSUE) |
| 2768 | ISSUE-hashmap close + 양식 강화 | P1 → P3 강등, closed/ 이동. `_template.md` 에 `estimated_cycles` + hypothesis 필드 추가 (cycle estimate 갭 패턴 회귀 방지) |
| 2769 | verify 도구 작성 + 1차 측정 | 17/17 측정 → 11 PASS, 4 unfairness, 2 fail. **6 결함 즉시 catch** |
| 2770 | sorting 재빌드 회귀 진단 | **P1 ~500× 슬로다운** (Rust compiler 회귀, Feb 9 main.exe 정상 vs May 12 rebuild 시 hang) |
| 2771 | verify CI 통합 | `full-cycle.sh` Step 3.5 추가 (`--skip-verify` opt, non-blocking 동작) |
| 2772 | json_serialize char bug | **P0 store_u8 silent UB 발견**. workaround 불가, multi-cycle fix 필요 |
| 2773 | HANDOFF/ROADMAP 갱신 | 본 cycle (commit 직전) |

### advisor leverage (세 번째 메타-패턴 등 4건)

- **Cycle 2765**: Option A 비현실성 + scope 축소 + HashMap 우선순위 권고
- **Cycle 2766**: HashMap "1.040x → 0.95x" expectation 근거 부재 지적 + measurement-first 권고
- **Cycle 2767**: 분기 ① 측정 후 부정 결과 → bootstrap port ROI 부정 결정
- **Cycle 2772 (메타)**: 도구가 P0 bug 즉시 catch — measurement integrity infrastructure 효과 누적 검증

**Meta-pattern**: ISSUE 본문 cycle estimate은 검증 전까지 가설. 3 cycle 연속 같은 패턴 → `_template.md` 메타 필드 추가 (cycle 2768).

---

## 1. 현재 상태

### Bootstrap 검증 상태 (변경 없음)

| 게이트 | 결과 (Cycle 2718) |
|--------|------------------|
| Stage 1 | ✅ 10.8s |
| Stage 2 (32G arena) | ✅ 29.2s |
| Stage 3 | ✅ 36.7s |
| **Fixed Point S2 == S3** | ✅ **유지** |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ (BMB compiler 변경 없음) |
| **신규 verify_bench_outputs.py** | ⚠️ 11/17 PASS (Tier 1 8/10 + Tier 3 3/7) |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated + Bootstrap | ✅ COMPLETE + 회복 |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~99% (HUMAN publish dispatch 잔여) |
| M4 Adopted | 🔄 ~50% |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ |

### ISSUE 백로그 변화 (Cycles 2765-2772)

| 시점 | active | closed |
|------|--------|--------|
| 시작 (Cycle 2764 종료) | 16 | 44 |
| 종료 (Cycle 2772) | **22** | **45** |

**Close 1건**: `hashmap-perf` (실측 1.020x ≈ parity)

**신규 등록 6건** (이번 세션):
| ISSUE | 우선순위 | scope |
|-------|---------|-------|
| `bmb-lexer-bench-zero-tokens` | P2 | lexer count_tokens 0-token correctness bug (cycle 2765) |
| `bootstrap-parser-stack-overflow` | P3 | hash_table size source가 bootstrap STATUS_STACK_OVERFLOW (cycle 2767) |
| `bench-output-fairness-survey` | P2 | 통합 ISSUE — verify 도구 발견 결함 6건 종합 (cycle 2769) |
| `sorting-rebuild-regression` | **P1** | sorting 재빌드 시 ~500× 슬로다운 (cycle 2770) |
| `store_u8-null-ptr-base` | **P0** | silent UB: pos가 base로 선택, pos=0 시 null base GEP (cycle 2772) |

추가로 `_template.md` 양식 강화 (cycle 2768): `estimated_cycles` + hypothesis 필드 + 양식 보존 가이드 #6.

---

## 2. 태스크 목록 (다음 세션 진입)

### 분기 P0 — store_u8 silent correctness bug (Cycle 2772 발견)

| # | 태스크 | 성격 |
|---|--------|------|
| P0.1 | `bmb/src/codegen/{llvm,llvm_text}.rs` store_u8 lowering audit | 자율 |
| P0.2 | 첫 operand을 base로 선택하는 휴리스틱 fix | **HUMAN 결정 (Rule 6 vs P0)** |
| P0.3 | bootstrap에 동일 패턴 audit | 자율 |
| P0.4 | text + inkwell 양쪽 fix (Rule 7) | 자율 (HUMAN approval 후) |
| P0.5 | 골든 테스트 추가 (`store_u8(arg_a + 0, c)`) | 자율 |
| P0.6 | full Tier 1/3 verify PASS 확인 | 자율 |

estimated_cycles: 3-5 (hypothesis per template). Rule 6 충돌 — HUMAN 결정 권장.

### 분기 P1 — sorting 재빌드 회귀 (Cycle 2770 발견)

| # | 태스크 | 성격 |
|---|--------|------|
| P1.1 | git log on `bmb/src/codegen/` + `bmb/src/mir/` Feb-May commits | 자율 |
| P1.2 | bisect (각 commit으로 sorting build + time) | 자율 |
| P1.3 | 회귀 commit 식별 → fix path 결정 | HUMAN 결정 |

estimated_cycles: 3-7 (hypothesis).

### 분기 B (HUMAN dispatch) — Publish (변경 없음, 이전 세션 누적)

- B.1-B.5: npm publish + PyPI publish (M3-3/M3-4)

### 분기 C — M4-1 B baseline (변경 없음)

- C.1 `BMB_BENCH_API_KEY` HUMAN setup
- C.2-C.5 자율 실행

### 추가 자율 백로그 (이번 세션 carry-forward)

| # | 태스크 | 성격 | 추정 |
|---|--------|------|------|
| 1 | Tier 3 workload amplification 잔여 5 benches | 자율 (or Option B inproc port) | 5-10 cycles |
| 2 | `--with-verify` opt-in to quick-check.sh | 자율 | 1 cycle |
| 3 | GitHub workflow에 verify step 추가 | 자율 + HUMAN approval | 1 cycle |
| 4 | FP tolerance epsilon arg (`verify_bench_outputs.py`) | 자율 | 1 cycle |
| 5 | full-cycle.sh first-real-run 검증 | 자율 | 1 cycle |
| 6 | clang knapsack outlier 분석 (ISSUE 기존) | 자율 (장기) | multi |
| 7 | sub-ISSUE 처리 (csv_parse / lexer / fibonacci / n_body 각각) | 자율 | 각 2-5 cycles |

---

## 3. 핵심 산출물 (이번 phase, Cycles 2765-2773)

### Code 산출

- `scripts/verify_bench_outputs.py` (신규, 240 LOC) — BMB ↔ C bench 출력 정합 검사
- `scripts/full-cycle.sh` — Step 3.5 verify 통합 (`--skip-verify` opt, non-blocking)
- `ecosystem/benchmark-bmb/benches/real_world/lexer/c/main.c` — 100x→1000x scaling + buffer 500K
- `ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main.bmb` — 100x→1000x scaling
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/c/main.c` — 9→99 outer loop
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main.bmb` — 9→99 outer loop

### Documentation 산출

- 6 신규 ISSUE 등록 (P0 1, P1 1, P2 3, P3 1)
- `claudedocs/issues/_template.md` 양식 강화 (estimated_cycles + hypothesis)
- `claudedocs/issues/closed/ISSUE-20260413-hashmap-perf.md` 이동 (P1 → P3 close)
- `claudedocs/ROADMAP.md` 갱신 (TBD: ROADMAP 미갱신, cycle 2773에서 진행)

### Measurement 산출

- Tier 1 verify: 8/10 PASS (`/tmp/verify_tier1.json`)
- Tier 3 verify: 3/7 PASS (`/tmp/verify_tier3.json`)
- hash_table A/B 측정: BMB orig 82.2ms / @inline 82.1ms / C 80.6ms (median, 10-run)

---

## 4. 환경 노트 (변경 없음)

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` (origin 4 commits ahead) |

### 운용 주의사항 (NEW 이번 세션)

- **`scripts/verify_bench_outputs.py` 도구**: BMB compiler 변경 후 측정 전 `python3 scripts/verify_bench_outputs.py --tier all --rebuild` 실행 권장 (silent 회귀 catch)
- **`store_u8(arg + pos, c)` 패턴 회피**: P0 bug 잔존 시까지. 가능한 경우 `store_u8(arg + non_zero_offset, c)` 또는 local var 통해 사용
- **bootstrap parser stack 한계**: deeply nested AST 회피 (hash_table 패턴) — bootstrap port 시 STATUS_STACK_OVERFLOW
- 이전 세션 운용 주의사항 그대로:
  - bmb-algo bench median-of-N (`bench_algo.py --runs=5`)
  - Tier 3 spawn overhead methodology (ISSUE-20260512)
  - bmb-algo submodule 아님
  - BMB_ARENA_MAX_SIZE default 32G
  - Token packing 5M scale
  - FP builtin arity guard 미적용

---

## 5. 다음 세션 시작 체크리스트

### 기본 검증
- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커, cycle 2773-2774 갱신)
- [ ] `cargo test --release` (옵션, BMB compiler 변경 없음)
- [ ] **신규**: `python3 scripts/verify_bench_outputs.py --tier all --rebuild` (≈ 165s, 측정 전 권장)

### P0 분기 진입 (`store_u8` silent bug)
- [ ] Rule 6 vs P0 결정 (HUMAN)
- [ ] `bmb/src/codegen/{llvm,llvm_text}.rs` audit
- [ ] `bootstrap/compiler.bmb` 동일 패턴 grep
- [ ] 첫 operand을 base로 fix → text + inkwell 동시 → verify PASS

### P1 분기 진입 (sorting 재빌드 회귀)
- [ ] git log 점검 (Feb-May commits)
- [ ] bisect 자동화 script 작성 또는 manual

### Publish 진입 (분기 B, M3-5 완결 그대로)
- [ ] (이전 세션 그대로) `gh workflow run npm-publish.yml -f dry_run=true`
- [ ] ... (전 세션 동일)

---

## 6. HUMAN 결정 사항 누적

| 항목 | 결정 |
|------|------|
| M3 showcase | ✅ bmb-algo |
| npm publish | ✅ 즉시 dispatch 가능 |
| PyPI publish | ✅ 즉시 dispatch 가능 |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 모델 | ✅ claude-sonnet-4-6 |
| M3-5 README/CHANGELOG | ✅ Cycles 2760-2764 완결 |
| **신규 P0 store_u8 fix Rule 6 검토** | ⏸️ **HUMAN 결정 필요** |
| **신규 P1 sorting rebuild fix Rule 6 검토** | ⏸️ **HUMAN 결정 필요** |
| 이번 세션 자율 결정:
| - `_template.md` 양식 강화 (estimated_cycles) | ✅ |
| - `scripts/verify_bench_outputs.py` 신규 도구 | ✅ |
| - `scripts/full-cycle.sh` Step 3.5 통합 | ✅ |
| - `hashmap-perf` ISSUE close (P3) | ✅ |
| - lexer/brainfuck workload amp partial | ✅ (POC, limited effect) |

---

## 7. 이번 phase의 메타 통찰

### 1. measurement integrity infrastructure의 누적 효과

| Cycle | 단계 | 결과 |
|-------|------|------|
| 2768 | ISSUE 양식 강화 (estimated_cycles + hypothesis) | 메타 회귀 방지 |
| 2769 | verify 도구 (240 LOC) | 즉시 6 결함 catch |
| 2771 | CI 통합 (Step 3.5) | 회귀 자동 alert |
| 2772 | 도구 효과 **P0 silent bug 식별** | **존재 자체가 정당화** |

3 cycles 작업이 **P0 발견**으로 이어짐. ROI 매우 높음.

### 2. advisor의 가설 거부 leverage

cycle 2766 advisor: "HashMap 4% 갭 자체가 measurement noise일 가능성"
cycle 2767 측정: 1.040x → 1.020x (real). 가설 정확. 5-7 cycles compiler work 비용 회피.

이는 advisor가 **expectation 추정 거부**의 시스템적 패턴. 향후 ISSUE estimate 평가 시 적용.

### 3. P-track 측정 신뢰도의 메타-우려

verify 도구 발견:
- **4/17 = 24% benches가 unfair comparison** (csv_parse, json_serialize, lexer, n_body)
- **2/17 = 12% benches가 build/run 회귀** (sorting hang, fibonacci C fail)

기존 P-track 측정 통합 (`tier_all_c2729.json` 등)이 이런 unfairness 위에 구축됨. **재측정 + verify 통과 확인 후 ratio 활용** 표준 권고.

### 4. Rule 6 / Rule 7 한계

이번 세션 3 가지 결함이 Rule 6 충돌:
- `should_no_inline_for_licm` (Rust 잔존) — 결국 ROI 부정 (cycle 2767)
- `sorting` 재빌드 회귀 (Rust codegen) — fix 차단
- `store_u8` silent bug (Rust codegen) — P0 fix 차단

Rule 6 (Rust frozen) + Rule 7 (parity) 와 P0 correctness 사이의 우선순위 정책 검토 필요 (HUMAN).

### 5. 진단 cycle 우선 정책의 가치

원 plan: cycle 2767 HashMap fix attempt (3-5 cycles).
실제: cycle 2766 진단 + cycle 2767 측정 검증 → 가설 거부 → 2 cycle만 소비. 5-7 cycles 회피.

advisor 권고 "1 cycle 진단 cycle 먼저" 패턴 검증. `_template.md` 양식 강화 (cycle 2768) 정합.

---

## 8. 다음 세션 첫 cycle 권고

### Cycle 2775 — 분기 결정

HUMAN 검토 후 분기:
- **분기 P0**: `store_u8` silent bug fix (Rule 6 우회 정책 결정 필요)
- **분기 P1**: sorting bisect (Rule 6 우회 정책 결정 필요)
- **분기 B**: publish dispatch (Rule 6 무관, 즉시 진입 가능)
- **분기 C**: M4-1 B baseline (HUMAN setup 필요)

자율 진입 가능 cycle:
- bootstrap에 store_u8 동일 패턴 audit (Rule 6 영향 없음, grep + 분석)
- verify 도구 enhancement (epsilon arg + opt-in to quick-check)

---

**세션 종료**: 2026-05-12 (Cycles 2765-2773 — bench verify infrastructure + P0 store_u8 bug 진단). HEAD `45a96748` (Cycle 2774 commit).
