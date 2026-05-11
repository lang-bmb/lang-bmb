# BMB Session Handoff — 2026-05-12 (Cycles 2760-2764 — M3-5 honest re-baseline)

> **HEAD**: TBD (Cycle 2764 commit 후 갱신)
> **이전 세션 핸드오프**: Cycle 2759 (`9f31fa74`) — 시퀀스 A.2 + B 확장 + methodology finding
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이번 세션 진입점**: Cycle 2760 (HUMAN review of M3-5 narrative → re-baseline 발견 → 정정)
> **이번 세션 cycle logs**: gitignored (disk only)

---

## 0. 이번 세션 작업 (Cycles 2760-2764, 5 cycles, doc-only)

### 🚨 중요 발견: Cycle 2754 measurements가 single-sample outlier

| 항목 | Cycle 2754 (이전 세션) | Cycle 2763 median-of-5 (지금) |
|------|------------------------|----------------------------|
| knapsack(100) | ~450× | **~243×** (235-257) |
| quicksort(15) | ~0.9× SLOW ⚠️ | **~1.73× FAST** |
| quicksort(1000) | ~2.5× | **~4.86×** |
| nqueens(10) | ~5.8× | **~3.96×** |
| knapsack(n=300) | ~600× | **~306×** |

**근본 원인**: Cycle 2754는 **n=2 sampling** ("442× vs 447× → 5% variance 정상"). 5-15% inter-run variance를 underestimate. Cycle 2752 ISSUE-quicksort-ffi-overhead는 **n=1 sampling**.

advisor 호출 시점: Cycle 2762 첫 측정에서 unexpected discrepancy 발견 → 5회 측정 → stable-but-different world 확인.

### Cycle-by-cycle 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2760 | M3-5 narrative review | 5개 review 항목: README/CHANGELOG/quicksort-ffi ISSUE/ROADMAP/LANGUAGE_REFERENCE. Finding 1 (LANGUAGE_REFERENCE § 10.4 자기모순 `len: i64{it < len}`) + Finding 2 (scaling table 재현 불가) 검출 |
| 2761 | LANGUAGE_REFERENCE § 10.4 수정 | 자기모순 refinement type 제거. `where { bounded: idx >= 0 and idx < len }` 단일 패턴 통일 |
| 2762 | bench_algo.py median-of-N 인프라 | `--runs=N` argparse + min-max spread reporting + scaling sweep wrap. 첫 측정에서 Cycle 2754 ≠ 지금 발견 |
| 2763 | Honest re-baseline 측정 + README 갱신 | median-of-5 fresh measurement. README headline "Up to ~245× (knapsack(100), median-of-5)". scaling table 갱신 (29×/119×/246×/306×). 3중 archival (v0.2.0 / Cycle 2754 / 현재) 정직 disclose |
| 2764 | quicksort-ffi ISSUE close + commit | ISSUE-bmb-algo-quicksort-ffi-overhead → closed/ 이동. Closure summary (median-of-5 1.64-1.74× FAST). ROADMAP § 5/6 + § M3 row 갱신. commit |

### advisor leverage

- **Cycle 2762 호출 (in-line stop, scope expansion)**: Cycle 2762 첫 scaling sweep 측정에서 README와 차이 큰 numbers 발견 시 advisor 호출 시도 → advisor가 자체 차단 ("Don't push to advisor yet — answer is in the data"). 5회 측정 권고. variance world 결정 후 honest re-baseline path 선택. **publish narrative 신뢰도 손실 회피, 정직성 우선 강화**.

---

## 1. 현재 상태

### Bootstrap 검증 상태

| 게이트 | 결과 (Cycle 2718, 이번 세션 코드 변경 없음 — doc + bench harness only) |
|--------|-----------|
| Stage 1 (Rust → BMB₁) | ✅ 10.8s |
| Stage 2 (BMB₁ → LLVM IR, 32G arena) | ✅ 29.2s |
| Stage 3 (BMB₂ → LLVM IR) | ✅ 36.7s |
| **Fixed Point S2 == S3** | ✅ **유지** |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210/6210 (이번 세션 BMB compiler 변경 없음) |
| 풀 골든 | ✅ Cycle 2736 2862/2862 PASS |
| bench_algo.py median-of-5 | ✅ 9/9 BMB FASTER (median) |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated + Bootstrap | ✅ COMPLETE + 회복 |
| M2 AI-Ready Infra | ✅ COMPLETE |
| **M3 External Bindings** | **🔄 ~99%** (M3-5 자율 honest re-baseline 완결, HUMAN publish dispatch 잔여) |
| M4 Adopted | 🔄 ~50% |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ |

### ISSUE 백로그 변화 (Cycles 2760-2764)

| 시점 | active | closed |
|------|--------|--------|
| 시작 (Cycle 2757 종료) | 17 | 43 |
| 종료 (Cycle 2764) | **16** | **44** |

**Close 1건**: `quicksort-ffi-overhead` (재현 불가, median-of-5에서 일관 FAST)

---

## 2. 태스크 목록

### 다음 세션 첫 cycle — 분기 B (publish dispatch)

M3-5 honest re-baseline 완료. **publish 차단 사유 없음**.

| # | 태스크 | 명령 |
|---|--------|------|
| B.1 | M3-3: npm publish dry_run | `gh workflow run npm-publish.yml -f dry_run=true` |
| B.2 | M3-3: npm 실 publish | `gh workflow run npm-publish.yml -f dry_run=false` |
| B.3 | M3-4: PyPI publish dry_run | `gh workflow run pypi-publish.yml -f publish=false -f repository=pypi` |
| B.4 | M3-4: PyPI 실 publish | `gh workflow run pypi-publish.yml -f publish=true` |
| B.5 | publish 결과 24h 모니터링 | 자율/HUMAN 혼합 |

### 분기 C — M4-1 B baseline (HUMAN setup + 자율 실행)

이전 세션 HANDOFF 그대로 (변경 없음):
- C.1 `.env.local BMB_BENCH_API_KEY` 설정 — HUMAN
- C.2 `bmb-ai-bench doctor` + dry-run — 자율
- C.3 `--all --runs 3 --model claude-sonnet-4-6` (8-12h) — 자율
- C.4-C.5 결과 commit + B-track 갱신

### 자율 분리 phase 후보 (별도 세션)

| # | 태스크 | 성격 |
|---|--------|------|
| (1) | `inttoptr` codegen 전환 | 5-10 cycles, P3 |
| (2) | HashMap 3% 갭 fix | 3-5 cycles |
| (3) | `or` chain proper fix | 3-5 cycles |
| (4) | **Tier 3 inproc 변환 (또는 workload amplification)** | ISSUE-20260512-tier3-spawn-overhead-methodology |
| (5) | ~~bmb-algo quicksort FFI 최적화~~ | ❌ 취소 (median-of-5 1.7× FAST 확인, 별도 ISSUE close) |

---

## 3. 핵심 산출물 (이번 phase, Cycles 2760-2764)

### Code 산출

- `ecosystem/bmb-algo/benchmarks/bench_algo.py` — `run()` median-of-N + `--runs=N` argparse + `--scaling` sweep + min-max spread reporting

### Documentation 산출

- `docs/LANGUAGE_REFERENCE.md` § 10.4 — `len: i64{it < len}` 자기모순 예제 정정 (Cycle 2761)
- `ecosystem/bmb-algo/README.md` — headline "Up to ~245×" 정정, median-of-5 + min-max table, 3중 archival narrative
- `ecosystem/bmb-algo/CHANGELOG.md` `[Unreleased]` — re-baseline narrative + variance 인프라 entries
- `claudedocs/ROADMAP.md` — § 6 Cycle 2760-2764 sub-section + M3-5 row 갱신 + M3 progress 99%

### ISSUE 산출

- Closed: `quicksort-ffi-overhead` (재현 불가, median-of-5 baseline)

### Measurement 산출

- median-of-5 baseline (2026-05-12): 9 algorithms + 9 scaling configs

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

- **bmb-algo bench 측정 시 반드시 median-of-N 사용** (NEW Cycle 2762-2763): n=2 sampling은 5% variance 가정이 무너짐. `bench_algo.py --runs=5` 권장. 직접 측정 도구에 통합.
- 이전 세션 운용 주의사항 그대로:
  - Tier 3 spawn overhead methodology (ISSUE-20260512-tier3-spawn-overhead-methodology)
  - bmb-algo는 submodule 아님 (직접 directory)
  - BMB_ARENA_MAX_SIZE default 32G
  - Token packing 5M scale
  - FP builtin arity guard 미적용

---

## 5. 다음 세션 시작 체크리스트

### 기본 검증
- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커, Cycle 2760-2764 갱신)
- [ ] `cargo test --release` → 6210/6210 (옵션, BMB compiler 변경 없음)

### Publish 진입 (분기 B, M3-5 완결)
- [ ] `gh workflow run npm-publish.yml -f dry_run=true`
- [ ] dry_run artifact 검증
- [ ] `dry_run=false` 재실행
- [ ] PyPI 동일 절차
- [ ] 24h 모니터링 (npm metadata + README rendering)

### M4-1 B baseline (분기 C, HUMAN setup 후)
- [ ] `BMB_BENCH_API_KEY` 설정 확인
- [ ] `bmb-ai-bench doctor` PASS
- [ ] `--all --runs 3` 실행

---

## 6. HUMAN 결정 사항 (Cycles 2737-2764 누적)

| 항목 | 결정 |
|------|------|
| M3 showcase | ✅ bmb-algo |
| npm publish | ✅ Cycle 2764 이후 **즉시 dispatch 가능** |
| PyPI publish | ✅ Cycle 2764 이후 **즉시 dispatch 가능** |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 모델 | ✅ **claude-sonnet-4-6** |
| **M3-5 README/CHANGELOG (Cycles 2753-2754 → 2760-2764)** | ✅ honest re-baseline 완결 (median-of-5, 3중 archival disclose) |
| M3-6 CI flag | ✅ Cycles 2746-2747 완결 |
| M3-7 baseline annotation | ✅ M4-1 종속 자동 |
| **신규 (Cycle 2763)**: quicksort-ffi ISSUE close | ✅ 자율 (재현 불가, median-of-5) |
| **신규 (Cycle 2761)**: LANGUAGE_REFERENCE § 10.4 자기모순 fix | ✅ 자율 (review 발견) |

---

## 7. 이번 phase의 메타 통찰

### 1. n=2 vs n=5 sampling — measurement integrity의 본질

Cycle 2754 "442× vs 447× → 5% variance 정상" 진술은 통계적으로 무효. 두 sample만으로는 spread를 알 수 없음. advisor 권고대로 5회 측정 시 **inter-run variance가 일부 항목에서 30-50%까지 vary** (특히 Python baseline의 cache/load 의존). median-of-N이 표준이 되어야 함.

**BMB 철학 정렬**:
- "측정 없는 성능 주장 금지" — n=1/n=2 = "측정 없음"의 보다 미묘한 형태
- median-of-N + min-max spread = 정직성의 정량 표준

### 2. Review의 leverage

Cycle 2760 review (분기 A 보조)가 단순 "narrative tweak" 검토에서 시작했으나, finding 1 (LANGUAGE_REFERENCE 버그) + finding 2 (scaling table 재현 불가) → 측정 자체의 무결성 의문 → median-of-5 재baseline 까지 확장. **review는 통과 도장 찍는 도구가 아니라 측정 무결성 검증 단계**.

### 3. v0.2.0 → Cycle 2754 → 현재 — 3중 archival 정직성

| 측정 | 표 위치 | 책임 |
|------|---------|------|
| v0.2.0 90× | Historical archived | 2026-03-23 단일 측정 (config 미상) |
| Cycle 2754 450× | Withdrawn (CHANGELOG에서 explicit) | n=2 outlier |
| 현재 243× (median-of-5) | Current baseline | 재현 가능 |

3중 disclose는 narrative 신뢰도 손실이 아닌 강화 — "우리는 우리의 측정 오류를 숨기지 않는다"는 신호.

### 4. ISSUE 등록 → close의 정직 cycle

Cycle 2752 quicksort-ffi ISSUE 등록 → Cycle 2763 close (not reproducible). 빠른 close는 부정적 신호 아님 — 측정 무결성 강화 결과. ISSUE 등록 자체는 옳았다 (n=1 발견 시 disclose는 정직성). close는 더 나은 측정 도구 (median-of-5) 도입의 결과.

---

## 8. 다음 세션 첫 cycle 권고

### Cycle 2765 — 분기 B publish dispatch

```bash
# 1) npm dry_run
gh workflow run npm-publish.yml -f dry_run=true
# 결과 검증 후
gh workflow run npm-publish.yml -f dry_run=false

# 2) PyPI 동일 절차
gh workflow run pypi-publish.yml -f publish=false -f repository=pypi
gh workflow run pypi-publish.yml -f publish=true
```

### Cycle 2766+ — 분기 C 또는 자율 분리 phase

분기 C (M4-1) HUMAN API key setup 후 자율 실행 가능. 또는 자율 분리 phase (4) Tier 3 inproc / (1) inttoptr codegen 진입.

---

**세션 종료**: 2026-05-12 (Cycles 2760-2764 — M3-5 honest re-baseline 완결). HEAD TBD (Cycle 2764 commit).
