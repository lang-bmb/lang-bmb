# Cycle 2769: bench output verification 도구 + 1차 측정

Date: 2026-05-12

## Re-plan

진입 — cycle 2768 carry-forward (bench output verification 도구 작성). Trigger ⚪ NONE.

## Scope & Implementation

### Step 1: 도구 작성

`scripts/verify_bench_outputs.py` (240 lines):
- argparse: `--tier {1,3,all}`, `--rebuild`, `--verbose`, `--json`
- 입력: Tier 1 (10 compute benches) + Tier 3 (7 real_world benches) hardcoded
- 절차: BMB + C 각각 build (옵션 rebuild) → run → normalize stdout → diff
- 출력: human-readable + JSON
- Exit code: 0 = all PASS, 1 = mismatch, 2 = build/run fail

normalize: trailing whitespace strip + collapse final blanks. FP precision differences detected as mismatch (별도 epsilon 정책 carry-forward).

### Step 2: Tier 3 측정 + 발견

7 benches, 75s wall:

| bench | 결과 |
|-------|------|
| brainfuck | ✅ PASS |
| **csv_parse** | ❌ MISMATCH — Strings 라인 누락 등 구조 diff |
| http_parse | ✅ PASS |
| json_parse | ✅ PASS |
| **json_serialize** | ❌ MISMATCH — `Array: {1,2,3,4,5]` (`[` 대신 `{`) |
| **lexer** | ❌ MISMATCH — pre-existing 0-token bug (cycle 2765 확인) |
| **sorting** | ❌ FAIL — 재빌드 binary hangs (main_verify.exe vs main.exe) |

### Step 3: Tier 1 측정 + 발견

10 benches, 92s wall:

| bench | 결과 |
|-------|------|
| binary_trees / fannkuch / fasta / hash_table / knapsack / mandelbrot / nqueen / spectral_norm | ✅ PASS (8) |
| **fibonacci** | ❌ FAIL (C run, returncode≠0) |
| **n_body** | ⚠️ MISMATCH — FP precision diff (~7th decimal, e.g., -0.169075211 vs -0.169075164) |

### Step 4: 통합 ISSUE 등록

`claudedocs/issues/ISSUE-20260512-bench-output-fairness-survey.md`:
- P2 ranking (multi-bench)
- 4 unfairness + 2 build/run fail 정리
- 6 sub-ISSUE 후보 명시 (sorting P1 우선)
- CI 통합 권고 (sub-cycle)
- FP tolerance 정책 carry-forward

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 도구 작성 (240 LOC) | ✅ `scripts/verify_bench_outputs.py` |
| Tier 1 측정 | ✅ 8/10 PASS, 2 issue 발견 |
| Tier 3 측정 | ✅ 3/7 PASS, 4 issue 발견 |
| JSON 출력 | ✅ machine-readable |
| `cargo test --release` | ✅ (도구 작성은 BMB compiler 변경 없음) |
| ISSUE 등록 | ✅ 통합 ISSUE + 기존 ISSUE 갱신 |

**Defects 발견** (이번 cycle):

| # | bench | severity | 비고 |
|---|-------|----------|------|
| 1 | sorting | **P1** | 재빌드 binary hangs — 컴파일러 회귀 가능성 (main.exe 작동 vs main_verify.exe hang) |
| 2 | json_serialize | P2 | 단일 char bug (`{` vs `[`) |
| 3 | csv_parse | P2 | 구조적 출력 diff |
| 4 | lexer | P2 | 0-token bug (이전 ISSUE) |
| 5 | fibonacci | P3 | C run fail |
| 6 | n_body | P3 | FP precision diff (정상 vs algorithm diff 판별 필요) |

**Defect resolution**: 본 cycle scope = 도구 작성 + 1차 측정 + ISSUE 등록. 개별 fix는 sub-ISSUE / carry-forward.

## Reflection

### advisor leverage

cycle 2767 권고 ("bench output verification CI") 정확히 leverage. 도구가 **6개 실제 결함** 즉시 발견 — 양식 강화 (cycle 2768)와 더불어 **measurement integrity infrastructure** 강화의 두 번째 단계.

### 외부 관점 — 6 dimensions

1. **Scope fit**: 충족 — 도구 작성 + 측정 + ISSUE 등록.
2. **Latent defects**: **6개 발견**. 4개 unfairness (BMB:C 비교 의미 의문), 2개 build/run fail.
3. **Structural improvement opportunities**:
   - CI workflow에 verify 통합 (carry-forward)
   - FP tolerance 정책 (epsilon)
   - golden test로 bench output 포함 (회귀 즉시 검출)
4. **Philosophy drift**: 없음. measurement integrity는 BMB 핵심 (Verification Principle).
5. **Roadmap impact**:
   - **P-track 신뢰도 재평가 필요** — 17 Tier 1+3 중 4 unfairness = 24%
   - cycle 2750 1.040x noise 추정도 다시 봐야 (csv_parse / json_serialize의 BMB 측 단축 작업이 ratio에 영향?)
   - Cycles 2770-2773 plan: sub-ISSUE 진단 + 가능한 fix
6. **User-facing quality**: 도구 자체 UX — human-readable + JSON 양쪽 지원, exit code 의미 명시, `--verbose` 옵션. 1차 사용 시 충분히 readable.

### 측정 신뢰도 재평가

P-track 측정 통합 (e.g., Cycle 2750 `tier_all_c2729.json`):
- BMB 측 일부 bench가 다른 작업 수행 가능성 (csv_parse 41 fields vs 44 fields, lexer 0 tokens vs 8900)
- ratio 값이 fair 비교가 아닐 수 있음

→ 메타 권고: P-track ratio 측정 전에 **반드시 verify 통과 확인** 절차 표준화

## Carry-Forward

### Actionable (다음 cycle)

**Cycle 2770**: **sorting rebuild hang 진단** (P1 sub-ISSUE)
- main.exe 작동, main_verify.exe hang → 컴파일러 차이? 빌드 옵션 차이?
- Rust compiler 회귀 가능성 → 부트스트랩 차단 가능성 평가
- 진단 결과로 fix path 또는 carry-forward

### Structural Improvement Proposals

- **CI workflow 통합** (`scripts/quick-check.sh` 추가): 1 cycle
- **FP tolerance 정책** (epsilon arg 추가): n_body case 정상화. 1 cycle
- **json_serialize char bug fix**: 단일 char fix (`[` 출력 회복). 1-2 cycles
- **csv_parse 진단 + fix**: 구조 diff 원인 (parsing logic / output ordering). 2-3 cycles
- **lexer 0-token fix**: 별도 ISSUE 진행. 2-5 cycles
- **fibonacci C run fail**: 빠른 진단 (1 cycle)

### Pending Human Decisions

- M3-3/M3-4 publish (HUMAN, 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 누적)
- Rule 6 / Rule 7 충돌
- FP tolerance 정책 (절대값 vs 상대값)
- **신규**: P-track measurement 시 verify 선행 mandatory 여부

### Roadmap Revisions

cycle-logs `ROADMAP.md`:
- Phase B' (HashMap) ✅ 종료 (cycles 2766-2768, 3 cycles)
- Phase D' (bench output verification) ✅ 도구 작성 + 1차 측정 (cycle 2769)
- 잔여 4 cycles (2770-2773): sub-ISSUE fix (sorting P1 우선) + CI 통합 + 종료 (2774)

### Next Recommendation

**Cycle 2770**: sorting hang 진단. 절차:
1. main_verify.exe vs main.exe IR diff (`--emit-ir` 양쪽)
2. 빌드 옵션 비교 (Rust 빌드 명령어 + 환경)
3. gdb attach 또는 strace로 hang point 식별
4. 진단 결과로 fix 시도 또는 carry-forward

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| 신규 verify 도구 (240 LOC) | `scripts/verify_bench_outputs.py` | tracked |
| 통합 ISSUE 등록 | `claudedocs/issues/ISSUE-20260512-bench-output-fairness-survey.md` | tracked |
| 측정 artifacts | `/tmp/verify_tier1.json`, `/tmp/verify_tier3.json` | gitignored (tmp) |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2769.md` | gitignored |
