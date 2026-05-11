# BMB Session Handoff — 2026-05-11 (Cycles 2718-2727 — P-track 대규모 triage + Bootstrap 안정성)

> **HEAD**: (이번 commit으로 갱신 예정)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: cycle-logs/cycle-2708~2717.md 참조

---

## 0. 이번 세션 작업 (Cycles 2718-2727)

### 🎉 큰 변곡점

| 항목 | Before | After |
|------|--------|-------|
| **P-track ISSUE 측정 데이터 신선도** | 1년 stale (2026-04-13 기준) | ✅ **2026-05-01/02 측정 데이터로 일괄 갱신** |
| **sorting (compare-inline)** | 110% slow | ✅ **0.910x BMB 9% FASTER** (close) |
| **lexer (match-jump-table)** | 109% slow | ✅ **1.000x parity** |
| **brainfuck** | 111% slow | 1.036x close |
| **hash_table** | 111% slow | 1.027x (8 pp 개선, carry-forward) |
| **active ISSUE 백로그** | 25 (cycle 2716 triage 후) | **23** (3 close + 1 new) |
| **CI 회귀 안전망** | bootstrap_3stage만 | bootstrap_3stage + **golden 50 sample** (Cycle 2720) |
| **bootstrap S2 == S3** | Stage 2 회복 직후 (cycle 2711-2714) | ✅ **유지** (Cycle 2718 재검증) |

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2718 | 시퀀스 A: 회귀 안전망 | cargo test 6210/6210 + bootstrap S2==S3 + 풀 골든 백그라운드 시작 |
| 2719 | 시퀀스 D: ISSUE 정리 | 16 resolved → closed/ (active 41→25) |
| 2720 | 시퀀스 D: CI gate | golden sample 50 → bootstrap-benchmark.yml에 통합 |
| 2721 | **RE-PLAN**: P-track backlog 평가 | 5 ISSUE 평가, FP guard → cycle 2726 deprioritize |
| 2722 | Match jump table 재진단 | brainfuck jump table 작동 (53 LJTI), lexer는 `or` chain 진짜 원인 |
| 2723 | `or` chain lowering 분석 | proper fix multi-cycle scope, 새 ISSUE 등록 |
| 2724 | StringBuilder SSO 진단 | fasta가 SB 미사용 (false positive). pivot to bulk re-measurement |
| 2725 | **Tier 1 bulk re-measurement** | historic + tier3-10runs 데이터 활용 → 3 ISSUE close + 3 갱신 |
| 2726 | **Rule 9 조기 종료** | FP arity guard carry-forward (low ROI, 36 사이트 부담) |
| 2727 | Closeout | ROADMAP § 5 갱신 + HANDOFF + commit (현재) |

---

## 1. 현재 상태

### Bootstrap 검증 상태 (Stage 2 회복 안정)

| 게이트 | Cycle 2718 결과 |
|--------|-----------|
| Stage 1 (Rust → BMB₁) | ✅ 10.8s |
| Stage 2 (BMB₁ → LLVM IR, 32G arena) | ✅ 29.2s |
| Stage 3 (BMB₂ → LLVM IR) | ✅ 36.7s |
| **Fixed Point S2 == S3** | ✅ **유지** |
| Total | 77.4s |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ **6210/6210 passed** (Cycle 2718) |
| Bootstrap Fixed Point | ✅ S2 == S3 |
| 풀 골든 (background, Cycle 2718 시작) | ⏳ **세션 종료 시점 ~2307/2862 진행 중** (stern_brocot 영역) — 다음 세션에서 확인 |
| Tier all 측정 (background, Cycle 2725 시작) | ⏳ **세션 종료 시점 미완료** — Cycle 2725는 historic JSON 데이터로 진행 |

### ISSUE 백로그 변화

- 시작: 25 active (Cycle 2716 triage)
- 종료: **23 active**, 34 closed
- 변경: -3 close (match-jump-table, string-builder-opt, compare-inline), +1 new (or-chain-lowering)
- 갱신: hashmap-perf (P0→P1, 102.7% stamp), alloc-optimization (P1→P2, 104.3% stamp), or-chain (P1→P2, 1.000x stamp)

### P-track 측정 데이터 (Tier 1/3 v098, 2026-05-01/02)

ROADMAP § 5 추가됨 — 6개 P-track 벤치마크 모두 ≤1.085x. M1 ≤1.05x 16/16 PASS 가설 재확인.

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated + Bootstrap | ✅ COMPLETE + 회복 (직전 세션) + **P-track 데이터 강화 (이번 세션)** |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~96% (자율 100%, HUMAN publish 잔여) |
| M4 Adopted | 🔄 ~50% |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ |

---

## 2. 태스크 목록 (잔여 + 신규)

### 다음 세션 최우선 (Structural Improvement — HIGHEST LEVERAGE)

> 🚨 **ISSUE 양식 측정 stamp + stale-after threshold 표준화**
> 
> **Why**: 이번 세션 1년 stale measurement 데이터로 cycles 2722-2724에서 3 연속 false positive. 패턴이 다음 세션에도 재현 가능. 양식 표준화 = 다음 세션 cycle 효율 보호.
> 
> **Plan**: ISSUE template에 measurement_date + stale_after fields 명시. 6개월 이상 측정 = "stale" 경고. 양식 변경 1-2 cycles.

### 자율 가능 작업

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| (1) | **ISSUE 양식 표준화** | 구조 개선 | 🚨 HIGHEST LEVERAGE |
| (2) | 풀 골든 결과 확인 (이번 세션 백그라운드) | 검증 | ⏳ 다음 세션 시작 시점 |
| (3) | Tier all 측정 결과 확인 (이번 세션 백그라운드) | 검증 | ⏳ 다음 세션 시작 시점 |
| (4) | FP 1+2-arg arity guard 통합 (36 사이트) | mechanical | ⏳ 낮은 우선순위 (1 cycle, 단 부담 큼) |
| (5) | HashMap 3% 갭 (1.027x → ≤1.00x) | multi-cycle | ⏳ Arena/hash 교체 phase |
| (6) | Alloc Arena 4% 갭 (1.043x) | multi-cycle | ⏳ runtime 구조 변경 phase |
| (7) | `or` chain proper fix | multi-cycle | ⏳ AST/MIR 변경 phase (3-5 cycles) |

### HUMAN 결정 잔여 (이전 세션과 동일)

| # | 태스크 |
|---|--------|
| M3-3 | npm publish (workflow_dispatch) |
| M3-4 | PyPI publish (workflow_dispatch) |
| M3-5 | bmb-algo README clang vs gcc 라벨 |
| M4-1 | B 공식 측정 (`BMB_BENCH_API_KEY`) |

---

## 3. 핵심 구현 사항 (이번 세션)

### CI Gate (Cycle 2720)

`.github/workflows/bootstrap-benchmark.yml`:
- bootstrap job 내 "Run Golden Sample 50" step 추가 (bootstrap stage1 binary 재활용)
- 풀 2862개 = 43분 → 첫 50개 deterministic = ~40s (PR latency 영향 최소)
- 잠재 약점: alphabetical 첫 50개. nightly 풀 golden 별 cycle 필요 (carry-forward)

`scripts/run-golden-tests.sh`:
- `--limit N` 옵션 추가 (default 0 = full)
- while 루프 break 가드 (TOTAL >= LIMIT)

### ISSUE 백로그 정리 (Cycle 2719 + 2722-2724)

| 그룹 | 카운트 | 변경 |
|------|--------|------|
| Resolved 16개 (Cycle 2716 triage) | 16 | active → closed (Cycle 2719) |
| match-jump-table | 1 | false positive close (Cycle 2722) |
| string-builder-opt | 1 | false positive close (Cycle 2724) |
| compare-inline | 1 | 목표 달성 close (Cycle 2725) |
| or-chain-lowering (신규) | 1 | Cycle 2723 등록, P2 (Cycle 2725 강등) |

### P-track 측정 데이터 활용 (Cycle 2725)

- `target/benchmarks/v098-historic.json` (2026-05-02, 5-run)
- `target/benchmarks/v098-tier3-10runs.json` (2026-05-01, 10-run, noise-gate)

historic.json은 git untracked (target/ ignored). 다음 세션 측정 재실행 시 동일 path 사용.

---

## 4. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항 (직전 세션 유지)

- **BMB_ARENA_MAX_SIZE**: bootstrap.sh default **32G** (Cycle 2713). compiler.bmb 1.04MB+ 처리에 필요.
- **Token packing 5M scale**: 사용자 정수 literal 한도 1.84e12.
- **Builtin name collision**: lint 11 + arity guard 30 사이트 (이중 안전망).
- **FP builtin arity guard 미적용** (낮은 우선순위, carry-forward).

---

## 5. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커, § 5 P-track Tier 1/3 표 추가됨)
- [ ] `claudedocs/cycle-logs/cycle-2718~2727.md` 읽기 (이번 세션 회복 라이브러리)
- [ ] `cargo test --release` → **6210/6210** 확인
- [ ] `./scripts/bootstrap.sh` → Fixed Point S2 == S3 확인
- [ ] **풀 골든 결과 확인** — `/tmp/golden-full-2718.json` fail count (이번 세션 종료 시점 ~2307/2862, 남은 ~555 tests 검증 필요)
- [ ] **Tier all 측정 결과 확인** — `target/benchmarks/tier_all_2026_05_11.json` (이번 세션 백그라운드 미완료)
- [ ] **ISSUE 양식 표준화 작업** (최우선 structural improvement)

---

## 6. HUMAN 결정 사항 (불변, 2026-05-10/11 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ bmb-algo |
| npm publish | ✅ 즉시 진행 |
| PyPI publish | ✅ 즉시 진행 |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 | ✅ 즉시 실행 — `BMB_BENCH_API_KEY` 필요 |
| README "knapsack 6.8x faster" | ⏳ clang -O3 outlier (M4-9 deferred) |

---

## 7. 이번 세션의 메타 통찰

### 1년 stale ISSUE 패턴 (3 cycles 손실)

| Cycle | ISSUE | 가설 (v0.51.22) | 현실 (v0.98) |
|-------|-------|----------------|------------|
| 2722 | match-jump-table | match → switch 매핑 부재 | brainfuck jump table 작동 ✅ |
| 2723 | (`or` chain) | (Cycle 2722에서 도출) | lexer 1.000x — 영향 미미 |
| 2724 | string-builder-opt | fasta StringBuilder 병목 | fasta SB 미사용 (false) |

**Lesson**: 측정 claim하는 ISSUE는 measurement date stamping + stale threshold 필수. 다음 세션 우선순위.

### advisor 활용 패턴

- Cycle 2721: 🟠 RE-PLAN trigger (P-track 우선) — 즉시 적용 효과
- Cycle 2724: pivot to bulk re-measurement — 단일 사이클 ROI 5x 향상
- Cycle 2726: Rule 9 조기 종료 — mechanical 회피 + 시간 절약

advisor의 가장 큰 가치: **이미 transcript에 있는 답을 명시화**. "Don't ask, just do." 패턴.

### Insights-driven 분기 적응

| 인식 | 적응 |
|------|------|
| Cycle 2722: 3 false-positive 패턴 | bulk measurement으로 pivot (Cycle 2725) |
| Cycle 2725: P-track 갭 거의 해소 | FP arity guard 조기 종료 (Cycle 2726) |
| Cycle 2727: ISSUE 양식 결함 | 다음 세션 최우선 structural improvement |

---

## 8. 다음 세션 첫 cycle 권고

### 시퀀스 A — 검증 (병렬 가능)

**Cycle 1 — 백그라운드 결과 확인** (≤2 min):
```bash
# 풀 골든 (cycle 2718 백그라운드)
grep -cE "PASS|FAIL" /tmp/golden-full-2718.json
grep -c "FAIL" /tmp/golden-full-2718.json

# Tier all 측정 (cycle 2725 백그라운드)
ls -la target/benchmarks/tier_all_2026_05_11.json
jq '.[] | select(.tier==1 or .tier==3)' target/benchmarks/tier_all_2026_05_11.json
```

### 시퀀스 B — Structural Improvement [최우선]

**Cycle 2 — ISSUE 양식 표준화** (1-2 cycles):
- 새 ISSUE template: `claudedocs/issues/_template.md`
- Required fields: `measurement_date`, `stale_after`, `measurement_source`
- 기존 23 active ISSUE에 stamp 적용

### 시퀀스 C — Multi-cycle phase (분리)

후속 세션 분리 (single cycle 부적합):
- HashMap 3% 갭 fix (해시 교체)
- Alloc Arena infra 신규
- `or` chain proper fix (AST/MIR 변경)
- FP arity guard 36 사이트 mechanical

### 시퀀스 D — HUMAN 잠금 해소

여전히 잠금 상태:
- M3-3 / M3-4 / M3-5 / M4-1

---

**세션 종료**: 2026-05-11 (Cycles 2718-2727 — P-track 대규모 triage + Bootstrap 안정성 + CI 안전망)
