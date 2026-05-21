# Cycle 2743: CI workflow gcc/clang flag 정합성 갭 발견 (HUMAN 결정 큐)

Date: 2026-05-11

## Re-plan

인계: Cycle 2742 carry-forward "추가 정책-vs-코드 갭 grep". Trigger: ⚪ NONE.

## Scope & Implementation

### 추가 갭 grep — CI workflow

`ecosystem/benchmark-bmb/` grep 결과:

| 위치 | 명령 | `-march=native` |
|------|------|----------------|
| `.github/workflows/benchmark-multiplatform.yml:94` (gcc Unix) | `gcc -O3 -march=native` | ✅ |
| `.github/workflows/benchmark-multiplatform.yml:96` (clang Unix) | `clang -O3` | ❌ |
| `.github/workflows/benchmark-multiplatform.yml:132` (MinGW Win) | `gcc -O3` | ❌ |
| `.github/workflows/benchmark.yml:71` (clang) | `clang -O3` | ❌ |
| `.github/workflows/benchmark.yml:118` (clang) | `clang -O3` | ❌ |
| `Dockerfile:64` (gcc) | `gcc -O3 -march=native` | ✅ |
| `Dockerfile:71` (clang) | `clang -O3` | ❌ |
| `run_benchmarks.sh:77` (clang) | `clang -O3 -march=native` | ✅ |
| `run_wsl_bench.sh:30,45` (clang) | `clang -O3 -march=native` | ✅ |
| `docs/BENCHMARK_MASTERPLAN.md:135` (spec) | `clang 18.x (-O3 -march=native)` | spec ✅ |

→ **명시 spec은 `-march=native` 사용**. 그러나 CI workflow + Dockerfile clang은 일부 누락.

### 일관성 분석

| 분류 | 정합 |
|------|------|
| spec 문서 (`BENCHMARK_MASTERPLAN.md`) | `-march=native` 명시 |
| 로컬 스크립트 (`run_benchmarks.sh`, `run_wsl_bench.sh`) | `-march=native` 적용 |
| Docker gcc (`Dockerfile:64`) | `-march=native` 적용 |
| **CI workflow clang (3 위치)** | **누락** ❌ |
| **CI workflow MinGW gcc (Win)** | **누락** ❌ |
| **Docker clang (`Dockerfile:71`)** | **누락** ❌ |

5개 CI/Docker 위치에서 `-march=native` 누락. spec과 불일치.

### Fix 결정 — defer to HUMAN

CI workflow + Dockerfile 변경의 blast radius:
- CI 측정 결과가 nominal하게 변동 (BMB vs C ratio 시프트 — 약간 더 BMB 불리 또는 동등 방향)
- 이전 CI 결과 (이전 commit hash 기준 자동 비교)와 비교 불가
- benchmark-bmb 패키지 외부 사용자가 CI history 참조 시 단절

→ 자율 변경 회피. ROADMAP HUMAN-Decisions 큐에 기록.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| spec과 코드 정합 매핑 완료 | ✅ 10 위치 분류 |
| 일관성 갭 5개 식별 | ✅ |
| HUMAN 결정 큐 등록 권고 | ✅ (cycle log) |

결함: 5개 CI/Docker 일관성 갭 (자율 fix 회피, HUMAN 결정).

## Reflection

### policy-vs-code 갭 패턴 누적

| Cycle | 갭 위치 | 결정 |
|-------|--------|------|
| 2739 | scripts/run_experiment.py 누락 truncation | ✅ 자율 fix (low-risk Python) |
| 2740 | bmb_ai_bench/run_cmd.py 누락 truncation | ✅ 자율 fix (테스트 보호) |
| 2742 | scripts/run_crosslang.py -march=native 누락 | ✅ 자율 fix (low-risk Python) |
| **2743** | **CI workflow + Dockerfile -march=native 누락 5위치** | ❌ **자율 fix 회피** (CI blast radius) |

Risk-by-blast-radius 기준 적용:
- Python script (local) → 자율
- Production library code (테스트 보호) → 자율
- CI workflow / Docker image (외부 영향) → HUMAN

### 추가 leverage limit

policy-vs-code 갭 발견은 leverage 풍부했지만 CI 영역에서 limit 도달. 다음 leverage 영역:
- HUMAN 결정 후 일괄 적용 (3 위치 다른 stakeholder 결정)
- 새 patterns (e.g., test fixture stale, doc reference rot)

## Carry-Forward

- Actionable: 없음 (자율 자체 완결)
- Structural Improvement Proposals: 없음
- Pending Human Decisions:
  - **NEW**: CI workflow + Dockerfile clang/MinGW gcc 5위치 `-march=native` 추가 — spec 정합. HUMAN 결정 필요 (CI history 단절 vs spec 정합)
- Roadmap Revisions: 없음 (HUMAN decisions 큐만 확장)
- Next Recommendation: **Cycle 2744** — 세션 wrap (HANDOFF 갱신 + cycle summary + commit prep). 추가 cycle 진행 시 saturation 확정.
