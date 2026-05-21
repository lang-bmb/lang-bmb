# Cycle 2742: bmb-ai-bench `run_crosslang.py` gcc flag README 정책 위반 fix

Date: 2026-05-11

## Re-plan

인계: Cycle 2741 carry-forward "ecosystem README quality grep". Trigger: 🟡 **SCOPE ADJUST** — README grep 진행 중 별도 defect 발견 (gcc flag 정책 위반), 우선 처리.

## Scope & Implementation

### 진단

`bmb-ai-bench` 정책:

| 위치 | 값 |
|------|----|
| `README.md` line 63 | "C baseline flags: `-O2 -march=native` unless overridden per-problem" |
| `bmb_ai_bench/registry.py` line 27 default | `c_baseline_flags: str = "-O2 -march=native"` |
| `bmb_ai_bench/registry.py` line 55 fallback | `meta.get("c_baseline_flags", "-O2 -march=native")` |
| 100/100 `problems/*/metadata.json` | `"c_baseline_flags": "-O2 -march=native"` |
| `scripts/run_crosslang.py` line 103 | `gcc -O2 -o ... -lm` ❌ — **`-march=native` 누락** |

→ `run_crosslang.py`의 C baseline이 README/metadata 정책 위반. crosslang 실험 결과의 BMB vs C 비교가 잘못된 C 베이스라인을 사용.

영향: BMB vs C 비교 (특히 vectorizable 코드)에서 C가 더 느려 보이는 결과 — BMB 측에 유리한 systematic bias.

### Fix

```python
# Before
r = subprocess.run(["gcc", "-O2", "-o", str(out), str(src), "-lm"], ...)

# After
# README baseline policy: -O2 -march=native (matches registry.py default + 100/100 problem metadata)
r = subprocess.run(["gcc", "-O2", "-march=native", "-o", str(out), str(src), "-lm"], ...)
```

`-march=native`만 추가. 다른 옵션 (`-lm`, capture, timeout) 그대로 유지.

### ecosystem README grep 결과

다른 ecosystem 패키지 (action-bmb / bmb-labs / benchmark-bmb / bmb-compute / bmb-crypto / bmb-json / bmb-mcp): "Nx faster" 마케팅 claim 없음. **bmb-algo만 unique**.

→ Cycle 2741의 bmb-algo discrepancy (90x headline vs 6.3x table)는 단발성 case. 다른 패키지로 같은 패턴 전파 없음.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Python syntax check (`py_compile`) | ✅ OK |
| `pytest` ai-bench 전체 | ✅ 30/30 PASS |
| 정책 정합성 (README + registry + metadata + script) | ✅ 4/4 통일 |
| bench 진행 영향 | ✅ 없음 (Python only) |

결함: 없음.

## Reflection

### crosslang systematic bias 발견 가치

이 결함은 B-track methodology에 직접 영향. 2026-03-26 crosslang 실험 결과 (BMB 90% vs C 82% vs Python 87%)의 C 베이스라인이 **`-march=native` 누락 상태**로 측정된 가능성. → 9%p BMB > C 갭의 일부는 baseline systematic bias일 수 있음.

M4-1 baseline 실행 시 이 fix가 적용되어 정확한 비교 가능. 다음 baseline 결과는 이전 (2026-03-26) 값과 직접 비교 불가 (baseline 변경 사실 기록 필요).

### 추가 leverage 발견

Cycle 2739-2740: context-overflow 양 스크립트 + production code 정합
Cycle 2742: gcc flag README + 100 metadata vs script 정합

→ 패턴: "정책 문서화와 코드 정합 갭" — README/metadata에 표명된 정책이 코드에 일관되게 반영되지 않은 case 발견. 양식 표준화의 코드 영역 적용.

### Active 백로그 변화

| 상태 | 카운트 |
|------|-------|
| 세션 시작 (Cycle 2737) | 19 active |
| Cycle 2742 종료 | **18 active** (Cycle 2739 close 유지) |
| Closed 누적 | 41 (+1 누적) |

ISSUE 백로그 자체는 변화 없음. Cycle 2742는 ISSUE에 등록되지 않은 silent defect fix.

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  - crosslang 결과 baseline 변경 사실 기록 — 2026-03-26 결과는 잘못된 C baseline. M4-1 re-baseline 시 명시.
- Pending Human Decisions:
  - M4-1 baseline 재실행 (HUMAN): Cycle 2742 fix 적용된 baseline은 이전 (2026-03-26)과 직접 비교 불가, 변경 사실 명시 필요
- Roadmap Revisions: 없음
- Next Recommendation: **Cycle 2743** — 추가 정책-vs-코드 갭 grep (다른 베이스라인 flag, build flag 정합성) + bench 진행 확인
