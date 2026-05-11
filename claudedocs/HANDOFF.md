# BMB Session Handoff — 2026-05-12 (Cycles 2750-2758 — 시퀀스 A.2 + B 확장 + methodology finding)

> **HEAD**: `9f31fa74` (Cycle 2759 — 9 cycles commit: feat(cycles 2750-2758))
> **이전 세션 핸드오프**: Cycle 2749 (`ac47fdfe`) — 시퀀스 E 완결 + bench JSON 완료 반영
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이번 세션 진입점**: Cycle 2750 (HANDOFF § 8 권고)
> **이번 세션 cycle logs**: cycle-2750.md ~ cycle-2758.md (gitignored, disk only)

---

## 결정 채택 (이전 세션 Cycle 2749, 채택 그대로)

| # | 결정 사항 | 채택 |
|---|----------|------|
| M4-1 | B baseline 실행 — claude-sonnet-4-6 × 100문제 × 3 run | ✅ HUMAN setup 대기 |
| M3-5 | bmb-algo bench 재실행 + README 정정 — publish 이전 처리 | ✅ **Cycles 2753-2754 완결** (자율) — HUMAN review 대기 |
| M3-3 + M3-4 | publish dry_run → 실 publish (HUMAN dispatch) | ⏳ M3-5 review 후 즉시 |
| M3-6 | CI workflow + Dockerfile -march=native | ✅ 완결 (Cycle 2746-2747) |
| M3-7 | M4-1 supersedes annotation | M4-1 종속 자동 |

---

## 0. 이번 세션 작업 (Cycles 2750-2758, 9 cycles)

### 🎉 큰 변곡점

| 항목 | Before | After |
|------|--------|-------|
| **bmb-algo headline** | "90× knapsack / 181× nqueens" (v0.2.0 archival, 재현 불가) | ✅ **"Up to ~450× (knapsack(100))"** — v0.98 직접 측정, scaling table demo |
| **M3 progress** | ~97% (M3-5 자율 잔여) | **~98%** (M3-5 자율 완결, HUMAN review 잔여) |
| **Active ISSUE 백로그** | 19 | **17** (-2 net: 2 신규, 3 close) |
| **Tier 3 measurement methodology** | "lexer 1.310x 회귀 후보" (Cycle 2750 false positive) | ✅ 회귀 가설 기각 + **methodology defect ISSUE 등록** (Cycle 2757) |
| **bench_algo.py scaling demo** | 단일 input config (10 items) | knapsack(10) + knapsack(100) + quicksort(15) + quicksort(1000) + scaling sweep section |
| **`docs/LANGUAGE_REFERENCE`** | multiple-pre clauses 제약 미명시 | § 10.4 명시적 disclose + `where { }` 권장 예시 |

### Cycle-by-cycle 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2750 | 시퀀스 A.2 — P-track ISSUE 측정 stamp 갱신 | 3 ISSUE stamps (hashmap-perf 1.040x, alloc-optimization 1.010x, or-chain-lowering 1.310x ⚠️). ROADMAP § 5 c2729 sub-section 추가. lexer/json_serialize/json_parse 회귀 후보 surfaced |
| 2751 | Tier 3 10-run noise-gate 재측정 | 3 회귀 후보 모두 기각 (lexer 1.000x, json_serialize 0.870x, json_parse 1.070x). ROADMAP § 5 Cycle 2751 검증 sub-section |
| 2752 | Bench framework C-side anomaly 점검 (advisor 권고) | clang/gcc fallback = clang (size match). **직접 측정 발견**: Tier 3 workload ~7ms vs framework 30-130ms (80-95% spawn overhead). bmb-algo submodule 아님 (직접 directory). 90x source = v0.2.0 vs Python (NOT clang outlier) |
| 2753 | M3-5 bmb-algo README 정정 v1 | option (a): headline "Up to 43× (prime_count)" + table v0.98 numbers + 90x archival note + quicksort 회귀 disclose. 신규 ISSUE-quicksort-ffi-overhead 등록 |
| 2754 | bench_algo.py large variant 추가 | knapsack(100) 추가 → **447×** 재현! v0.2.0 90× 가 n≥30에서 reproducible 확인. quicksort(1000) 추가 → 2.5×. README headline "Up to ~450×" 회복 + scaling section. v0.2.0 nqueens(8) 181× 모든 N에서 재현 불가 disclose |
| 2755 | ISSUE 백로그 cleanup 1 | alloc-optimization close (1.010x ≈ parity, Arena infra 부적합 ROI). smt-integration close (Deferred 1+ 년 archival). active 17 |
| 2756 | multiple-pre-clauses ISSUE close | Rule 6 정렬 (Rust 새 기능 금지) → documentation 옵션. `docs/LANGUAGE_REFERENCE.md` § 10.4 갱신. acceptance criterion (2) 충족 close. active 16 |
| 2757 | Tier 3 spawn-overhead methodology ISSUE 등록 (advisor 권고) | Cycle 2752 발견 영속화 — ISSUE-20260512-tier3-spawn-overhead-methodology. 3 옵션 (A: workload amplification / B: inproc port / C: warning). active 17 |
| 2758 | HANDOFF rewrite (본 cycle) | — |

### advisor leverage

- **Cycle 2752 호출**: "C-side anomaly 가볍게 처리 위험" 지적 → 5분 investigation → spawn overhead 발견 → Cycle 2754 scaling 재발견 정합. **Cycle 2750 회귀 가설 단정 시 5-10 cycles 헛수고 회피**.
- **Cycle 2756 호출**: "Tier 3 methodology gitignored, 90x narrative 강화, FP arity guard 36 sites drop" 지적 → Cycle 2757-2759 정확 정합

### 진행 회피 (의도된 skip)

| 시퀀스 | 사유 |
|--------|------|
| C (M3-3/4 publish) | HUMAN dispatch 필요. M3-5 review 후. |
| D (M4-1 B baseline) | HUMAN `BMB_BENCH_API_KEY` setup 필요 |
| Multi-cycle phase (inttoptr / HashMap / Arena / or-chain proper fix / Tier 3 inproc) | 잔여 cycle 부족, 별도 세션 |

### 부모 repo 상태

- HEAD: `eaa60a21` (변경 없음)
- 미커밋 변경:
  - `claudedocs/ROADMAP.md` (§ 5 Cycle 2750/2751 sub-section + § 6 Cycle 2755-2757 sub-section + M3 progress 98% + M3-5 row ✅)
  - `ecosystem/bmb-algo/README.md` (headline 450× + scaling table + variant table + v0.2.0 archival)
  - `ecosystem/bmb-algo/CHANGELOG.md` ([Unreleased] section)
  - `ecosystem/bmb-algo/benchmarks/bench_algo.py` (large variant + scaling sweep)
  - `docs/LANGUAGE_REFERENCE.md` (§ 10.4 single-clause constraint)
- 서브모듈: ` M ecosystem/benchmark-bmb` (untracked content, Cycle 2746부터 — **commit 금지**)
- Cycle 2759 = commit all changes

---

## 1. 현재 상태

### Bootstrap 검증 상태

| 게이트 | 결과 (직전 세션 Cycle 2718, 이번 세션 코드 변경 없음) |
|--------|-----------|
| Stage 1 (Rust → BMB₁) | ✅ 10.8s |
| Stage 2 (BMB₁ → LLVM IR, 32G arena) | ✅ 29.2s |
| Stage 3 (BMB₂ → LLVM IR) | ✅ 36.7s |
| **Fixed Point S2 == S3** | ✅ **유지** |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210/6210 passed (코드 변경 없음, 이번 세션 doc-only) |
| 풀 골든 | ✅ Cycle 2736 2862/2862 PASS (이번 세션 코드 변경 없음) |
| Tier all bench | ✅ c2729 + tier3_10run_c2751 (이번 세션 측정) |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated + Bootstrap | ✅ COMPLETE + 회복 |
| M2 AI-Ready Infra | ✅ COMPLETE |
| **M3 External Bindings** | **🔄 ~98%** (M3-5 자율 ✅ + M3-6 ✅, HUMAN review/dispatch 잔여) |
| M4 Adopted | 🔄 ~50% |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ |

### ⚠️ M3-5 review surface area 증가 (이전 plan 대비)

원 시퀀스 B 계획 (Cycle 2745): "bmb-algo bench 재실행 + README 표 정정 + headline 옵션 선택"
실제 (Cycles 2753-2754):
1. ✅ 표 정정 (knapsack(10/100), quicksort(15/1000), prime_count, nqueens, edit_distance, merge_sort, fibonacci — v0.98 numbers)
2. ✅ **headline 옵션 (a) → 추가 (a+b hybrid)**: v0.98 numbers + scaling variant 추가 → 450× 회복
3. ✅ **scaling sweep section 신규** (n=10/30/100/300 knapsack, n=15/100/1000 quicksort)
4. ✅ **bench_algo.py 코드 변경**: WEIGHTS_LG/VALUES_LG/CAPACITY_LG/SORT_DATA_LG 신규 + 2 신규 run() 호출
5. ✅ v0.2.0 nqueens(8) 181× **재현 불가 명시 disclose**
6. ✅ quicksort(15) 0.6× FFI overhead **disclose + 별도 ISSUE 등록**
7. ✅ CHANGELOG [Unreleased] 다중 entry (re-baseline + headline 갱신 + scaling demo + variant added)

→ "narrative tweak" 수준이 아닌 **실측 회복 + 신규 demo 추가 + 코드 변경**. HUMAN review 시간 평소 대비 증가 권고.

### ISSUE 백로그 변화 (이번 세션)

| 시점 | active | closed |
|------|--------|--------|
| 시작 (Cycle 2735 종료) | 19 | 40 |
| 종료 (Cycle 2757) | **17** | **43** |

**신규 2건**: `ISSUE-20260512-bmb-algo-quicksort-ffi-overhead`, `ISSUE-20260512-tier3-spawn-overhead-methodology`
**Close 3건**: `alloc-optimization` (1.010x parity), `smt-integration` (Deferred archival), `multiple-pre-clauses` (Rule 6 → documentation)

---

## 2. 태스크 목록

### 다음 세션 첫 cycle — 진입점 분기

#### 분기 A — HUMAN review (M3-5 narrative) 우선

| # | 태스크 | 책임 |
|---|--------|------|
| A.1 | `ecosystem/bmb-algo/README.md` 검토: headline "Up to ~450×" + scaling table + v0.2.0 archival note 적정성 | HUMAN |
| A.2 | `ecosystem/bmb-algo/CHANGELOG.md [Unreleased]` 검토 | HUMAN |
| A.3 | quicksort 회귀 disclose 표현 검토 (별도 ISSUE 처리 적절성) | HUMAN |
| A.4 | review 후 추가 정정 지시 또는 publish 허가 | HUMAN |

#### 분기 B — review 통과 후 publish dispatch (시퀀스 C)

| # | 태스크 | 명령 |
|---|--------|------|
| B.1 | M3-3: npm publish dry_run | `gh workflow run npm-publish.yml -f dry_run=true` |
| B.2 | M3-3: npm 실 publish | `gh workflow run npm-publish.yml -f dry_run=false` |
| B.3 | M3-4: PyPI publish dry_run | `gh workflow run pypi-publish.yml -f publish=false -f repository=pypi` |
| B.4 | M3-4: PyPI 실 publish | `gh workflow run pypi-publish.yml -f publish=true` |
| B.5 | publish 결과 24h 모니터링 (metadata + README rendering + links) | 자율/HUMAN 혼합 |

#### 분기 C — M4-1 B baseline (HUMAN setup + 자율 실행)

| # | 태스크 | 책임 |
|---|--------|------|
| C.1 | `.env.local` `BMB_BENCH_API_KEY` 설정 | HUMAN |
| C.2 | `bmb-ai-bench doctor` 검증 + `--pilot --dry-run` | 자율 |
| C.3 | `bmb-ai-bench run --all --runs 3 --model claude-sonnet-4-6` (예상 8-12h) | 실행 |
| C.4 | 결과 commit + ROADMAP § 5 B축 baseline 선언 + M3-7 자동 annotation | 자율 |
| C.5 | 9 B-track ISSUE 일괄 갱신 (`_b_track_methodology_stamp.md`) | 자율 |

#### 자율 분리 phase 후보 (별도 세션)

| # | 태스크 | 성격 |
|---|--------|------|
| (1) | `inttoptr` codegen 전환 (golden-flakiness 근본 fix) | multi-cycle 5-10, P3 |
| (2) | HashMap 3% 갭 fix (해시 교체) | multi-cycle 3-5 |
| (3) | `or` chain proper fix (codegen AST/MIR 변경) | multi-cycle 3-5 |
| (4) | **Tier 3 inproc 변환 (또는 workload amplification)** | multi-cycle (A: 1-2, B: 5-10) — ISSUE-20260512-tier3-spawn-overhead-methodology |
| (5) | bmb-algo quicksort FFI 최적화 (Option B) | multi-cycle 2-3 — ISSUE-20260512-bmb-algo-quicksort-ffi-overhead |

---

## 3. 핵심 산출물 (이번 세션)

### Documentation 산출

- `docs/LANGUAGE_REFERENCE.md` § 10.4 — multiple-pre clauses 제약 + `where { }` 권장
- `ecosystem/bmb-algo/README.md` — headline 450× + scaling section + v0.2.0 archival
- `ecosystem/bmb-algo/CHANGELOG.md` `[Unreleased]` — 다중 entry
- `claudedocs/ROADMAP.md` § 5 Cycle 2750/2751 sub-sections + § 6 Cycle 2755-2757

### Code 산출

- `ecosystem/bmb-algo/benchmarks/bench_algo.py` — WEIGHTS_LG/VALUES_LG/CAPACITY_LG/SORT_DATA_LG + 2 신규 run() 호출 (knapsack(100), quicksort(1000))

### ISSUE 산출

- 신규: `ISSUE-20260512-bmb-algo-quicksort-ffi-overhead.md` (P3, ecosystem/bindings)
- 신규: `ISSUE-20260512-tier3-spawn-overhead-methodology.md` (P2, ci/benchmarks)
- Closed: alloc-optimization, smt-integration, multiple-pre-clauses

### Measurement 산출

- `target/benchmarks/tier3_10run_2026_05_12_c2751.json` (10-run noise-gate Tier 3, 36s wall, 7 benches)
- bmb-algo bench v0.98 측정 (knapsack/nqueens/quicksort scaling sweep)

---

## 4. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항

- **Tier 3 bench 측정 시 주의** (NEW from Cycle 2752): wall-time 30-130ms 중 80-95%가 OS spawn overhead. ratio_c는 fair, absolute ms 변화는 노이즈. `ISSUE-20260512-tier3-spawn-overhead-methodology` 참조.
- **bmb-algo는 submodule 아님** (NEW from Cycle 2752): `.gitmodules`에 없음. `ecosystem/bmb-algo/`는 부모 repo 직접 directory. PR 부담 없음.
- 직전 세션 운용 주의사항 그대로:
  - BMB_ARENA_MAX_SIZE default 32G
  - Token packing 5M scale
  - Builtin name collision (Lint 11 + arity guard 30 사이트)
  - FP builtin arity guard 미적용 (낮은 우선순위, carry-forward — advisor "drop filler" 권고로 본 세션 진행 안 함)

---

## 5. 다음 세션 시작 체크리스트

### 기본 검증
- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커, Cycle 2750-2757 누적 반영)
- [ ] `claudedocs/cycle-logs/cycle-2758.md` 읽기 (이 cycle, gitignored disk only)
- [ ] `claudedocs/cycle-logs/cycle-2750~2757.md` 필요 시 (gitignored)
- [ ] `cargo test --release` → 6210/6210 (필요 시, 이번 세션 코드 변경은 doc + bench_algo.py만 — bmb compiler 영향 없음)

### M3-5 review 진입 (분기 A)
- [ ] `ecosystem/bmb-algo/README.md` headline + scaling table review
- [ ] `ecosystem/bmb-algo/CHANGELOG.md [Unreleased]` review
- [ ] 통과 시 → 분기 B (publish dispatch)
- [ ] 추가 정정 필요 시 → 자율 정정 cycle

### Publish 진입 (분기 B, M3-5 review 통과 후)
- [ ] `gh workflow run npm-publish.yml -f dry_run=true`
- [ ] dry_run artifact 검증
- [ ] `dry_run=false` 재실행
- [ ] PyPI 동일 절차

### M4-1 B baseline (분기 C, HUMAN setup 후)
- [ ] `BMB_BENCH_API_KEY` 설정 확인
- [ ] `bmb-ai-bench doctor` PASS
- [ ] `--all --runs 3` 실행 (8-12h, ScheduleWakeup 활용 가능)

---

## 6. HUMAN 결정 사항 (Cycles 2737-2758 누적)

| 항목 | 결정 |
|------|------|
| M3 showcase | ✅ bmb-algo |
| npm publish | ✅ M3-5 review **후** 즉시 진행 |
| PyPI publish | ✅ M3-5 review **후** 즉시 진행 |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 모델 | ✅ **claude-sonnet-4-6** |
| B 공식 측정 실행 | ✅ 즉시 — API key 설정 후 자동 |
| **M3-5 README/CHANGELOG (Cycles 2753-2754 누적)** | ⏳ **HUMAN review 대기** — surface area 증가 (headline 450× 회복 + scaling table + variant 추가 + quicksort disclose) |
| M3-6 CI flag | ✅ **Cycles 2746-2747 완결** |
| M3-7 baseline annotation | ✅ M4-1 종속 자동 |
| **신규 (Cycle 2755)**: alloc-optimization close (1.010x parity) | ✅ 자율 |
| **신규 (Cycle 2755)**: smt-integration archival (Deferred 1+ 년) | ✅ 자율 |
| **신규 (Cycle 2756)**: multiple-pre-clauses documentation 옵션 | ✅ 자율 (Rule 6 정렬) |

---

## 7. 이번 세션의 메타 통찰

### 1. advisor 절제 leverage (3차)

- **Cycle 2752 호출** (Cycle 2751 advisor): "C-side anomaly 가볍게 처리 위험" → 5분 investigation → spawn overhead 발견 → Cycle 2750 회귀 가설 단정 회피 (5-10 cycles 분석 부담 회피)
- **Cycle 2756 호출**: "Tier 3 methodology gitignored, 90x narrative 강화, FP arity guard drop" → Cycle 2757-2759 정확 정합 (재구성된 2-cycle wrap-up + 결과 ISSUE 영속화)
- 각각 1.5-10 cycles ROI

### 2. v0.2.0 archival → 회복의 narrative 강화

Cycle 2753 option (a) "v0.98 numbers, 90× retire"가 conservative honesty. Cycle 2754 scaling demo가 발견적 회복 — knapsack(100) 447× → **v0.2.0 90× 가 합당했으며, 실제 더 큰 N에서 amplify 됨**. 

이 패턴은 BMB 철학 정렬:
- "측정 없는 성능 주장 금지" — v0.98 numbers 명시
- "성능 저하는 버그다" — quicksort(15) 0.6× disclose + ISSUE 등록
- **"BMB는 C/Rust 추월 목표"** — knapsack(100) 447×는 BMB의 존재가치 입증 demo

### 3. 측정 framework methodology 발견의 영속화

Cycle 2752 발견 (Tier 3 workload < spawn overhead)이 gitignored cycle log에만 → advisor 권고로 ISSUE 영속화 (Cycle 2757). gitignored cycle logs는 본 세션에서만 유효, **tracked artifact만 다음 세션에 전파**. 핵심 발견은 ISSUE/ROADMAP에 등록 필수.

### 4. Rule 6 적용의 좁은 범위

multiple-pre-clauses는 작은 Rust 변경으로 fix 가능했으나 Rule 6 적용 → documentation 옵션. BMB가 Rust 컴파일러 졸업으로 점진 전환 중인 상태에서 새 예외 만들지 않음. 작은 변경이라도 컨벤션 우선.

---

## 8. 다음 세션 첫 cycle 권고

### Cycle 2760 — 분기 A (HUMAN review)

```bash
# Review m3-5 narrative
cat ecosystem/bmb-algo/README.md       # headline + scaling + archival
cat ecosystem/bmb-algo/CHANGELOG.md    # [Unreleased]
cat claudedocs/issues/ISSUE-20260512-bmb-algo-quicksort-ffi-overhead.md  # 별도 disclose
```

review 결과 분기:
- ✅ 통과 → Cycle 2761 분기 B (publish dispatch)
- 정정 필요 → 자율 정정 → review 재요청

### Cycle 2761+ — 분기 B/C

분기 B (publish dispatch) → 분기 C (M4-1 baseline) 또는 multi-cycle phase 시작 (자율 분리 phase 후보 (1)~(5)).

### Cycle 2762+ — 자율 분리 phase 후보 진입

advisor pacing 권고: bounded single-cycle vs multi-cycle phase 결정. 잔여 cycle 충분 시 (4) Tier 3 inproc 변환 또는 (1) inttoptr codegen 추천 (둘 다 5-10 cycles, P-track 신뢰도 향상 직결).

---

**세션 종료**: 2026-05-12 (Cycles 2750-2759 — 시퀀스 A.2 + B 확장 완결 + methodology finding + ISSUE -2 net). HEAD `9f31fa74` (Cycle 2759 commit).
