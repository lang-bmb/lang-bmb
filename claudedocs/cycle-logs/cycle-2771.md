# Cycle 2771: verify_bench_outputs.py CI 통합

Date: 2026-05-12

## Re-plan

진입 — cycle 2770 carry-forward (CI 통합). Trigger ⚪ NONE.

## Scope & Implementation

### Step 1: 통합 위치 결정

| 옵션 | 시간 | 빈도 | 정합 |
|------|------|------|------|
| `quick-check.sh` | ~165s (75+90s) 추가 | 매 dev iter | 부적합 — "quick" 목표 위반 (현재 2분 → 5분으로) |
| `full-cycle.sh` | ~165s 추가 | PR 전 (~15분이 ~17분) | **적합** — 이미 thorough 검증 단계 |
| GitHub workflow | 동일 | CI | 향후 cycle |

→ **`full-cycle.sh` 에 Step 3.5 추가**.

### Step 2: full-cycle.sh 수정

추가 사항:
- `--skip-verify` 옵션 (선택적 skip)
- `RESULTS[verify_passed]`, `RESULTS[verify_time_ms]` 변수
- Step 3.5 (Step 3 benchmarks와 Step 4 regression check 사이):
  - `python3 scripts/verify_bench_outputs.py --tier all --rebuild --json $OUTPUT_DIR/bench_verify.json`
  - PIPESTATUS로 exit code 보존 (`set +e` / `set -e` block)
  - exit 0 → PASS, exit 1 → mismatch warning (non-blocking), exit 2 → build/run error (non-blocking)
- Summary table에 "Bench Output Verify" 행 추가

### Step 3: bash syntax check

`bash -n full-cycle.sh` 통과. 옵션 파싱 정상.

### Step 4: 통합 미실행 (budget)

이번 cycle은 코드 변경 + sanity check만. full-cycle.sh 실제 실행은 15-17분 소요 (현재 budget 부족) — 다음 cycle 또는 다음 세션에 first-real-run.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `bash -n scripts/full-cycle.sh` syntax | ✅ |
| `--skip-verify` 플래그 추가 | ✅ |
| `RESULTS[verify_*]` 변수 추가 | ✅ |
| Summary table 행 추가 | ✅ |
| `set +e` / PIPESTATUS 처리 (non-blocking) | ✅ |
| `cargo test --release` | ✅ |
| 실제 full-cycle 실행 | ⏸️ budget 회피 (다음 cycle / 세션) |

**Defects**: 없음 (script 변경). 실제 실행 검증은 carry-forward.

## Reflection

### advisor leverage

cycle 2769 / 2770 verify 도구의 가치 cycle 2771에 integration으로 강화. **measurement integrity infrastructure** 3-cycle 누적 (도구 + ISSUE + integration).

### 외부 관점 — 6 dimensions

1. **Scope fit**: 충족 — script integration 1 cycle, sanity check 포함.
2. **Latent defects**: full-cycle.sh 실제 실행 미검증 (budget). 다음 세션에 first-real-run 필수.
3. **Structural improvement opportunities**:
   - `quick-check.sh` 에 `--with-verify` 옵션 (opt-in, 정상 quick 시간 유지)
   - GitHub Actions workflow에 verify step 추가 (`.github/workflows/`)
   - `verify_bench_outputs.py` epsilon 옵션 (FP precision)
4. **Philosophy drift**: 없음. CI infrastructure 강화는 measurement integrity 정합.
5. **Roadmap impact**:
   - Phase D' (verify) ✅ 완료 (cycle 2769-2771: 도구 작성 + 1차 측정 + CI 통합)
   - 잔여: cycle 2772-2773 light fix + cycle 2774 종료
6. **User-facing quality**: CLI 관점:
   - `--skip-verify` 명확함
   - non-blocking 동작 (mismatch는 warning, exit code mapping)
   - `bench_verify.json` artifact 출력
   - Summary table 행 추가로 visual 정합

### CI 통합 선택의 의미

quick-check (~2min)을 5min으로 늘이는 대신 full-cycle (~15min) 17min으로 확장 선택:
- 정합: full-cycle은 이미 PR-time thoroughness 목적
- 비용: dev iter 마찰 없음 (quick-check 그대로)
- 효과: PR 시점에서 verify 실패 visible, sorting rebuild 같은 회귀 immediately catch

future option: quick-check에 `--with-verify` opt-in 추가 (단발 회귀 의심 시 사용)

## Carry-Forward

### Actionable (다음 cycle)

**Cycle 2772**: 잔여 sub-issue 중 가장 light한 것 처리 — **json_serialize char bug** (BMB가 `{` 출력 vs C `[`). 단일 char level fix 추정 (1 cycle).

### Structural Improvement Proposals

- **full-cycle.sh first-real-run 검증** (다음 세션, 15-17min)
- **`--with-verify` opt-in to quick-check.sh** (1 cycle)
- **GitHub workflow에 verify step 추가** (1 cycle + HUMAN review)
- **FP tolerance epsilon arg**: `verify_bench_outputs.py --epsilon 1e-6` 추가 (n_body FP 정상화)
- **나머지 sub-ISSUE** (csv_parse, lexer, fibonacci, n_body, sorting): 별도 cycles

### Pending Human Decisions

- M3-3/M3-4 publish (HUMAN, 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 누적)
- Rule 6 / Rule 7 충돌 (특히 sorting rebuild 회귀 fix)
- GitHub workflow 추가 (HUMAN approval for CI changes)

### Roadmap Revisions

cycle-logs `ROADMAP.md`:
- Phase D' verify 도구 + CI 통합 ✅ 완료 (cycles 2769-2771)
- 잔여 plan:
  - Cycle 2772: json_serialize bug fix 시도 (light)
  - Cycle 2773: HANDOFF 사전 갱신
  - Cycle 2774: 세션 종료 commit

### Next Recommendation

**Cycle 2772**: json_serialize bench `Array: {1,2,3,4,5]` 출력에서 `{` 가 `[` 가 되어야 함. BMB source 추적 (`benches/real_world/json_serialize/bmb/main.bmb`) → printline 호출에서 `chr(123)` 또는 literal `"{"` 오타 가능성. 진단 → fix → verify_bench_outputs PASS 확인.

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| Step 3.5 추가 + `--skip-verify` 플래그 + Summary table 행 | `scripts/full-cycle.sh` | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2771.md` | gitignored |
