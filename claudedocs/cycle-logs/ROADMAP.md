# Cycle-Logs 방향성 로드맵
> 최종 업데이트: 2026-05-20 (Cycle 2991 — ISSUE triage + 방향성 재정비)
> 이 파일은 **방향성 앵커**다 — 각 사이클 Derive-Next에서 수정 가능.
> 실무 앵커: `claudedocs/ROADMAP.md`

## 현재 상태 (Cycle 2991 기준)

- HEAD: `474f2d04`
- B-axis: Claude 98.0% (고정 베이스라인) / GPUStack 99.7% (299/300, 공식)
- P-track: 16/16 ≤1.05x ✅
- cargo test --release: 6260 tests, 0 failed ✅
- 3-Stage Fixed Point: ✅ (Cycle 2930 확인)
- Active ISSUEs: 6개 (4 stale methodology + 2 P3 technical)

## Cycles 2991-3000 방향성

### 우선순위 (계층 순)

1. **ISSUE triage** (Cycles 2991-2993) — 4개 stale ISSUE-20260326 현황 갱신
   - multi-model-validation: GPUStack = 2번째 모델 → PARTIALLY RESOLVED
   - external-problem-validation: bench verify 17/17 PASS → AC1 충족
   - integration-category-weakness: B-axis 해소됨 → 이미 PARTIALLY RESOLVED
   - problem-difficulty-bias: 변화 없음 → OPEN 유지

2. **P3 ISSUE 분석** (Cycle 2994) — clang-knapsack-outlier 라벨 명확화
   - BMB 6.7x 빠름 = BMB 개선 불필요, README 라벨만 추가
   - golden-flakiness-inttoptr: HUMAN 결정 대기, 상태 확인

3. **problem.md 품질** (Cycles 2995-2996) — multi-shot 패턴 추가 탐색
   - 04_fibonacci는 CRITICAL 노트 추가 완료
   - 다른 일관 2-shot 패턴 grep으로 탐색

4. **세션 정리** (Cycle 2997) — HANDOFF/ROADMAP 갱신 + commit

5. **조기 종료 대비** (Cycles 2998-3000) — 잔여 범위가 HUMAN-blocked이면 종료

### 조기 종료 조건

- Cycle 2997 이후 자율 작업이 소진되면 종료
- HARD STOP 없이 자연 종료 (모든 잔여 항목이 HUMAN-blocked)

## 알려진 HUMAN-blocked 항목 (자율 사이클 범위 밖)

- GPT-4o 실험 (multi-model-validation AC2)
- npm/PyPI publish (M3 잔여)
- golden-flakiness-inttoptr 옵션 결정 (A/B/C)
- problem-difficulty-bias: 20개 신규 hard 문제 추가

## 이전 세션 성과 (2026-05-20, Cycles 2981-2990)

- else-if 세미콜론 규칙 발견 (BMB 언어 특성)
- 13개 problem.md 수정 (예방적 포함)
- GPUStack 99.7% 공식 달성 (3-run 299/300)
- 6260 tests ✅

## RE-PLAN 트리거 (다음 사이클용)

- ISSUE triage에서 자율 범위 항목 발견 시 → SCOPE ADJUST
- problem.md audit에서 새 언어 패턴 발견 시 → RE-PLAN

## 종료 조건

- 자율 잔여 작업이 소진되었거나
- HARD STOP (철학 위반 / 아키텍처 무효화) 발생 시 조기 종료
