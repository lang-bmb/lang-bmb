# Cycle 2660: nqueen 정식 측정 + clang -O3 baseline 추가
Date: 2026-05-11

## Re-plan
Cycle 2659 carry-forward: nqueens benchmark 추가 (1 cycle, 자율).
**HANDOFF 진단 부정확 발견**: nqueen 벤치는 이미 `ecosystem/benchmark-bmb/benches/compute/nqueen/` (s 빠짐) 에 존재.
SCOPE ADJUST: "벤치 추가" → "기존 nqueen 정식 측정"으로 조정.

## Scope & Implementation

### 1. nqueen BMB vs C 측정 (Cycle 2660 핵심)
- BMB v0.98 release build → 829ms (median 5-run)
- gcc -O2 -march=native → 8360ms
- gcc -O3 -march=native -flto → 6440ms
- clang -O3 -march=native → 847ms

### 2. 측정 정직성: clang baseline 추가 (5개 알고리즘)
- knapsack/floyd_warshall/sieve/fibonacci/nqueen 모두 clang -O3로 추가 빌드
- → BMB ≈ clang -O3 (4/5 ≤1.05x, 5/5 ≤1.5x)
- knapsack은 clang 기준 7.9x faster, gcc 기준 1.22x slower (역전)

### 3. M3-2-bench-results.md 갱신
- nqueen 신규 행 추가
- clang -O3 baseline 컬럼 추가
- gcc 비교는 보충 표로 보존
- README 주장 분석 추가 (gcc 미재현 / clang 재현)

### 4. BMB IR 분석으로 우위 메커니즘 확인
- `bmb_user_main`이 `count_solutions(0,0,0,0,32767)` 1회 호출 + `mul i64 %_t29.i.i, 10`
- → `nqueens` 함수의 결정성을 LLVM이 추론, 10x 호출 축약
- 동일 패턴: fibonacci 1000x faster의 메커니즘과 동일 (자동 @pure 추론)

## Verification & Defect Resolution

**측정 결과 일관성**: BMB와 모든 C 변형이 동일한 결과 (`22791840`) 출력 ✅
**테스트 영향**: 본 사이클은 측정만, 컴파일러 변경 없음 → cargo test 재실행 불필요
**문서 일관성**: M3-2-bench-results.md ↔ HANDOFF/ROADMAP cross-check 필요 (다음 사이클)

## Reflection

**Scope fit**:
- 의도 = nqueens 측정 → 달성 ✅
- 추가 발견 = HANDOFF 부정확 (벤치 부재 → 실제 존재) — 즉시 SCOPE ADJUST로 처리

**Latent defects**:
- M3-2-bench-results.md의 이전 ITERS=500 결과 vs 새 ITERS=10 결과 충돌 명시 필요 — 기록 완료
- 측정 노이즈: fibonacci 4.8ms는 startup-dominated → in-process timing (Cycle 2661+) 정당화

**Structural improvement opportunities**:
- M3-2 측정 결과를 자동 회귀 가드로 통합 가능 (현재는 ad-hoc)
- benchmark-bmb suite의 디렉토리 명명 일관성 (nqueen vs nqueens) — 기존 그대로 유지 (오류 아님)

**Philosophy drift 점검**:
- "Performance > Everything" 가설 = clang -O3 동등성으로 부분 검증 ✅
- 정직성: gcc 비교에서 BMB가 더 빠른 경우(gcc 한계)도 명시 ✅
- README "6.8x faster than C" 검증 = 측정 조건(clang vs gcc) 명시로 정량화 ✅

**Roadmap impact**:
- M3-2 자율 부분 ✅ 완료 (7/7 측정, 5/5 clang ≤1.5x)
- M3 ~93% → ~95% (HUMAN publish 잔여)
- README 주장 검증 결과 = HUMAN 결정 input 명확화

**User-facing quality**:
- M3-2-bench-results.md 측정 조건 / baseline / verdict 모두 명시 — 외부 reader 직접 검증 가능
- 표 가독성: gcc 비교 / clang 비교 분리로 정보 손실 없이 명확화

## Carry-Forward
- Actionable: 없음 (이번 사이클 자율 작업 완전 종료)
- Structural Improvement Proposals:
  - in-process `time_ns()` timing (Cycle 2661+): startup overhead 제거
  - benchmark suite 자동 회귀 가드 (CI integration)
  - bmb-algo README 갱신 (HUMAN 결정 후)
- Pending Human Decisions:
  - bmb-algo README "6.8x faster" 검증 결정 (clang baseline 명시 권장)
  - npm/PyPI publish (M3 publish 단계)
- Roadmap Revisions: M3-2 7/7 완료 — ROADMAP M3 줄에서 "nqueens 부재" 표시 제거 예정 (다음 사이클)
- Next Recommendation: Cycle 2661 — in-process timing 인프라 설계 (BMB 내장 `time_ns()` API + 벤치 harness 패턴)
