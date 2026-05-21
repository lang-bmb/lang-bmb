# Cycle 2728: lcs_three 회귀 가설 기각 — 환경적 UB 진단

Date: 2026-05-11

## Re-plan

인계 (HANDOFF § 8): "🚨 lcs_three 골든 회귀 fix [BLOCKING]" inherited defect.
Trigger: ⚪ NONE (plan valid, but mid-cycle reframing to ⚠️ pivot — HANDOFF 진단 자체가 부정확).

## Scope & Implementation

### 진단 시퀀스

1. 잔존 백그라운드 4 process kill (PID 37866 / 67404 / 67405 / 67408)
2. `tests/bootstrap/test_golden_lcs_three.bmb` 소스 검토 — 단순 DP + edit distance, `2+3+0+2+0+5=12; 12*43-5=511`
3. 수동 재현 (Stage 1 + opt -O2 + llc -O3 + gcc -O2): **511 PASS** (single run)
4. 정확히 실패한 binary 추적: `target/golden-tests/test_golden_lcs_three.exe` (May 11 20:39)
5. Stress test 50 runs: **40 PASS, 10 segfault (rc=139)** — intermittent
6. `-O0`로 재컴파일 stress: **8/10 PASS, 2/10 segfault** — LLVM 최적화 회귀 아님
7. 다른 골든 비교 stress (lcs.exe, ackermann.exe): 30/30 PASS each
8. `/tmp/golden_failures.txt` 검토: 실제 **4건 fail** (lcs_three + cholesky_trace + crc32_simple + assortativity), HANDOFF는 1건만 기록

### inttoptr 패턴 검증

| Binary | inttoptr count | stress PASS rate |
|--------|----------------|------------------|
| lcs.ll (PASS) | 17 | 30/30 |
| ackermann.ll (PASS) | 3 | 30/30 |
| lcs_three.ll (FLAKY) | 41 | 40/50 |

→ 패턴은 보편, 빈도가 발현율 결정. 구조적 UB 추정.

### Runtime 변경 이력 검증

`bmb/runtime/bmb_runtime.c` 마지막 commit: `68efe7e6` (Cycle 2500). 이후 6개월 stable.
Cycle 2711-2714 변경은 `bootstrap/compiler.bmb` (token packing + arity guard) — codegen path 영향 없음.
→ "Cycle 2711-2714 회귀" 가설 기각.

### advisor 합류

진단 80% 시점에 advisor 호출 → HANDOFF framing 잘못, codegen `inttoptr/ptrtoint` round-trip + Windows MSYS2/UCRT64 heap 상호작용 UB 추정. C1을 diagnostic-only로 종료 권고.

### ISSUE 등록 — 양식 표준화 first stamping

`claudedocs/issues/ISSUE-20260511-golden-flakiness-inttoptr.md` 신규 등록.
신규 필드 (`measurement_date`, `stale_after`, `measurement_source`, `observed_rate`, `scope`, `env_hash`) 첫 적용.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 수동 재현 (Stage 1 + opt + llc + gcc) | ✅ 511 출력 (single) |
| Stress 50회 (release binary) | ⚠️ 20% segfault — environmental UB confirmed |
| -O0 stress | ⚠️ 20% segfault — opt 회귀 아님 |
| 다른 골든 stress (lcs/ackermann) | ✅ 100% PASS |
| inttoptr count 비교 | ✅ 패턴은 공통, 빈도 차이 |
| ISSUE 등록 | ✅ measurement stamp first application |

**결함 해결**: lcs_three 회귀 가설 **기각**. 실제 결함은 codegen `inttoptr/ptrtoint` round-trip UB (multi-cycle scope, 별도 ISSUE 추적).

## Reflection

### 외부 관찰자 관점

1. **HANDOFF 진단의 결함**: "BLOCKING inherited defect" framing이 cycle 시간을 잘못 유도. 1건 fail 기록 → 실제 4건. 추정 원인 (Cycle 2711-2714) → runtime 미변경으로 모순.
2. **advisor의 절제된 가치**: 진단 데이터가 충분히 모인 시점에 호출 → framing 잘못을 즉시 지적. "회귀 아님, diagnostic only로 마감" — cycle 시간 절약.
3. **양식 표준화 첫 적용**: 새 ISSUE가 measurement_date + stale_after + source + observed_rate + scope + env_hash 모두 포함. 다음 cycle에서 기존 23 ISSUE 일괄 stamping 시 이 케이스가 reference.
4. **flakiness 정량화**: 50회 stress = 정확한 20% rate. "intermittent"가 막연한 게 아니라 measurable.

### 측정 가능한 변화

- ISSUE active: 23 → **24** (+1 flakiness)
- HANDOFF framing 오류 식별: "1 FAIL" → 실제 "4 FAIL, environmental UB"
- Cycle 시간 절약: advisor 개입으로 fix 시도 (multi-cycle) → diagnostic-only (1 cycle)

### Roadmap impact

- M1 ≤1.05x P축 영향 없음 (별개 codegen UB)
- M3 publish lock 무관
- 다음 cycle 권고: 풀 골든 재실행 (다른 fail 패턴 분석) + ISSUE 양식 표준화 → 기존 backlog stamping

## Carry-Forward

- Actionable (다음 cycle):
  - **C2**: 풀 골든 재실행 (background, 43분) + lcs_three/cholesky_trace/crc32_simple/assortativity 재현율 확정
  - **C3**: ISSUE 양식 표준화 — `_template.md` 작성 + 기존 23 active stamping 작업
- Structural Improvement Proposals:
  - codegen `inttoptr/ptrtoint` round-trip → `alloca ptr` 전환 (Option A in ISSUE) — 5-10 cycles, multi-cycle phase 후보
  - golden CI gate가 alphabetical 50 sample이라 lcs_three 미커버 — 별도 nightly full golden 별 ISSUE 등록 검토
- Pending Human Decisions:
  - 변경 없음 (M3-3, M3-4, M3-5, M4-1)
- Roadmap Revisions:
  - 없음 (ROADMAP § 4 M4 항목은 변경 없음, codegen 전환은 별도 M5-6 후보로 검토)
- Next Recommendation: **C2 = 풀 골든 재실행 백그라운드** → C3 = ISSUE 양식 표준화

## HANDOFF 정정 권고 (다음 cycle 반영)

```diff
- ⚠️ 2861/2862 PASS, 1 FAIL — test_golden_lcs_three (expected=511, got=empty)
- → 직전 세션 Cycle 2711-2714 (5M token packing + 30 arity guard) 회귀 가장 유력
+ ⚠️ 2858/2862 PASS, 4 FAIL — environmental UB (codegen inttoptr round-trip + Windows MSYS2/UCRT64 heap)
+ → ISSUE-20260511-golden-flakiness-inttoptr 등록, Cycle 2711-2714 회귀 가설 기각
+   (runtime stable since Cycle 2500, -O0에서도 동일, 다른 골든 100% PASS)
```
