# Cycle 2724: fasta StringBuilder ISSUE 재진단 + Tier 1 bulk re-measurement pivot
Date: 2026-05-11

## Re-plan
인계 (Cycle 2723): StringBuilder SSO 분석. Trigger 🟠 RE-PLAN (advisor 적용).

**핵심 발견**: 3개 P-track ISSUE 연속 false positive 패턴. ISSUE 데이터가 v0.51.22 (2026-04-13, 1년 stale). 개별 deep dive 대신 **bulk re-measurement**로 pivot.

## Scope & Implementation

### fasta ISSUE 진단 (예정 task)

`ecosystem/benchmark-bmb/benches/compute/fasta/bmb/main.bmb` 검토:

| 항목 | 사용 방식 |
|------|----------|
| 문자열 빌딩 | ❌ StringBuilder 미사용 |
| 라인 버퍼 | `malloc(61)` + `store_u8` (raw byte buffer) |
| 출력 | `puts_cstr(line)` |
| nucleotide 선택 | `select_iub_code` if-else 체인 14 비교 |

C 버전 동일 패턴 (`char line[LINE_WIDTH+1]` + `puts`). → ISSUE 원진단 "StringBuilder 병목"은 **false positive**.

### 3-연속 false-positive 패턴 인식

| Cycle | ISSUE | 결과 |
|-------|-------|------|
| 2722 | match-jump-table | brainfuck jump table 작동 (53 LJTI). lexer는 `or` lowering이 진짜 원인 |
| 2723 | (`or` chain) | proper fix multi-cycle scope, 새 ISSUE 등록 |
| 2724 | string-builder-opt | fasta가 SB 미사용. **false positive** |

→ **공통**: 측정 데이터가 v0.51.22 (2026-04-13, 약 1년 전). 그 사이 컴파일러 진화:
- M5-1~M5-5g enum/payload/multi-field/Array<f64>
- 5M token packing (Stage 2 회복)
- Builtin arity guard 30 사이트 (lint 11 + arity guard 이중 안전망)
- inproc timing infra (Cycle 2661+)

따라서 모든 P-track ISSUE는 stale 가능성 큼.

### Pivot 결정 (advisor 합의)

**Cycle 2725 재계획**: TBD → **Tier 1 bulk re-measurement**.

```bash
./scripts/benchmark.sh --tier 1 --runs 5 --output target/benchmarks/tier1_2026_05_11.json
```

산출:
- 6개 P-track ISSUE 벤치마크 (fasta/hash_table/brainfuck/lexer/sorting/binary_trees) 현재 값
- BMB ≤ 100% 이미 달성한 ISSUE → 일괄 close
- 남은 갭은 v0.98 IR 기준 재진단

### Structural Improvement (advisor 권고)

**ISSUE 측정 데이터에 stamp + stale-after threshold**:
- 측정일자 명시 필수
- 6개월 이상 측정 → "stale" 경고
- ISSUE 양식 표준화 다음 세션 작업

### 작업

1. ✅ fasta ISSUE close (header 갱신 + closed/ 이동)
2. ✅ ROADMAP.md Cycle 2725 pivot 기록
3. ⏳ Cycle 2725 실제 재측정은 다음 cycle

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| fasta BMB 코드에서 SB 사용 검색 | ✅ 0건 (malloc/store_u8/puts_cstr만) |
| fasta C 코드와 패턴 비교 | ✅ 동일 (raw buffer) |
| ISSUE close + closed/ 이동 | ✅ |
| benchmark.sh --tier 1 옵션 확인 | ✅ 존재 |
| 백그라운드 golden task 진행 확인 | ✅ ~1559 lines (절반) |
| ROADMAP Cycle 2725 pivot 기록 | ✅ |

active ISSUE 카운트: 24 (closed 32, +1 new from cycle 2723, -1 from cycle 2722, -1 from cycle 2724)

결함: 없음.

## Reflection

### 외부 관찰자 관점

1. **3 연속 false positive의 의미**: 무작위 우연이 아니라 **systematic data staleness**. ISSUE backlog의 모든 측정 claims를 재검증해야. advisor의 통찰 정확.

2. **개별 deep-dive vs bulk re-measurement 비교**:
   - 개별: 5개 ISSUE × 1 cycle = 5 cycles, 부분 결과
   - bulk: 1 cycle 재측정 → 전체 triage
   - **bulk가 5x ROI 높음**

3. **advisor 자문 시점의 정확성**: Cycle 2724 시작 시 자문 받았으면 cycle 2722-2723 deep dive 회피 가능. 그러나 패턴 인식 자체가 가치 — 3 데이터포인트 없이는 stale 가설 검증 불가.

4. **ISSUE 데이터 stamping의 부재**: 모든 P-track ISSUE에 측정일자가 있으나 stale-after threshold 없음. 다음 세션 양식 표준화.

### Roadmap impact (large)

- Cycle 2725 = **Tier 1 bulk re-measurement** (TBD 해소)
- 진척 측면: 개별 fix 시도 → 측정 인프라 활용으로 전환
- 새 Structural Improvement: ISSUE 양식에 측정 메타데이터 표준 추가

## Carry-Forward

- Actionable (Cycle 2725): **Tier 1 benchmark bulk re-measurement** — `./scripts/benchmark.sh --tier 1 --runs 5 --output target/benchmarks/tier1_2026_05_11.json`
- Structural Improvement Proposals:
  - **ISSUE 양식 표준화**: 측정일자 + stale-after threshold (예: 6개월) — 다음 세션
  - **`or`/`and` lowering proper fix phase** (Cycle 2723 carry-forward 유지) — multi-cycle next session
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: Cycle 2725 TBD → Tier 1 bulk re-measurement (advisor RE-PLAN, ROADMAP.md 적용 완료)
- Next Recommendation: Cycle 2725 = `benchmark.sh --tier 1` 재측정 + P-track ISSUE 일괄 triage
