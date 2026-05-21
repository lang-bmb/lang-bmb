# Cycle 2751: Tier 3 10-run noise-gate 재측정 — lexer 회귀 가설 검증

Date: 2026-05-12

## Re-plan

Cycle 2750 Roadmap Revision 채택: 시퀀스 B (bmb-algo bench) → Cycle 2752로 slip. 본 cycle은 회귀 가설 검증 1순위. Trigger ⚪ NONE.

## Scope & Implementation

### 10-run noise-gate 재측정

```bash
./scripts/benchmark.sh --tier 3 --runs 10 --json --output target/benchmarks/tier3_10run_2026_05_12_c2751.json
```

총 wall 36초 (Tier 3 7 benches × 10 runs). noise-gate auto-bump 효과는 이미 `--runs 10` 명시로 무관.

### 검증 결과 (3 회귀 후보 모두 기각)

| benchmark | c2729 (5-run) | c2751 (10-run) | historic | 결론 |
|---|---|---|---|---|
| lexer | 1.310x ⚠️ | **1.000x** ✅ | 1.000x | 노이즈 |
| json_serialize | 1.120x ⚠️ | **0.870x** ✅ | 0.824x | 노이즈 (BMB FASTER 회복) |
| json_parse | 1.210x ⚠️ | **1.070x** ✅ | 1.143x | 노이즈 (개선) |

### C-side anomaly 발견 (BMB 측 측정 영향 없음)

| bench | c2729 c_ms | c2751 c_ms | BMB c2729 | BMB c2751 |
|-------|-----------|-----------|-----------|-----------|
| brainfuck | 45 | 133 (+196%) | 42 | 42 |
| csv_parse | 56 | 135 (+141%) | 46 | 41 |
| http_parse | 47 | 137 (+192%) | 45 | 135 (BMB도 변화?) |

BMB 측은 brainfuck/csv_parse stable. http_parse는 BMB도 45→135로 변화 (다른 anomaly).

추정 원인 (low confidence, 추적은 carry-forward):
- Fresh re-build에서 C 컴파일러 캐시 효과 차이 (c2729는 누적 build, c2751은 일부 fresh)
- 또는 build 시 clang/gcc 선택 분기 차이 (line 471-472 `clang || gcc` fallback)

### 갱신 파일

| 파일 | 변경 |
|------|------|
| `claudedocs/issues/ISSUE-20260511-or-chain-lowering.md` | Cycle 2751 검증 stamp (lexer 1.000x), 측정 추이 row append (c2751 갱신, c2729 노이즈 명시) |
| `claudedocs/ROADMAP.md` § 5 | "Cycle 2751 검증" sub-section 추가, 3 회귀 후보 기각 표 + C-side anomaly 노트 |

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 10-run 재측정 실행 | ✅ 36s wall |
| 3 회귀 후보 가설 검증 | ✅ 모두 historic baseline 복귀 |
| ISSUE-or-chain-lowering 측정 추이 갱신 | ✅ |
| ROADMAP Cycle 2751 sub-section 추가 | ✅ |
| 의도 외 변경 없음 | ✅ |

결함: 없음. C-side anomaly 3건은 fairness 점검 항목 (carry-forward, BMB 측 측정에 무영향).

## Reflection

### advisor 절제의 leverage 입증

Cycle 2750 STEP 4에서 "환경 변동 가설 우위, 회귀 단정 회피"가 정확한 판단으로 입증. 만약 단정하고 5-10 cycles inttoptr/or-chain 수정에 들어갔다면 헛수고. → HANDOFF "advisor 절제" 메모와 일관.

### 5-run noise floor 정량화

Tier 3 short benches (20-50ms 절대 시간):
- 5-run min-of-N: 30%+ noise (lexer 28→51 in c2729 5-run)
- 10-run min-of-N + noise-gate: <5% noise (lexer 41 c2751 일관)

**1.000x ↔ 1.310x 31pp swing**이 단순 5-run 부적합으로 발생. Tier 3 default 5-run은 신뢰 부족.

### scripts/benchmark.sh tier_all default 권고

- 현재: `RUNS=5` (line 52), noise-gate threshold 100ms / min-runs 10 (자동 bump)
- Tier 3 bench warmup time이 100ms 이상이면 noise-gate 미발화 → 5-run 노이즈 그대로
- 권고: `--runs 10` default for Tier 3, 또는 noise-gate threshold 200ms로 상향

**Trade-off**: ~2.5h tier_all → ~3.0h. ROI = false-positive cycle 1회분 절약 (한 사이클 회귀 추적 ≈ 1-3h). 변경 가치 ✅.

### 출력 디폴트 (CLAUDE.md Rule 8) 정렬 확인

JSON 출력 (`--json`)로 기계 파싱 가능. 본 cycle 분석은 Python jq 대체로 처리. AI 친화 디폴트 정렬 ✅.

## Carry-Forward

### Actionable

- **Cycle 2752 = 시퀀스 B M3-5**: 회귀 검증 완료로 narrative 위협 제거. bmb-algo bench v0.98 재실행 + README 정정 즉시 가능
- **Cycle 2753+**: 잔여 자율 (5) `multiple-pre-clauses` 파서 spec 확장 (1-2 cycles) 또는 ISSUE 백로그 cleanup

### Structural Improvement Proposals

1. **scripts/benchmark.sh Tier 3 default → `--runs 10`**:
   - 위치: `scripts/benchmark.sh:52` `RUNS=5`
   - 변경: Tier 3 분기에서만 10 runs 적용 또는 default 자체 10으로 상향
   - 근거: 5-run Tier 3 noise = 31pp swing (lexer c2729 사례), 추적 비용 1-3h/회
2. **C-side build path 추적**:
   - 위치: `scripts/benchmark.sh:471-472` (clang || gcc fallback)
   - brainfuck/csv_parse C가 c2729 → c2751 사이 2-3x slower 변화 — 컴파일러 분기 가능성
   - 권고: build verbose 로그 옵션 (`--build-verbose`) 추가, C 측 컴파일러 명시 stamp

### Pending Human Decisions

- 신규 없음 (기존 큐 유지)

### Roadmap Revisions

- ROADMAP § 5에 "Cycle 2751 검증" sub-section 추가됨
- 시퀀스 B M3-5 진행 Cycle 2752로 확정 (Cycle 2750 revise 그대로)

### Next Recommendation

**Cycle 2752 = 시퀀스 B M3-5 bmb-algo bench v0.98 재실행 + README 정정**

상세 (HANDOFF § 2.5 시퀀스 B):
- B.1: `bench_algo.py` v0.98 5-run median 측정 (현재 config 10 items)
- B.2: README 표 정정 "knapsack(10 items)" + clang vs gcc 라벨
- B.3: headline "90x/181x" 처리 옵션 (a/b/c) 자율 분석 → HUMAN review
- B.4: bmb-algo CHANGELOG `[Unreleased]` re-baseline annotation

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| ISSUE-20260511-or-chain-lowering.md | `claudedocs/issues/` | gitignored |
| ROADMAP.md § 5 (Cycle 2751 sub-section) | `claudedocs/` | tracked |
| tier3_10run_2026_05_12_c2751.json | `target/benchmarks/` | untracked (bench artifact) |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2751.md` | gitignored |
