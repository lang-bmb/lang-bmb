# Cycle 2741: closed ISSUE 갱신 + bmb-algo README 불일치 발견 (HUMAN 결정 큐)

Date: 2026-05-11

## Re-plan

인계 (Cycle 2740): "closed ISSUE 본문 갱신 + 백그라운드 bench 상태 확인 + 자율 발굴". Trigger: ⚪ NONE.

## Scope & Implementation

### closed ISSUE 갱신

`closed/ISSUE-20260326-context-overflow-prevention.md` Implementation Location 섹션에 `bmb_ai_bench/run_cmd.py::_run_one_problem` (Cycle 2740 production code path) 추가. 3 위치 (scripts/run_experiment.py + scripts/run_crosslang.py + run_cmd.py) 모두 명시.

### 백그라운드 bench 상태

| 시점 | 진행 |
|------|------|
| 21:42 (Cycle 2729 시작) | array_alpha 0% |
| 22:30 (Cycle 2736 closeout) | array_remove_dups_sorted (~50%) |
| 23:00 (Cycle 2741) | array_triple_sum (~60%?) |

JSON 파일 (`target/benchmarks/tier_all_2026_05_11_c2729.json`) 아직 미생성. Tier 1 array_* 카테고리는 200+ 벤치마크 추정. 추가 1-2시간 진행 예상.

### bmb-algo README 불일치 발견

`ecosystem/bmb-algo/README.md` 테이블 line 24: `knapsack(100 items) | 2 us | 12 us | 6.3x`

실제 벤치마크 (`benchmarks/bench_algo.py` line 103): `WEIGHTS = [2, 3, 4, 5, 6, 7, 8, 9, 1, 3]` — **10 items**.

추가 헤드라인 (line 3): "90x faster than Python on knapsack. 181x faster on N-Queens" — 표에는 6.3x / 4.1x만 표시. headline 수치 source 불명.

→ ROADMAP HUMAN-Decisions 큐의 M3-5 "bmb-algo README clang vs gcc 라벨 명시" 와 같은 카테고리. 자율 변경 회피.

### HANDOFF 큐 추가

```diff
| M3-5 | bmb-algo README 측정 주장 검증/갱신 | HUMAN | ⏳ — clang vs gcc 라벨 명시 권장
+        + knapsack(100 items) 표기 vs 실제 10 items 불일치 (Cycle 2741)
+        + headline "90x/181x faster" source 검증 필요
```

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| closed/ISSUE 본문 정확 (3 위치 명시) | ✅ |
| 백그라운드 bench 진행 확인 | ✅ array_* progressing |
| bmb-algo README 불일치 기록 | ✅ HANDOFF 큐 |

결함: 없음 (자체 cycle).

## Reflection

### "close ≠ 끝" 패턴 강화

Cycles 2733/2735/2740이 모두 "close 시 더 깊은 grep으로 추가 leverage" 패턴 도출. closed/ ISSUE 본문은 closure 후에도 갱신해야 함 (현재 cycle 적용 — 3 위치 명시).

### 외부 가시성 quality issue (M3-5 scope)

bmb-algo README headline 수치 (90x/181x) 가 표 (6.3x/4.1x)와 불일치. 표 자체도 "knapsack(100 items)" 표기 vs 실제 10 items. 이런 marketing-vs-measurement 갭은:
- 외부 사용자 신뢰 손실 risk
- M3-5 HUMAN 결정 scope에 정합

→ Cycle 2741은 fix 회피하고 발견 사실만 기록. M3-5 HUMAN 검토 시 함께 처리.

## Carry-Forward

- Actionable:
  - **HANDOFF M3-5** 항목에 bmb-algo README knapsack 항목 불일치 추가 (다음 closeout cycle)
- Structural Improvement Proposals: 없음 (현 cycle)
- Pending Human Decisions:
  - M3-5 scope 확장: bmb-algo README "knapsack(100 items)" → "knapsack(10 items)" 정정 + headline "90x/181x" source 검증
- Roadmap Revisions: 없음 (HUMAN decisions 큐에만 추가)
- Next Recommendation: **Cycle 2742** — 백그라운드 bench wait + 추가 ecosystem README quality grep (bmb-compute / bmb-crypto / bmb-json 동일 패턴 확인)
