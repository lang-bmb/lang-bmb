# Cycle 2770: Sorting rebuild regression 진단 — P1 ISSUE 등록

Date: 2026-05-12

## Re-plan

진입 — cycle 2769 carry-forward (sorting hang 진단 P1). Trigger ⚪ NONE.

## Scope & Implementation

### Step 1: 재현

- `main.exe` (Feb 9 build): 234ms wall, output `403905348` ✅
- `main_verify.exe` (May 12 rebuild from same source): >120s wall, empty stdout ❌
- 3 attempts via Python subprocess (timeout=120): 100% reproducible
- 직접 shell `./main_verify.exe`: wall 2m+, user 0.1s CPU → IO bound 또는 internal sleep

### Step 2: 차이 분석

| 측면 | Feb 9 main.exe | May 12 main_verify.exe |
|------|---------------|----------------------|
| File size | 204 KB | 350 KB (+71%) |
| PE32+ sections | 18 | 19 |
| Wall time | 234ms | >120s (500× slowdown) |
| Output | `403905348` | (empty) |

소스 동일 (git working dir clean for sorting/main.bmb). **Rust compiler 변경이 회귀 원인**.

### Step 3: 가설

- A: Cycle 2532 `noinline` pass 부착으로 hot loop call overhead
- B: opt -O2 transformation regression
- C: 다른 codegen 변경 (text/inkwell parity, readonly enum, M5-* 인프라 등)

미진단: git bisect 필요. 본 cycle budget 부족.

### Step 4: ISSUE 등록

`claudedocs/issues/ISSUE-20260512-sorting-rebuild-regression.md`:
- **P1** (M1 P-track 측정 차단)
- estimated_cycles: 3-7 (hypothesis)
- Option A (bisect + fix, proper) / B (bench workaround) / C (bootstrap 전환)
- HUMAN 결정 필요 — Rule 6 충돌

### Step 5: 정리

본 cycle 빌드된 `main_verify.exe` 17 개 (Tier 1/3 verify 도구 출력) 삭제 — 옛 `main.exe` 보존, 회귀 추적 baseline 유지.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 재현 (3 attempts) | ✅ 100% hang |
| 옛 binary 보존 | ✅ Feb 9 main.exe 정상 작동 |
| Rust compiler 단독 회귀 (소스 동일) | ✅ |
| ISSUE 등록 | ✅ |
| `cargo test --release` | ✅ |
| main_verify.exe cleanup | ✅ 17 binaries 제거 |

**Defect**: sorting rebuild 회귀 P1. 본 cycle scope = 진단 + ISSUE. fix는 multi-cycle.

## Reflection

### advisor leverage (메타)

cycle 2769 verify 도구가 즉시 P1 회귀 감지. 도구 작성이 **value 입증** — cycle 2768 양식 강화 (cycle estimate hypothesis 필드) + cycle 2769 도구 추가 = **measurement integrity infrastructure** 누적 효과.

만약 verify 도구 없었으면: sorting 회귀가 Tier 3 측정에서 "noise" 또는 "system load"로 오해되었을 가능성. cycle 2750-2751 lexer/json_serialize "환경 변동성" 가설 일부도 실제로는 rebuild 회귀였을 가능성 — 재측정 필요.

### 외부 관점 — 6 dimensions

1. **Scope fit**: 충족 — P1 진단 + ISSUE 등록 1 cycle.
2. **Latent defects**: sorting 회귀가 알려졌으나 fix 미실시 (multi-cycle scope).
3. **Structural improvement opportunities**:
   - `verify_bench_outputs.py` CI 통합 (cycle 2769 carry-forward, cycle 2771 권고)
   - git bisect 자동화 (회귀 cycle 식별을 빠르게)
   - bench 출력 golden test 통합 (회귀 즉시 alert)
4. **Philosophy drift**: 없음. workaround 회피 (Option B 권고 안 함). proper fix path (Option A bisect)는 multi-cycle scope.
5. **Roadmap impact**:
   - P-track 신뢰도 재평가 — cycle 2750 c2729 측정 일부 (lexer/json_serialize "회귀")이 rebuild 회귀일 가능성
   - cycle 2771-2773 plan: CI 통합 + 가능한 fix
6. **User-facing quality**: N/A.

### 측정 신뢰도의 메타-우려

cycle 2750 BMB 측 absolute time 일관 +30~100% 증가 → "환경 변동성" 가설. 그러나 sorting 회귀 발견으로 **BMB binary rebuild 회귀**가 일부 원인일 가능성. P-track 측정값 stable-인지 verify 통과 확인 후 측정 권고.

## Carry-Forward

### Actionable (다음 cycle)

**Cycle 2771**: CI 통합 (`scripts/verify_bench_outputs.py` 를 `scripts/quick-check.sh` 또는 CI workflow에 추가). 회귀 즉시 alert.

### Structural Improvement Proposals

- **git bisect 자동화 스크립트**: sorting 회귀 commit 식별. 1-2 cycles.
- **bench golden test 통합**: bench output을 정식 golden test로. 1-2 cycles.
- **FP tolerance**: `verify_bench_outputs.py` epsilon arg 추가. 1 cycle.
- **sub-ISSUE 처리**: json_serialize bug (1-2 cycles), csv_parse (2-3 cycles), lexer (2-5 cycles)

### Pending Human Decisions

- M3-3/M3-4 publish (HUMAN, 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 누적)
- Rule 6 / Rule 7 충돌 — 특히 sorting Rust 회귀 fix이 "부트스트래핑 차단"에 해당하는가?
- bisect 시작 시점 (multi-cycle phase)

### Roadmap Revisions

cycle-logs `ROADMAP.md`:
- Phase D' (verify 도구) 완료 + 발견된 6 sub-ISSUE
- 잔여 plan:
  - Cycle 2771: CI 통합
  - Cycle 2772: 가능한 light fix (json_serialize char bug)
  - Cycle 2773: HANDOFF 사전 갱신
  - Cycle 2774: 세션 종료 commit

### Next Recommendation

**Cycle 2771**: CI 통합 — `scripts/quick-check.sh` 에 `python3 scripts/verify_bench_outputs.py --tier all --rebuild` 추가. exit code mapping (1=mismatch warn, 2=build fail block).

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| 신규 P1 ISSUE | `claudedocs/issues/ISSUE-20260512-sorting-rebuild-regression.md` | tracked |
| main_verify.exe cleanup (Tier 1/3) | (binary 삭제) | untracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2770.md` | gitignored |
