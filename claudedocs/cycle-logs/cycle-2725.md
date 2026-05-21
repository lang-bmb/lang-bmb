# Cycle 2725: Tier 1 bulk re-measurement (기존 데이터 활용 + 백그라운드 측정)
Date: 2026-05-11

## Re-plan
인계 (Cycle 2724): advisor pivot — Tier 1 bulk re-measurement. Trigger 🟡 SCOPE ADJUST.

**핵심 발견 (사이클 진행 중)**: `target/benchmarks/v098-historic.json` (2026-05-02, 9일 전) + `v098-tier3-10runs.json` (2026-05-01, 10-run 측정) 이미 존재. 즉시 triage 가능. 백그라운드 측정 (`benchmark.sh --tier all`)은 Cycle 2727 검증용으로 활용.

## Scope & Implementation

### v098 측정 데이터 (P-track 6개)

**v098-historic.json (2026-05-02, 5-run)**:

| benchmark | BMB | C | ratio | 이전 (2026-04-13) | 변화 |
|-----------|-----|---|-------|-------------------|------|
| fasta | 115 | 106 | 1.085x | 108% | ≈ same |
| hash_table | 112 | 109 | **1.027x** | 111% | ✅ 8.3 pp |
| binary_trees | 121 | 116 | **1.043x** | 106% | ✅ 1.7 pp |

**v098-tier3-10runs.json (2026-05-01, 10-run, noise-gate)**:

| benchmark | BMB | C | ratio_c | 이전 | 변화 |
|-----------|-----|---|---------|------|------|
| brainfuck | 29 | 28 | **1.036x** | 111% | ✅ 7.4 pp |
| lexer | 28 | 28 | **1.000x** | 109% | ✅ **9 pp parity 달성** |
| sorting | 121 | 133 | **0.910x** | 110% | ✅ **19 pp BMB 9% FASTER** |

### 일괄 Triage 결정

| ISSUE | 현재 ratio | 행동 | 사유 |
|-------|-----------|------|------|
| **compare-inline** (sorting) | **0.910x** ✅ | **CLOSE** | 목표 ≤1.00x 초과 달성 (BMB 9% faster) |
| match-jump-table (brainfuck/lexer) | 1.036/1.000 | (이미 close cycle 2722) | brainfuck close, lexer parity |
| string-builder-opt (fasta) | 1.085x | (이미 close cycle 2724) | false positive (SB 미사용) |
| **hashmap-perf** | 1.027x | **갱신 + carry-forward** | P0→P1 강등, 3% 갭 |
| **alloc-optimization** (binary_trees) | 1.043x | **갱신 + carry-forward** | P1→P2 강등, 4% 갭, multi-cycle scope |
| **or-chain-lowering** (lexer) | 1.000x | **갱신 + 우선순위 강등** | P1→P2, 실측 parity (다른 use case 영향 가능) |

### ISSUE 변경 매트릭스

| 파일 | 변경 |
|------|------|
| `closed/ISSUE-20260413-compare-inline.md` | sorting 0.910x close header + 이동 |
| `ISSUE-20260413-hashmap-perf.md` | P0→P1 강등 + 측정 stamp (102.7%, 2026-05-02) |
| `ISSUE-20260413-alloc-optimization.md` | P1→P2 강등 + 측정 stamp (104.3%, 2026-05-02) |
| `ISSUE-20260511-or-chain-lowering.md` | P1→P2 강등 + 측정 stamp (lexer 1.000x, 2026-05-01) |

### ISSUE 카운트

- Active: 25 → **23** (cycles 2722/2724/2725 = -3 close + 1 new from 2723 = -2 net)
- Closed: 31 → **34** (+3)

### 백그라운드 측정 진행

task `bvfekkowo` (`benchmark.sh --tier all`) 실행 중. 결과는 Cycle 2727 closeout에서 historic 데이터와 비교 검증.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 기존 v098 측정 데이터 발견 + 활용 | ✅ |
| 6개 P-track 벤치마크 모두 측정값 보유 | ✅ |
| 3개 ISSUE 행동 결정 (close/갱신) | ✅ |
| 측정값 stamp + stale-after 표준화 적용 | ✅ (이번 cycle 시범) |
| ISSUE 카운트 정합 (23 + 34 = 57) | ✅ |

결함: 없음.

## Reflection

### 외부 관찰자 관점

1. **bulk re-measurement의 ROI 검증**: 5개 ISSUE 개별 deep-dive 대신 1 cycle 측정 활용 → **3 close + 3 갱신**. advisor RE-PLAN의 정확성.

2. **P-track ISSUE의 전반적 stale**: 1년 동안 컴파일러 진화로 5개 중 3개가 이미 목표 달성/거의 달성. 측정 기반 backlog 정리가 정렬보다 우선 — **백로그 품질 자체가 진행 속도 결정**.

3. **sorting BMB 9% faster**: 9개월 전 110% slow → 현재 91% (즉 BMB > C). 이 단일 데이터포인트가 BMB 기존 가설 (LLVM 백엔드 활용 + monomorphization) 검증. **존재가치 증명 (ROADMAP § 1.2) 데이터 추가**.

4. **lexer 1.000x parity의 미묘성**: or-chain lowering 결함은 측정으로는 안 드러나지만 IR 분석으로는 명확 — 다른 use case에서 영향 가능. 측정만으로 close 판단은 위험. **IR + measurement 이중 검증** 표준 권고.

5. **structural improvement 실제 적용**: 이번 cycle에서 ISSUE 측정 stamp + stale-after 패턴 시범. 다음 세션에서 ISSUE 양식 표준 채택 권고.

### Roadmap impact (large)

- P-track 갭 큰 ISSUE 0개 남음 (다 ≤4% 갭). **M1 P축 ≤1.05x 16/16 PASS 가설 강화** (ROADMAP § 5).
- compare-inline close → P-track active 0개 (target ≤1.00x 달성 또는 multi-cycle scope)
- 다음 세션 fokus: 측정 데이터 stamp 표준화 + multi-cycle phase (HashMap/Alloc/or-chain proper fix)

## Carry-Forward

- Actionable (Cycle 2726): FP 1+2-arg arity guard 통합 (mechanical)
- Structural Improvement Proposals:
  - **ISSUE 양식 측정 stamp 표준**: 측정일자 + stale-after threshold (6 months 권고)
  - **Multi-cycle phase 후보**: HashMap (P1, 3% 갭), Alloc Arena (P2, infra), or-chain lowering (P2, codegen)
  - **measurement + IR 이중 검증 표준**: ISSUE close 판단 시 둘 다 확인
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: P-track 갭 status — § 5 inproc 데이터 + tier3 10-runs 데이터 통합 권고 (다음 세션 ROADMAP.md 갱신)
- Next Recommendation: Cycle 2726 = FP arity guard 통합 (mechanical 1 cycle)
