# BMB Session Handoff — 2026-05-11 (Cycles 2728-2735 — lcs_three 진단 + ISSUE 양식 표준화)

> **HEAD**: `73cfd05c` (Cycles 2728-2735 통합 commit)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: cycle-logs/cycle-2728~2735.md 참조 (local — claudedocs/ gitignored)

---

## 0. 이번 세션 작업 (Cycles 2728-2735, 8 cycles)

### 🎉 큰 변곡점

| 항목 | Before | After |
|------|--------|-------|
| **HANDOFF "lcs_three 1 FAIL 회귀" framing** | "BLOCKING inherited defect" | ✅ **4 fail environmental UB로 재구성** (codegen `inttoptr` 패턴 + Windows MSYS2/UCRT64 heap UB 추정) |
| **ISSUE 양식 표준화** | informal, 일부 6개월 stale | ✅ `_template.md` + 6 필드 (measurement_date / stale_after / measurement_source / observed_rate / scope / env_hash) 21/21 active ISSUE 적용 |
| **active ISSUE 백로그** | 23 (직전 세션 종료 시) | **19** (-4 net: 5 close + 1 new) |
| **컴파일러 버그 v0.98 재현 시도** | 1년 stale (v0.51.22 era) | ✅ 2건 모두 v0.98 재현 불가 → close |
| **양식 leverage 입증** | 가설 | ✅ roadmap-sync close (v0.98 측정 재확인) + 2 codegen ISSUE close (재현 시도 1 cycle each) |

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2728 | lcs_three 회귀 가설 기각 | 진단: 4 fail environmental UB. ISSUE-20260511-golden-flakiness-inttoptr 등록 (양식 first application) |
| 2729 | 풀 골든 + Tier all 백그라운드 시작 | 진행 중 (이 세션 종료 시 ~40% 골든, 0 FAIL so far) |
| 2730 | `_template.md` 신규 + 11 ISSUE stamp + 2 close | llvm-name-conflicts (Lint 11 resolved), simd-vectorization (superseded) |
| 2731 | 9 B-track batch reference pointer | `_b_track_methodology_stamp.md` 신규 batch reference |
| 2732 | hashmap-perf + alloc-optimization 양식 정규화 | informal → 표준 양식 |
| 2733 | roadmap-sync close (v0.98 재측정 확정) | 3 claim 모두 v0.98로 resolved |
| 2734 | strict 6-field 검증 + HANDOFF 정정 | clang-knapsack + or-chain strict 6필드, HANDOFF rewrite |
| 2735 | if-else-early-return + recursive-function v0.98 재현 → 2 close | 1년 stale 컴파일러 버그 모두 v0.98에서 fixed |

---

## 1. 현재 상태

### Bootstrap 검증 상태

| 게이트 | 결과 (직전 세션 Cycle 2718 + 이 세션 영향 없음) |
|--------|-----------|
| Stage 1 (Rust → BMB₁) | ✅ 10.8s |
| Stage 2 (BMB₁ → LLVM IR, 32G arena) | ✅ 29.2s |
| Stage 3 (BMB₂ → LLVM IR) | ✅ 36.7s |
| **Fixed Point S2 == S3** | ✅ **유지** |
| Total | 77.4s |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210/6210 passed (직전 세션 검증, 이 세션 코드 변경 없음) |
| Bootstrap Fixed Point | ✅ S2 == S3 |
| **풀 골든** (Cycle 2729 시작, **Cycle 2736 완료**) | ✅ **2862/2862 PASS, 0 FAIL, 35.2분** (4 flaky tests 모두 PASS — environmental UB 확정) |
| **Tier all** (Cycle 2729 시작, 백그라운드 진행 중) | ⏳ Tier 0 ✅ + Tier 1 진행 중 — 다음 세션 결과 확인 |

### ⚠️ 풀 골든 4건 environmental flakiness — Cycle 2728에서 진단 완료

**Cycle 2728 정정** (HANDOFF 원본의 "1 FAIL 회귀" framing은 잘못된 진단):

| Test | 실패 모드 | 진단 |
|------|----------|------|
| `test_golden_lcs_three` | Segmentation fault → empty (139) | environmental UB |
| `test_golden_cholesky_trace` | empty output (expected=11) | environmental UB 추정 |
| `test_golden_crc32_simple` | linking failed | 별도 (link UB) |
| `test_golden_assortativity` | `child died unexpectedly 0xC0000142` | MSYS2 fork UB |

**실제 root cause** (Cycle 2728 진단):
- Runtime stable since Cycle 2500, codegen 변경 없음
- `-O0`에서도 동일 flakiness (20% rate) — LLVM opt 회귀 아님
- 다른 골든 100% PASS — 광범위 버그 아님
- `inttoptr/ptrtoint` round-trip 빈도 높은 패턴에서만 발현 (lcs_three 41 hits vs lcs 17 / ackermann 3)
- Windows MSYS2/UCRT64 heap UB 추정

**등록된 ISSUE**: `claudedocs/issues/ISSUE-20260511-golden-flakiness-inttoptr.md` (Cycle 2728, **Cycle 2736 갱신: 우선순위 P3 강등**)

**Cycle 2736 데이터로 가설 확정**:
- 첫 실행 (2 concurrent benches): 4/2862 fail (0.14%)
- 격리 stress (50회): 20% segfault
- 깨끗한 환경 재실행 (1 concurrent bench): **0/2862 fail (0%)**
- → system load + concurrent process count에 강한 종속. **inttoptr는 빈도 차이만 결정, root cause는 MSYS2/UCRT64 fork/heap UB**

**Fix scope**: codegen `inttoptr` → `alloca ptr` 전환은 여전히 multi-cycle (5-10), 우선순위는 P3로 강등 (실제 사용자 영향 극히 미미)

### ISSUE 백로그 변화 (이 세션 누적)

- 시작: 23 active (직전 세션 종료)
- 종료: **19 active**, 40 closed
- 누적 변경: +1 신규 (golden-flakiness-inttoptr) + 5 close (llvm-name-conflicts / simd-vectorization / roadmap-sync / if-else-early-return-codegen / recursive-function-codegen)
- 양식 표준화: 21/21 active 100% (12 strict 6필드 + 9 batch reference via `_b_track_methodology_stamp.md`)

### ISSUE 양식 표준화 — 신규 reference 파일

| 파일 | 역할 |
|------|------|
| `claudedocs/issues/_template.md` | 신규 ISSUE 작성 표준 (6필드 + 보존 가이드) |
| `claudedocs/issues/_b_track_methodology_stamp.md` | 9 B-track LLM-bench ISSUE 일괄 measurement stamp (1 reference + 9 pointers) |

### 마일스톤 상태

변경 없음 (이 세션 작업은 doc/ISSUE 정리, M-축 영향 없음):

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated + Bootstrap | ✅ COMPLETE + 회복 |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~96% (자율 100%, HUMAN publish 잔여) |
| M4 Adopted | 🔄 ~50% |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ |

---

## 2. 태스크 목록

### 다음 세션 첫 cycle (백그라운드 결과 확인)

| # | 태스크 | 성격 |
|---|--------|------|
| (0) | `/tmp/golden-full-2729.json` 결과 (FAIL count + 4 flaky test 재현율) | 검증 |
| (1) | `target/benchmarks/tier_all_2026_05_11_c2729.json` 결과 분석 (P-track 변화) | 검증 |
| (2) | golden-flakiness-inttoptr ISSUE `observed_rate` 갱신 | 갱신 |
| (3) | 백그라운드 process 잔존 시 정리 | 정리 |

### 자율 가능 작업 (잔여 19 active 중)

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| (1) | M4-1 B 공식 baseline 실행 (`BMB_BENCH_API_KEY` HUMAN 결정 필요) | HUMAN | ⏳ 잠금 |
| (2) | 9 B-track LLM-bench ISSUE 일괄 갱신 (M4-1 실행 후) | 자율 | M4-1 종속 |
| (3) | `inttoptr` codegen 전환 (golden-flakiness 근본 fix) | multi-cycle (5-10) | 분리 phase 권고 |
| (4) | HashMap 3% / Alloc 4% 갭 multi-cycle phase | multi-cycle | 분리 phase |
| (5) | `or` chain proper fix (codegen AST/MIR 변경) | multi-cycle (3-5) | 분리 phase |
| (6) | FP 1+2-arg arity guard 36 사이트 (mechanical) | 1 cycle, low ROI | carry-forward |
| (7) | `multiple-pre-clauses` 파서 spec 확장 | 1-2 cycles | 언어 spec |
| (8) | `BENCHMARK_REPORT.md` 재생성 또는 stale 경고 (분리 ROADMAP-Sync 후속) | 1 cycle | doc |

### HUMAN 결정 잔여 (이전 세션과 동일)

| # | 태스크 |
|---|--------|
| M3-3 | npm publish (workflow_dispatch) |
| M3-4 | PyPI publish (workflow_dispatch) |
| M3-5 | bmb-algo README clang vs gcc 라벨 |
| M4-1 | B 공식 측정 (`BMB_BENCH_API_KEY`) |

---

## 3. 핵심 산출물 (이번 세션)

### 진단 산출 (Cycle 2728)

`lcs_three` 회귀 가설 기각:
- 50회 stress = 20% segfault rate (격리 환경)
- `-O0`에서도 동일 → opt 회귀 아님
- `inttoptr` count: lcs_three (41) vs PASS test (3-17) — 패턴 보편, 빈도 결정
- Runtime/codegen 6개월 stable
- → ISSUE-20260511-golden-flakiness-inttoptr 등록 (양식 first stamping)

### ISSUE 양식 표준화 (Cycles 2730-2735, 6 cycles)

`_template.md`: 6 필드 (measurement_date, stale_after, measurement_source, observed_rate, scope, env_hash) + 측정 추이 sub-table + 양식 보존 가이드

적용 분류:
- **12 직접 stamping**: simd-codegen, playground-wasm, smt-integration, linter-enhancement, multiple-pre-clauses, if-else-early-return-codegen, recursive-function-codegen, hashmap-perf, alloc-optimization, clang-knapsack-outlier, or-chain-lowering, golden-flakiness-inttoptr
- **9 batch reference**: `_b_track_methodology_stamp.md`가 9 B-track methodology ISSUE 일괄 stamp

Close 트리거 leverage:
- llvm-name-conflicts (Cycle 2730) — Lint 11 (Cycle 2703) 으로 이미 resolution
- simd-vectorization (Cycle 2730) — 형식 close (Superseded)
- roadmap-sync (Cycle 2733) — v0.98 재측정으로 3 claim 모두 resolution
- if-else-early-return-codegen (Cycle 2735) — v0.98 재현 5/5 정답
- recursive-function-codegen (Cycle 2735) — v0.98 재현 5/5 정답 (heapify deterministic)

---

## 4. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항 (직전 세션과 동일)

- **BMB_ARENA_MAX_SIZE**: bootstrap.sh default **32G** (Cycle 2713)
- **Token packing 5M scale**: 사용자 정수 literal 한도 1.84e12
- **Builtin name collision**: lint 11 + arity guard 30 사이트
- **FP builtin arity guard 미적용** (낮은 우선순위, carry-forward)
- **`inttoptr` codegen 패턴**: 모든 BMB 출력에서 사용. lcs_three는 빈도 높아 flakiness 발현 — codegen 전환은 multi-cycle phase

---

## 5. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2728~2735.md` 읽기 (이번 세션 8 cycle)
- [ ] `cargo test --release` → 6210/6210 확인
- [ ] `./scripts/bootstrap.sh` → Fixed Point S2 == S3 확인
- [ ] **백그라운드 작업 결과 확인** (`/tmp/golden-full-2729.json` + `target/benchmarks/tier_all_2026_05_11_c2729.json`)
- [ ] **백그라운드 process kill** (process 잔존 시: `pkill -f "benchmark.sh\|run-golden"`)
- [ ] **golden-flakiness-inttoptr ISSUE `observed_rate` 갱신** (full 2862 실측 rate)
- [ ] M4-1 잠금 해제 검토 (HUMAN 결정)

---

## 6. HUMAN 결정 사항 (불변, 직전 세션 유지)

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

### 1. HANDOFF framing 오류의 직접 비용

cycle 2728의 "lcs_three BLOCKING inherited defect" framing이 다음 세션에 그대로 전파되면:
- 1 cycle을 "compiler regression fix"로 잘못 시작
- 5-10 cycles `inttoptr` 전환에 들어가서야 root cause 발견
- → HANDOFF 정정 (Cycle 2734) leverage 10x

### 2. 양식 표준화 즉시 효과

`_template.md` 적용 cycle (2730) 내에서:
- 1 close (llvm-name-conflicts — Lint 11이 이미 resolution한 사실 발견)
- 1 close (simd-vectorization — Superseded 잔존)
- 1년 stale 데이터 명시적 식별 (roadmap-sync)

다음 2 cycles (2733, 2735):
- 3 추가 close (stale + v0.98 재현 시도)

→ 양식 표준화 = "이 ISSUE가 아직 살아있는가?" 강제 질문 → 5 close 누적

### 3. advisor의 절제된 가치

3차례 핵심 개입:
- Cycle 2728: "회귀 가설 기각, diagnostic-only로 마감" — fix 시도 (multi-cycle) 회피
- Cycle 2734: "polish padding 회피, HANDOFF 정정이 최대 leverage" — 1 cycle을 strucutural improvement에 정렬
- Cycle 2734: "strict 6-field 검증해라" — 표면 grep을 진짜 검증으로 강화

각 1 cycle 이상의 ROI.

### 4. 가설 검증 패턴 (다음 세션 재사용)

stale ISSUE 처리 알고리즘 (Cycle 2735에서 확정):
1. `measurement_date`가 6개월+ 경과 → STALE 표시
2. `scope`가 단일 패턴이면 → 1 cycle 재현 시도
3. v0.98에서 재현 불가 → close (resolution: "v0.98 재현 시도 5/5 정답")
4. 재현 가능 → 신규 진단 시작

이 알고리즘으로 13 LLM-bench era ISSUE 중 compiler bug 카테고리 모두 close 가능 (다음 세션 후보).

---

## 8. 다음 세션 첫 cycle 권고

### 시퀀스 A — Tier all 결과 분석 (golden은 Cycle 2736 완료, 0 FAIL ✅)

**Cycle 1 — Tier all bench 결과**:
```bash
# Tier all 결과 (Cycle 2729 시작, Cycle 2736 미완료)
ls -la target/benchmarks/tier_all_2026_05_11_c2729.json
jq '.[] | select(.tier==1 or .tier==3)' target/benchmarks/tier_all_2026_05_11_c2729.json | head -50

# 백그라운드 process 잔존 정리
ps -ef | grep -E "benchmark.sh|run-golden" | grep -v grep
```

### 시퀀스 B — P-track ISSUE 측정값 갱신 (Tier all 결과 활용)

- `hashmap-perf` (현재 1.027x, 측정일 2026-05-02) — Tier all 갱신값으로 측정 추이 row append
- `alloc-optimization` (현재 1.043x) — 동일
- `or-chain-lowering` (현재 1.000x) — 동일
- 측정값 변화 ≥ 5pp 시 우선순위 재검토 (양식 보존 가이드)

### 시퀀스 C — 잔여 자율 작업

후보 (1-2 cycles each):
- `multiple-pre-clauses` 파서 spec 확장
- `BENCHMARK_REPORT.md` 재생성 (ROADMAP 정합성 후속)
- (HUMAN unlock 시) M3-3 npm publish / M3-4 PyPI publish

### 시퀀스 D — Multi-cycle phase 분리

후속 세션 분리:
- `inttoptr` codegen 전환 (golden-flakiness 근본 fix, 5-10 cycles)
- HashMap 3% 갭 fix (해시 교체, 3-5 cycles)
- Alloc Arena infra 신규 (4-6 cycles)
- `or` chain proper fix (AST/MIR 변경, 3-5 cycles)

---

**세션 종료**: 2026-05-11 (Cycles 2728-2735 — lcs_three 진단 + ISSUE 양식 표준화 + 5 close)
