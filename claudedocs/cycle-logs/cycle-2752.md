# Cycle 2752: Bench framework C-side anomaly 점검 + M3-5 사전 조사

Date: 2026-05-12

## Re-plan

🟠 **RE-PLAN**: Cycle 2751 종료 후 advisor 호출. 핵심 지적:
- "BMB 측 측정에 영향 없음" framing이 http_parse BMB 자체 45→135 3x 변화를 간과
- Cycle 2751 lexer 1.000x verdict 신뢰성 의문
- M3-5 진입 전 framework 안정성 5분 investigation 권고
- 90x는 clang anomaly 가설 — M4-9 ISSUE 정렬 확인 필요

본 cycle: framework 안정성 점검 (5분) + M3-5 진입 전 bmb-algo 사전조사 (서브모듈/headline/bench 출력).

## Scope & Implementation

### Framework 안정성 점검 (~5분)

**Step 1 — Binary 신원 확인** (clang || gcc fallback 검증):
- `target/benchmarks/*_rw_c.exe` size = 138262 bytes (lexer)
- `clang -O3 -march=native ...` 명시 출력 = 138262 bytes (match) ✅
- `gcc -O3 -march=native ...` 명시 출력 = 141195 bytes (diff)
- **결론**: bench framework는 clang 사용 중. GCC strings은 MinGW libc runtime artifact (모든 Windows 바이너리에 포함)

**Step 2 — 직접 측정 (Python perf_counter_ns × 10-run, shell=True)**:

| bench | BMB direct | C direct | ratio |
|-------|-----------|----------|-------|
| lexer | min=7ms med=8ms | min=7ms med=7ms | **1.000** |
| http_parse | min=6ms med=7ms | min=6ms med=7ms | **1.000** |
| brainfuck | min=6ms med=7ms | min=6ms med=7ms (one 104ms outlier) | **1.000** |

**결정적 발견**: Tier 3 워크로드 실시간 = 6-8ms. bench framework가 보고한 28-135ms는 **OS process-spawn overhead 노이즈 floor**, 워크로드 시간 아님.

### Framework methodology defect

`run_benchmark` (`scripts/benchmark.sh:154`):
```bash
time_cmd() {
    local start=$(get_time_ms)
    "$@" > /dev/null 2>&1 || true
    local end=$(get_time_ms)
    echo $((end - start))
}
```

이 측정은 (1) bash subshell fork, (2) exe spawn, (3) redirection, (4) **workload (~7ms)**, (5) teardown 합산. Tier 3 short benches는 workload < spawn overhead.

**시스템 부하 변동 → spawn overhead 비례 변동**:
- c2729 (2.5h tier_all 백그라운드 실행 중간): bmb=28, c=28
- c2751 (36s tier3-only 깨끗): bmb=41, c=41 (전반 +50%)
- c2729 light bench (brainfuck c=45) vs c2751 (c=133) = **3x OS jitter**

**Cycle 2751 verdict reframe**:
- "lexer 1.000x" — ✅ 올바름 (BMB와 C 같은 spawn 오버헤드 부담, 실제 워크로드도 parity)
- "회귀 가설 기각" — ✅ 올바름 (회귀가 아닌 OS jitter)
- **caveat**: Tier 3 wall-time 측정은 신호 < 노이즈. 절대값 (28, 41, 51, 135 등)에 의미 부여 회피.

### bmb-algo 사전 조사 (Cycle 2753 진입 준비)

**Submodule status**: `bmb-algo`는 `.gitmodules`에 없음 — **부모 repo 직접 디렉토리**. PR overhead 없음 ✅ (Cycle 2747 maintainer 직접 main merge 정책 무관).

**README headline 확인**:
> 90x faster than Python on knapsack. 181x faster on N-Queens.

표 위치 헤더는 "**Benchmarks (vs Pure Python)**" — `clang/gcc` 표현 없음. headline은 Python 비교 (advisor 가설 "clang -O3 outlier"와 무관).

**90x/181x source**:
- `bmb-algo/CHANGELOG.md` v0.2.0 (2026-03-23):
  - "knapsack: **90.7x** faster than Python, **6.8x** faster than C"
  - "nqueens(8): **181.6x** faster than Python"
- (note: nqueens **8** not 10 — bench 코드는 10 사용)
- v0.2.0 측정 이후 bench config 변경 추정 (knapsack 입력 크기 다를 수 있음)

**현 v0.98 bench 측정** (`python3 bench_algo.py`, 2026-05-12):

| Algorithm | 현 README 표 | v0.98 측정 | 분류 |
|-----------|------------|-----------|------|
| knapsack | 6.3x (100 items 라벨) | **7.46x** (실제 10 items) | 라벨 잘못 + 정정 가능 |
| nqueens(10) | 4.1x | 5.34x | 정정 가능 |
| prime_count(10k) | 32x | 42.86x | 정정 가능 |
| edit_distance | 6.4x | 7.89x | 정정 가능 |
| merge_sort(15) | 3.3x | 2.91x | 정정 가능 (-) |
| **quicksort(15)** | 2.1x | **0.60x SLOW** ⚠️ | **회귀** |
| fibonacci(30) | 2x | 1.48x | 정정 가능 (-) |
| Headline 90x knapsack | (v0.2.0 historical) | 7.46x | **재현 불가** |
| Headline 181x nqueens | (v0.2.0 historical, n=8) | 5.34x (n=10) | **재현 불가** |

### 갱신 파일

| 파일 | 변경 |
|------|------|
| `claudedocs/ROADMAP.md` | 변경 없음 (Cycle 2751 sub-section 그대로 유효) |

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Binary 신원 (clang vs gcc) | ✅ clang (size match) |
| 직접 측정 (3 benches × 10 runs) | ✅ 모두 ~7ms |
| Cycle 2751 verdict 정합성 | ✅ "lexer 1.000x" 정확 (workload parity), wall-time absolute는 노이즈 |
| bmb-algo submodule 여부 | ✅ 아님 (부모 directory) |
| 90x/181x source 추적 | ✅ v0.2.0 CHANGELOG, vs Python (NOT clang) |
| v0.98 현 bench 측정 | ✅ knapsack 7.46x / nqueens 5.34x (headline 재현 불가) |
| quicksort 회귀 발견 | ⚠️ 2.1x → 0.60x — 진단 별도 |

결함: 

1. **quicksort 회귀** (2.1x → 0.60x) — bench_algo.py 실측. M3-5 narrative 정정 시 disclose 필수.
2. **README knapsack(100 items) 라벨 ≠ 코드 (10 items)** — 라벨 정정 (M3-5 잘 알려진 항목).
3. **Tier 3 framework methodology**: workload <spawn overhead. inproc 변환 또는 workload 100x amplification 권고 — 별도 multi-cycle phase.

## Reflection

### advisor 절제 + 검증의 leverage

advisor 우려 → 5분 investigation → 3 sub-finding:
- (a) clang/gcc fallback은 정상 (clang 사용 중)
- (b) workload 자체는 7ms parity, framework wall-time이 OS spawn overhead 측정 중
- (c) bmb-algo는 submodule 아님, headline은 vs Python (M4-9 clang anomaly 무관)

5분 investment로 Cycle 2753 entry plan을 narrowed scope (label 정정 + headline 재정정 + quicksort 회귀 disclose) 으로 정합.

만약 investigation skip하고 Cycle 2753 진입 시: 90x reproducibility 가정 → bench 실행 → 가정 깨짐 → 1 cycle 분 분석 부담. → advisor 절제 ROI 1.5x cycle.

### Cycle 2750 reframe (필요시 ROADMAP 정정)

ROADMAP § 5 Cycle 2751 sub-section의 "회귀 가설 기각" 표현은 ✅ 정확. 단 "Tier 3 5-run vs 10-run" 비교 결론은 **부분만 옳음**: 실제로는 두 측정 모두 OS overhead 노이즈 위에서 BMB:C ratio가 의미 있는 정보. **절대 ms 변화**에 진단적 의미 부여 회피.

ROADMAP 정정 불필요 (이미 "환경 변동성 가설 우위" 명시), 단 Cycle 2752 sub-section을 추가하여 framework methodology defect를 표시할 가치 있음.

### M3-5 narrative 옵션 재평가

원 HANDOFF § 2.5 시퀀스 B.3 headline 옵션:
- (a) v0.98 측정값으로 재정정
- (b) 100-items bench 변종 추가
- (c) "Up to" 약화 표현

advisor 평가: (c) "Up to" 약화는 v0.2.0 / v0.98 차이 hide → 부정확. → **option (a) 권고**: v0.98 numbers + v0.2.0 archival note.

**Cycle 2753 권고**:
- (a) headline 정정: v0.98 numbers (knapsack 7.46x, nqueens 5.34x) + v0.2.0 historical 7.46x → 90.7x discrepancy를 "archived measurement at v0.2.0, bench config diverged" 형식으로 disclose
- README 표 "(100 items)" 라벨 정정 → "(10 items, cap 20)"
- quicksort regression disclose + 분리 ISSUE 등록
- CHANGELOG [Unreleased] re-baseline annotation

### Pacing 점검 (advisor 지적)

소비 cycle: 3/10 (2750/2751/2752). 잔여 7 cycles:
- Cycle 2753: M3-5 정정 (1 cycle)
- Cycle 2754: quicksort 회귀 분석 (1 cycle 추정)
- Cycle 2755+: 자율 백로그 4-5 cycles

가능한 자율 작업:
- multiple-pre-clauses 파서 spec 확장 (1-2 cycles, 언어 spec) — bounded
- FP arity guard 36 sites mechanical (1 cycle, low ROI) — bounded
- ISSUE 백로그 cleanup (variable, 0-2 cycles)
- Tier 3 inproc 변환 multi-cycle phase 시작 (5-10 cycles) — **잔여 cycle 부족, 시작 회피**

→ Cycle 2755-2759는 bounded single-cycle 작업 우선. multi-cycle phase는 별도 세션.

## Carry-Forward

### Actionable

- **Cycle 2753 = 시퀀스 B M3-5**: bmb-algo README 정정 (knapsack(10 items) 라벨 + v0.98 numbers + headline 정정 + quicksort regression disclose + CHANGELOG annotation)
- **Cycle 2754 = quicksort 회귀 분석**: bmb-algo `quicksort(SORT_DATA)` 2.1x → 0.60x 원인 진단 (15-elem 입력에서 Python list 슬라이싱 vs BMB ctypes FFI 오버헤드 가설; ctypes call 자체가 짧은 입력에 비효율적일 수 있음)

### Structural Improvement Proposals

1. **Tier 3 measurement methodology — inproc 변환 multi-cycle phase**: 현 framework는 short benches에서 신호 < 노이즈. 옵션:
   - (a) Tier 3 bench main()을 `time_ns()` + `bmb_black_box` 로 in-process 측정 (Cycle 2661 Tier 1 inproc 패턴) — 5-10 cycles
   - (b) Tier 3 workload 100x amplification (현 1배 → 100x 입력) — 1-2 cycles, 빠르지만 amplification factor 임의
   - 권고: option (b) 우선 (낮은 비용, framework 변경 불요). 별도 세션 multi-cycle phase.

2. **bench_algo.py knapsack 100-items config 추가** (M3-5 B.3 옵션 b 변종): 현 10-items + 100-items dual config로 README "Benchmarks (vs Pure Python at small / large)" sub-section 분리. 1 cycle, 추후 candidate.

### Pending Human Decisions

- 신규 없음
- 기존 큐 유지

### Roadmap Revisions

ROADMAP § 5 변경 없음. Cycle 2752 sub-section은 추가하지 않음 (framework methodology defect는 Carry-Forward에 명시, ROADMAP은 측정값 변동을 기록하는 anchor — methodology 논의는 ISSUE 등록이 더 적합).

### Next Recommendation

**Cycle 2753: M3-5 bmb-algo README 정정 + CHANGELOG annotation**

상세:
- README headline: "Up to 43x faster than Python (prime_count)" + table updated with v0.98 numbers
- knapsack label "(100 items)" → "(10 items, cap 20)"
- 90x/181x archival note: "v0.2.0 (2026-03-23) measured 90.7x knapsack and 181.6x nqueens(8) — bench config has since diverged, see CHANGELOG."
- quicksort 회귀 inline note: "(15-elem ctypes overhead — known)" 또는 disclose + 별도 ISSUE 등록
- CHANGELOG [Unreleased]: "measurement re-baseline 2026-05-12 — supersedes 2026-03-23 v0.2.0 numbers"

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| (없음, 측정 + 조사 only) | — | — |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2752.md` | gitignored |
