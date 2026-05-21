# Cycle 2720: CI 게이트 — Golden Sample 50 (시퀀스 D)
Date: 2026-05-11

## Re-plan
인계 (Cycle 2719): CI 게이트 추가 (bootstrap_3stage.sh + golden sample 50). Trigger ⚪ NONE.

## Scope & Implementation

### 사전 발견
- `bootstrap-benchmark.yml`에 **3-Stage Bootstrap + Fixed Point 검증은 이미 존재** (`needs.bootstrap.outputs.fixed_point`). Cycle 2505 이후 honest-fail 정렬.
- `ci.yml`에 `bootstrap-check` job 있으나 type-check만, 골든 binary 테스트 없음.
- Golden 테스트는 풀 manifest만 — sample 옵션 미존재.

### 변경 매트릭스

| 파일 | 변경 | LOC |
|------|------|-----|
| `scripts/run-golden-tests.sh` | `--limit N` 옵션 + while 루프 break 가드 | +9 |
| `.github/workflows/bootstrap-benchmark.yml` | bootstrap job 내 "Run Golden Sample 50" step + artifact upload 갱신 | +12, -1 |

### 설계 결정

**왜 별도 job이 아닌 step 통합?**: bootstrap job이 이미 `target/bootstrap/bmb-stage1` 빌드 보유. 별도 job은 stage1 재빌드 (60s+) 비용 발생. step 통합 = 0초 추가 비용 + 50 tests = ~40s. PR latency 영향 최소.

**왜 --limit 50?**: 풀 2862 = 43분 (Cycle 2701). 50개 = ~40s. CI bound <1분.

**Manifest 순서**: alphabetical. 첫 50 deterministic = `a_star_path`, `aabb_tree`, `abs_diff`, ... — manifest 끝 부분 변경 catch 못함. **약점 인정** → carry-forward로 nightly 풀 골든.

### Smoke test

```
./scripts/run-golden-tests.sh --limit 5 --json
{"type":"golden_tests","passed":5,"failed":0,"total":5,"elapsed_ms":3757}
```

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `--limit 5` 로컬 실행 | ✅ 5/5 PASS, 3.7s |
| `bootstrap.sh --stage1-only` 존재 | ✅ L17/L58/L282 |
| workflow yaml 구조 (step 순서) | ✅ 3-Stage → Verify → Golden 50 → Upload |
| summary needs 조정 | ✅ test/bootstrap/benchmark (golden은 bootstrap 통합) |

결함: 없음.

## Reflection

### 외부 관찰자 관점

1. **이미 존재하는 CI 안전망 발견의 가치**: Cycle 2716 triage에서 "CI 게이트 미구축"이라 적었으나 실제로는 3-Stage Bootstrap 부분은 이미 honest-fail (Cycle 2505). triage 보고서가 부정확했음 — **백로그 항목 검증 의무** 교훈.

2. **커버리지 vs latency 균형**: 첫 50 deterministic = 1.7% 커버리지. PR latency가 우선시되는 환경에서 옳은 trade-off. 풀 검증은 nightly로 분리.

3. **잠재 결함**: 첫 50 alphabetical은 manifest 끝의 신규 테스트 변경 catch 못함. 다음 사이클에서 representative 50 manifest 큐레이션 권고.

### Roadmap impact
없음.

## Carry-Forward
- Actionable (Cycle 2721): FP 1-arg arity guard 확장 (`@fabs/floor/ceil/round/sqrt/sin/cos/exp/log`)
- Structural Improvement Proposals:
  - **nightly-bench.yml에 풀 골든 추가**: PR-time bound 유지 + nightly full coverage
  - **Representative sample 50 manifest** (`golden_tests_sample.txt`): manifest 끝 추가 catch
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2721 = FP builtin 1-arg arity guard (i64 패턴 mechanical 확장)
