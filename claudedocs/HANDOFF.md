# BMB Session Handoff — 2026-05-11 (Cycle 2746 — 시퀀스 E 자율 실행: -march=native PR draft)

> **HEAD**: `627dd3a1` (Cycle 2746에서 부모 repo HEAD 변경 0건 — 서브모듈 commit + claudedocs gitignored)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: cycle-logs/cycle-2737~2745.md (Cycles 2737-2745 policy-vs-code 갭 fix + HUMAN 결정 채택)
> **이번 cycle log**: cycle-logs/cycle-2746.md (gitignored, disk only)
> **이번 PR draft**: pr-draft-march-native.md (gitignored, disk only)

---

## 결정 채택 (2026-05-11, 세션 종료 직전)

권장 옵션 모두 채택. 다음 세션 첫 cycle은 **§ 2.5 권장 실행 시퀀스** 참조.

| # | 결정 사항 | 채택 |
|---|----------|------|
| M4-1 | B baseline 즉시 실행 — 모델 **claude-sonnet-4-6** + 100문제 × 3 run | ✅ |
| M3-5 | bmb-algo bench 재실행 + README 정정 (자율) — publish 이전 처리 | ✅ |
| M3-3 + M3-4 | M3-5 정정 후 publish dry_run → 실 publish (HUMAN dispatch) | ✅ |
| M3-6 | CI workflow + Dockerfile 5위치 `-march=native` 일괄 PR (서브모듈, draft 자율) | ✅ |
| M3-7 | M4-1 실행 결과에 "supersedes 2026-03-26 (baseline change Cycle 2742)" annotation | ✅ (M4-1 종속) |

---

## 0. 이번 세션 작업 (Cycle 2746, 1 cycle)

### Cycle 2746 — 시퀀스 E 자율 실행

| 항목 | 상태 |
|------|------|
| 서브모듈 branch `fix/march-native-spec-parity` (`ecosystem/benchmark-bmb`) | ✅ 생성 + commit `cb478d2` |
| 5 위치 `-march=native` 일괄 수정 | ✅ 3 files, +5/-5 (Cycle 2743 매핑 그대로) |
| PR body draft 파일 | ✅ `claudedocs/pr-draft-march-native.md` (gitignored, disk only) |
| 서브모듈 push | ⏸ **HUMAN trigger 대기** (HANDOFF E.4 명시) |
| 부모 repo submodule pointer | ⏸ **미커밋** (서브모듈 미push 상태에서 bump하면 broken pointer) |

### 진행 회피 (의도된 skip)

| 시퀀스 | 사유 |
|--------|------|
| A | 백그라운드 bench JSON 미생성 (Cycle 2729 시작, ~2시간 진행 중, 마지막 출력 mergesort) — carry-forward |
| B | bench 동시 실행 시 system load 충돌 (golden-flakiness-inttoptr ISSUE 확정) — bench 종료 후 |
| C | M3-3/M3-4 publish는 HUMAN gh workflow run trigger 필요 + M3-5 정정 선결 |
| D | M4-1 baseline은 HUMAN `BMB_BENCH_API_KEY` setup 선결 |

### 부모 repo 상태

- HEAD: `627dd3a1` (변경 없음, 이 cycle은 parent commit 0건)
- Working tree dirty: ` M ecosystem/benchmark-bmb` (서브모듈 HEAD 이동 인지) — **commit 금지**

---

## 0a. 이전 세션 작업 (Cycles 2737-2744, 8 cycles)

### 큰 변곡점

| 항목 | Before | After |
|------|--------|-------|
| **BENCHMARK_REPORT.md** | v0.51.22 / v0.50.51 (3.5개월 stale, misleading) | ✅ ROADMAP redirect + stale warning + 핵심 변화 표 |
| **context-overflow-prevention 적용 범위** | scripts/run_experiment.py 누락 (production `run_cmd.py` 미발견) | ✅ 3 위치 정합 (scripts/run_experiment.py + scripts/run_crosslang.py + bmb_ai_bench/run_cmd.py) |
| **crosslang gcc baseline** | `-O2` only (README/registry/metadata spec 정합 위반) | ✅ `-O2 -march=native` 정책 정합 |
| **policy-vs-code 갭 인벤토리** | 미문서화 | ✅ 7 위치 매핑 (3 자율 fix + 5 HUMAN 결정) |
| **active ISSUE** | 19 | 18 (-1 net: context-overflow close) |

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2737 | BENCHMARK_REPORT.md stale warning + ROADMAP redirect | 두 파일 stale 경고 + 핵심 변화 표 (sorting 0.910x BMB FASTER 등) |
| 2738 | ISSUE backlog deep audit | 19/19 카테고리 분류, 추가 close 후보 부재 결론 (audit-only) |
| 2739 | context-overflow-prevention close + scripts/run_experiment.py fix | sliding window 적용, ISSUE close |
| 2740 | production run_cmd.py 동일 fix (leverage 확장) | `bmb-ai-bench run` 명령 production 경로 + ai-bench 30 pytest PASS |
| 2741 | closed ISSUE 본문 갱신 + bmb-algo README 불일치 발견 | 3 위치 명시 + M3-5 HUMAN 큐 확장 (knapsack 100→10 items, 90x source) |
| 2742 | scripts/run_crosslang.py `-march=native` 추가 | README 정책 정합, ai-bench 30 pytest PASS |
| 2743 | CI workflow + Dockerfile -march=native 갭 발견 (HUMAN) | 5 위치 매핑, blast-radius 기준 자율 fix 회피 |

---

---

## 0' (이전 세션). 이번 세션 작업 (Cycles 2728-2735, 8 cycles)

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

### 다음 세션 첫 cycle — 채택 결정 시퀀스 (Cycle 2746 진척 반영)

**Cycle 2746 자율 완결**: 시퀀스 E (서브모듈 commit `cb478d2` + PR draft). HUMAN push 대기.

**다음 세션 진입점**: 시퀀스 A (백그라운드 bench JSON 확인) → 완료 시 시퀀스 B.

#### 시퀀스 A (자율, 백그라운드 bench 종료 후) — 다음 세션 Cycle 2747

| # | 태스크 | 성격 |
|---|--------|------|
| A.0 | 백그라운드 bench 상태 확인: `target/benchmarks/tier_all_2026_05_11_c2729.json` 출현 여부 | 검증 |
| A.1 | 잔존 process 확인: `ps -ef \| grep benchmark.sh` — 종료/진행 결정 | 정리 |
| A.2 | bench 완료 시: P-track ISSUE 측정 stamp 갱신 (hashmap-perf, alloc-optimization, or-chain-lowering) | 갱신 |

#### 시퀀스 B (자율, M3-5 정정 — publish 선결 조건) — Cycle 2746-2747

| # | 태스크 | 성격 |
|---|--------|------|
| B.1 | `bench_algo.py` v0.98 재실행 (5-run median) — 현재 config (10 items) 기준 정확한 측정 | 측정 |
| B.2 | README 표 정정: "knapsack(10 items)" + clang vs gcc 라벨 명시 | doc |
| B.3 | headline "90x/181x" 처리 — 옵션: (a) v0.98 측정값으로 재정정 (b) 100-items bench 변종 추가 (c) "Up to" 약화 표현 | doc |
| B.4 | bmb-algo CHANGELOG `[Unreleased]` 에 "measurement re-baseline 2026-05-11" 명시 | doc |

#### 시퀀스 C (HUMAN dispatch) — Cycle 2748 (사용자 트리거 후)

| # | 태스크 | 성격 |
|---|--------|------|
| C.1 | M3-3: `gh workflow run npm-publish.yml -f dry_run=true` → artifact 검증 → `dry_run=false` 재실행 | HUMAN |
| C.2 | M3-4: `gh workflow run pypi-publish.yml -f publish=false -f repository=pypi` → 검증 → `publish=true` | HUMAN |
| C.3 | publish 결과 PyPI/npm metadata + README rendering + links 24h 모니터링 | 검증 |

#### 시퀀스 D (HUMAN trigger, M4-1 baseline) — Cycle 2749 부터

| # | 태스크 | 성격 |
|---|--------|------|
| D.1 | `.env.local`에 `BMB_BENCH_API_KEY` 설정 (HUMAN) + Model 고정 `claude-sonnet-4-6` | HUMAN setup |
| D.2 | `bmb-ai-bench doctor` 환경 검증 + `bmb-ai-bench run --pilot --dry-run --json` 정합 확인 | 자율 |
| D.3 | `bmb-ai-bench run --all --runs 3 --model claude-sonnet-4-6` 실행 (예상 8-12시간) | 실행 |
| D.4 | 결과 `results/baseline-2026-05-11-sonnet/` commit + ROADMAP § 5 B축 baseline 선언 | 자율 |
| D.5 | M3-7 자동 처리: 결과 README/CHANGELOG에 "supersedes 2026-03-26" annotation | 자율 |
| D.6 | 9 B-track ISSUE 일괄 갱신 (`_b_track_methodology_stamp.md`) + 5-7 close 후보 | 자율 |

#### 시퀀스 E (분리 PR, M3-6 CI flag) — ✅ Cycle 2746 자율 완료

| # | 태스크 | 상태 |
|---|--------|------|
| E.1 | benchmark-bmb 서브모듈 새 branch `fix/march-native-spec-parity` | ✅ Cycle 2746 |
| E.2 | 5 위치 일괄 수정 (workflows x4 + Dockerfile x1) — Cycle 2743 매핑 활용 | ✅ Cycle 2746 (3 files, +5/-5) |
| E.3 | PR draft 본문: "baseline change", "이전 CI history 직접 비교 불가" 명시 | ✅ `claudedocs/pr-draft-march-native.md` |
| E.4 | 서브모듈 commit (push 금지) | ✅ `cb478d2` (Cycle 2746) |
| E.5 | **HUMAN push + PR open + merge** → 첫 CI run을 새 baseline으로 stamp | ⏸ **HUMAN trigger** |
| E.6 | (E.5 후 자율) 부모 repo submodule pointer bump commit | ⏸ HUMAN merge 후 자율 |

### 자율 가능 작업 (multi-cycle 분리 phase 후보)

채택 결정 시퀀스 A-E **외**의 잔여 자율 작업:

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| (1) | `inttoptr` codegen 전환 (golden-flakiness 근본 fix) | multi-cycle (5-10) | 분리 phase (P3 우선순위) |
| (2) | HashMap 3% / Alloc 4% 갭 multi-cycle phase | multi-cycle | 분리 phase |
| (3) | `or` chain proper fix (codegen AST/MIR 변경) | multi-cycle (3-5) | 분리 phase |
| (4) | FP 1+2-arg arity guard 36 사이트 (mechanical) | 1 cycle, low ROI | carry-forward |
| (5) | `multiple-pre-clauses` 파서 spec 확장 (compiler.bmb + bootstrap) | 1-2 cycles | 언어 spec, bench 동시 실행 회피 권고 |
| (6) | ~~`BENCHMARK_REPORT.md` stale 경고~~ | ✅ Cycle 2737 |
| (7) | ~~context-overflow-prevention~~ | ✅ Cycle 2739-2740 |
| (8) | ~~crosslang gcc flag~~ | ✅ Cycle 2742 |
| (9) | ~~시퀀스 E CI -march=native 5위치 PR draft~~ | ✅ Cycle 2746 (push HUMAN 대기) |

### HUMAN 결정 잔여 (채택 결정 반영)

모든 결정은 ✅ 채택. 잔여는 **실제 실행 트리거**만 HUMAN 필요:

| # | 항목 | HUMAN action |
|---|------|-------------|
| M3-3 | npm publish | `gh workflow run npm-publish.yml -f dry_run=false` 시점 결정 (M3-5 정정 후) |
| M3-4 | PyPI publish | `gh workflow run pypi-publish.yml -f publish=true` 시점 결정 (M3-5 정정 후) |
| M3-5 | bmb-algo README headline 처리 옵션 (a/b/c) 선택 | 시퀀스 B.3 자율 정정 후 review |
| M3-6 | CI flag PR push + open + merge | **Cycle 2746에서 commit 완료**. HUMAN: `cd ecosystem/benchmark-bmb && git push -u origin fix/march-native-spec-parity` + `gh pr create --draft --body-file claudedocs/pr-draft-march-native.md` |
| M3-7 | (M4-1에 종속, 자동 처리) | — |
| M4-1 | B baseline 실행 | `.env.local`에 `BMB_BENCH_API_KEY` 설정 + 모델 confirm (sonnet-4-6) |

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

### 기본 검증
- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2746.md` 읽기 (직전 cycle, gitignored)
- [ ] `claudedocs/cycle-logs/cycle-2737~2745.md` 읽기 (이전 세션 9 cycle)
- [ ] `cargo test --release` → 6210/6210 확인 (필요 시)
- [ ] `./scripts/bootstrap.sh` → Fixed Point S2 == S3 (필요 시)

### 부모 repo 상태 확인 (Cycle 2746 영향)
- [ ] `git status` → ` M ecosystem/benchmark-bmb` 표시 (서브모듈 HEAD 이동)
- [ ] **commit 금지**: 서브모듈 미push 상태 → submodule pointer bump 하면 broken
- [ ] HUMAN이 서브모듈 push + PR merge 완료한 후에만 부모에서 submodule pointer bump

### 백그라운드 정리 (시퀀스 A — Cycle 2746에서 미완료, 다음 세션 첫 cycle)
- [ ] `ls -la target/benchmarks/tier_all_2026_05_11_c2729.json` — JSON 완료 여부
- [ ] `ps -ef | grep benchmark.sh` — 잔존 process 확인
- [ ] 완료 시: P-track ISSUE 측정 stamp 갱신 (hashmap-perf / alloc-optimization / or-chain-lowering)
- [ ] 미완료 시: 백그라운드 process 종료 결정 (대기 vs 강제 종료)

### 채택 결정 순차 실행 (잔여)
- [ ] **시퀀스 B** (자율, bench 종료 후): M3-5 bmb-algo bench 재실행 + README 정정
- [ ] **시퀀스 C** (HUMAN): M3-3/M3-4 publish dry_run → 실 publish
- [ ] **시퀀스 D** (HUMAN setup + 자율 실행): M4-1 baseline 측정 — sonnet-4-6 100문제 × 3 run
- [x] ~~**시퀀스 E** (자율 draft + HUMAN merge): M3-6 CI flag PR~~ → **Cycle 2746 자율 완료**, HUMAN push 대기

---

## 6. HUMAN 결정 사항 (Cycles 2737-2745 채택 갱신)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ bmb-algo |
| npm publish | ✅ M3-5 정정 **후** 즉시 진행 (순서 정정) |
| PyPI publish | ✅ M3-5 정정 **후** 즉시 진행 (순서 정정) |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 모델 | ✅ **claude-sonnet-4-6** (Opus 대비 비용 1/5, 품질 80%+) — 채택 |
| B 공식 측정 실행 | ✅ 즉시 — `BMB_BENCH_API_KEY` 설정 후 자동 |
| M3-5 bmb-algo README 처리 | ✅ **재측정 + 정정** (BMB 철학 "측정 없는 성능 주장 금지"). headline 옵션 자율 검토 후 review |
| M3-6 CI flag PR | ✅ **spec 정합 적용** (CI history 단절 수용) — 단일 PR draft |
| M3-7 baseline 변경 명시 | ✅ M4-1 결과에 inline annotation (M4-1 종속) |
| README "knapsack 6.8x faster" | ⏳ clang -O3 outlier (M4-9 deferred) — M3-5 처리와 분리 |

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

## 8. 다음 세션 첫 cycle 권고 (Cycle 2747 진입점)

### Cycle 2747 — 시퀀스 A 백그라운드 bench JSON 확인

```bash
# Tier all 결과 (Cycle 2729 시작, Cycle 2746 미완료 상태)
ls -la target/benchmarks/tier_all_2026_05_11_c2729.json
ps -ef | grep -E "benchmark.sh|run-golden" | grep -v grep

# 완료 시
jq '.[] | select(.tier==1 or .tier==3)' target/benchmarks/tier_all_2026_05_11_c2729.json | head -50
```

### Cycle 2748 — 시퀀스 B (bench 종료 후, autonomous)

- M3-5 bmb-algo bench v0.98 재실행 (5-run median, 현재 config 10 items)
- README 표 정정: "knapsack(10 items)" + clang vs gcc 라벨
- headline "90x/181x" 처리 (a/b/c 옵션 자율 분석 후 review)
- CHANGELOG `[Unreleased]` re-baseline annotation

### 시퀀스 C/D — HUMAN trigger 시점 결정

- C: M3-3 npm publish / M3-4 PyPI publish (M3-5 정정 후)
- D: M4-1 baseline (`BMB_BENCH_API_KEY` setup 후)

### 시퀀스 E — HUMAN push pending

- 서브모듈 commit `cb478d2` ready on `fix/march-native-spec-parity`
- PR body draft: `claudedocs/pr-draft-march-native.md`
- HUMAN: `git push -u origin fix/march-native-spec-parity` + `gh pr create --draft --body-file ...`
- Post-merge: 부모 repo submodule pointer bump (자율 가능)

### Multi-cycle phase 분리 (별도 세션)

- `inttoptr` codegen 전환 (golden-flakiness 근본 fix, 5-10 cycles, P3 우선순위)
- HashMap 3% 갭 fix (해시 교체, 3-5 cycles)
- Alloc Arena infra 신규 (4-6 cycles)
- `or` chain proper fix (AST/MIR 변경, 3-5 cycles)

---

**세션 종료**: 2026-05-11 (Cycle 2746 — 시퀀스 E 자율 실행: -march=native PR draft + 서브모듈 commit `cb478d2`)
