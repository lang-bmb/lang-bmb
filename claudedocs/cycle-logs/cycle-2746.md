# Cycle 2746: 시퀀스 E 자율 실행 — CI -march=native spec parity PR draft

Date: 2026-05-11

## Re-plan

인계: HANDOFF (Cycle 2745) § 2.5 시퀀스 A-E. 백그라운드 tier all bench(Cycle 2729 시작) 아직 진행 중 (마지막 출력 ~1분 전).

자율 선택:
- 시퀀스 A: bench 미완료 → carry-forward
- 시퀀스 B: bmb-algo bench 재실행 → system load 충돌 회피 (golden-flakiness ISSUE에서 system load 종속 확정)
- 시퀀스 C: HUMAN dispatch 필요 → skip
- 시퀀스 D: HUMAN setup 필요 (`BMB_BENCH_API_KEY`) → skip
- **시퀀스 E**: 자율 가능, bench와 진짜 독립 → **실행**

advisor 1회 호출로 시퀀스 E 진행 점검: ① M3-6 채택 결정 재확인 (✅ Cycle 2745) ② push 금지 (commit only) ③ PR body는 파일로 저장 ④ Cycle 2743 5위치 매핑 정확히 활용.

## Scope & Implementation

### 서브모듈 branch 생성

`ecosystem/benchmark-bmb` 서브모듈 `main` → `fix/march-native-spec-parity` checkout.
Untracked mandelbrot inproc 변환 파일 2건은 추적 외 (별개 진행 중 작업) — 손대지 않음.

### 5 위치 일괄 수정 (+5 / -5)

Cycle 2743 매핑 활용:

| 파일 | 라인 | 컴파일러 | Before | After |
|------|------|----------|--------|-------|
| `.github/workflows/benchmark-multiplatform.yml` | 96 | clang Unix | `clang -O3` | `clang -O3 -march=native` |
| `.github/workflows/benchmark-multiplatform.yml` | 132 | gcc MinGW Win | `gcc -O3` | `gcc -O3 -march=native` |
| `.github/workflows/benchmark.yml` | 71 | clang | `clang -O3` | `clang -O3 -march=native` |
| `.github/workflows/benchmark.yml` | 118 | clang | `clang -O3` | `clang -O3 -march=native` |
| `Dockerfile` | 71 | clang | `clang -O3` | `clang -O3 -march=native` |

`git diff --stat` 결과: 3 files changed, 5 insertions(+), 5 deletions(-). 의도 외 변경 없음.

### PR body draft 파일

`claudedocs/pr-draft-march-native.md` — HUMAN이 `gh pr create --body-file`로 활용 가능한 형식. 명시:
- 5 위치 매핑 표
- "baseline change" 경고 (CI history discontinuity)
- spec 근거 (`BENCHMARK_MASTERPLAN.md:135`)
- post-merge actions (CI run 새 baseline stamp + 부모 repo submodule pointer bump)
- HUMAN dispatch 명령 예시 (push + `gh pr create --draft`)

### 서브모듈 commit (push 금지)

```
cb478d2 fix(ci): -march=native spec parity in workflows + Dockerfile
```

HEREDOC commit message (8줄 본문). HEAD `cb478d2` on `fix/march-native-spec-parity` (서브모듈 internal). 부모 repo는 submodule pointer 변경 인지만 (commit 안 함, HUMAN merge 후 그쪽에서 bump).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 채택 결정 M3-6 확인 (Cycle 2745) | ✅ |
| 5 위치 정확히 수정 | ✅ (`git diff --stat` 3 files, +5/-5) |
| 의도 외 변경 없음 | ✅ |
| PR body draft 파일 저장 | ✅ `claudedocs/pr-draft-march-native.md` |
| 서브모듈 commit | ✅ `cb478d2` on `fix/march-native-spec-parity` |
| Push 회피 (HUMAN trigger) | ✅ |
| 부모 repo submodule pointer 미변경 | ✅ (working tree만 update, commit 안 함) |

결함: 없음.

## Reflection

### 시퀀스 E 자율 실행의 leverage

HANDOFF가 5 위치 정확히 명시했고 cycle 2743 로그가 라인 단위 매핑 보유. → 1 cycle 내 완결.

대조: 5 위치 자체를 찾는 데서 시작했다면 (별도 grep + 분류) 2 cycle 이상 소요. 직전 세션의 cycle 2743 audit 결과물이 핵심 leverage.

### advisor 절제

질문 4개 점검만으로 충분 (M3-6 채택 / push 금지 / PR body file / Cycle 2743 매핑 활용). 추가 호출 불요. advisor 가치는 결정 전 1회.

### 백그라운드 bench 모니터링 정책

`Monitor` 도구 사용 없이 시퀀스 E 완료 후 1회 확인 → 진행 중 → carry-forward. polling 회피.

## Carry-Forward

### Actionable

- **시퀀스 A**: 백그라운드 bench JSON (`target/benchmarks/tier_all_2026_05_11_c2729.json`) 출현 시 P-track ISSUE 측정 stamp 갱신 (hashmap-perf / alloc-optimization / or-chain-lowering). 다음 세션 첫 cycle 또는 mid-session check.
- **시퀀스 B**: bmb-algo bench 재실행 + README 정정 (M3-5). bench 종료 후 진행 권고 (system load 충돌 회피).
- **시퀀스 E.4 HUMAN trigger**: `git push -u origin fix/march-native-spec-parity` (서브모듈) + `gh pr create --draft --body-file claudedocs/pr-draft-march-native.md`.

### Structural Improvement Proposals

없음.

### Pending Human Decisions

이번 cycle에서 새로 추가 없음. 기존 큐 그대로:
- M3-3 / M3-4 publish dispatch (M3-5 정정 후)
- M3-5 headline 옵션 (a/b/c) review
- M3-6 PR review/merge (이 cycle 산출물)
- M4-1 `BMB_BENCH_API_KEY` 설정 + 실행

### Next Recommendation

- **다음 cycle**: bench 완료 시 시퀀스 A.2 (P-track ISSUE stamp 갱신) — 자율
- **분기 1**: bench 미완료라면 시퀀스 B 진행 회피 → 다른 자율 작업 (e.g., 잔여 자율 (5) `multiple-pre-clauses` 파서 spec 확장)
- **분기 2**: bench 완료 + B.1 v0.98 재측정 → bmb-algo README 정정 → B.3 headline 옵션 (a/b/c) 자율 분석 후 HUMAN review

## Files

| 변경 | 위치 |
|------|------|
| 서브모듈 commit `cb478d2` | `ecosystem/benchmark-bmb` (push 보류) |
| PR body draft | `claudedocs/pr-draft-march-native.md` |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2746.md` |
| 부모 repo 변경 | 없음 (commit 보류, 서브모듈 pointer는 HUMAN merge 후 bump) |
