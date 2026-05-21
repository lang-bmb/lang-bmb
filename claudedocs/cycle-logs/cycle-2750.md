# Cycle 2750: 시퀀스 A.2 — P-track ISSUE 측정 stamp 갱신 + 회귀 후보 탐지

Date: 2026-05-12

## Re-plan

HANDOFF § 8 명시: bench JSON ✅ 생성 (`tier_all_2026_05_11_c2729.json`, 37401 bytes, 00:11 KST = 15:11 UTC). Trigger 초기 ⚪ NONE.

STEP 2 진행 중 lexer +31pp / json_serialize +29.6pp / json_parse +6.7pp 회귀 후보 발견 → 🟡 SCOPE ADJUST: 단순 stamp 갱신 + 회귀 가설 분석 + 환경 변동성 비교표 추가.

## Scope & Implementation

### 측정 분석 (target/benchmarks/tier_all_2026_05_11_c2729.json)

- timestamp: 2026-05-11T15:11:41Z (start), total_time_ms 8979496 (~2.5h wall)
- 330 results, 5-run aggregated median per (tier, name)
- Tier 1 9 P-track benchmarks: 8/9 ≤1.05x (knapsack outlier 제외) — M1 가설 유지
- Tier 3 7 P-track benchmarks: 절대 시간 +30~96% 증가 (전반적 슬로다운). 비대칭 패턴:
  - C가 더 슬로다운 (환경 변동 가설 우위): brainfuck, csv_parse, http_parse
  - BMB가 더 슬로다운 (회귀 후보): **lexer (+31pp), json_serialize (+29.6pp), json_parse (+6.7pp)**

### 갱신 파일 (3 ISSUE + 1 ROADMAP)

| 파일 | 변경 |
|------|------|
| `claudedocs/issues/ISSUE-20260413-hashmap-perf.md` | 측정 추이 row append (2026-05-11, 1.040x, +1.2pp 노이즈 범위), 우선순위 P1 유지 |
| `claudedocs/issues/ISSUE-20260413-alloc-optimization.md` | 측정 추이 row append (2026-05-11, 1.010x, -3.3pp ≤1.05x 충족), 우선순위 P2→**P3 강등**, close 후보 (10-run 재측정 시 확정) |
| `claudedocs/issues/ISSUE-20260511-or-chain-lowering.md` | 측정 추이 row append (2026-05-11, lexer 1.310x, +31pp ⚠️), 환경 변동성 가설 + Tier 3 비대칭 패턴 표 추가, 가설 검증 액션 3건 명시 |
| `claudedocs/ROADMAP.md` § 5 | 새 sub-section "Cycle 2750 — tier_all_c2729 갱신" 추가: 16 P-track benchmarks 전수 표 + 환경 변동 진단 |

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 3 ISSUE 파일 Cycle 2750 stamp 라인 존재 | ✅ (grep -c: 3+3+4) |
| ROADMAP Cycle 2750 sub-section 추가 | ✅ (grep -c: 2) |
| 의도 외 변경 없음 | ✅ |
| 회귀 후보 단정 회피 (5-run noise 가설 우위) | ✅ — 재측정 carry-forward |
| `_template.md` 양식 보존 | ✅ — 측정 추이 row append, 기존 row 보존 |

결함: 없음.

## Reflection

### 단일 cycle 내 발견 가치

HANDOFF가 요구한 "3 ISSUE stamp 갱신"은 mechanical task였으나, 데이터 자체에서 lexer/json_serialize 회귀 후보 surfaced. 즉시 close 압력 회피 ✅:
- alloc-optimization 1.010x → P-track 기준 충족이지만 5-run 단일 측정으로 close 비결정 → carry-forward 권고
- or-chain-lowering 1.310x → 즉시 회귀 단정 회피, 환경 변동 가설 1순위 유지

### 환경 변동 vs 실제 회귀 — 비대칭 패턴 분석

핵심 진단: **Tier 3 전반 +30~96% 슬로다운에서 BMB:C ratio 변화 방향이 비대칭**.
- 동질적 환경 변동이라면 BMB와 C가 동일 비율 슬로다운 → ratio 변화 ≈ 0
- 비대칭 슬로다운 (BMB가 더 슬로다운) → 환경 변동이 BMB 패턴에 더 민감하거나 실제 codegen 회귀
- brainfuck/csv_parse/http_parse가 BMB 유리 방향 환경 변동을 보임 → 단순 thermal/load는 아님
- → 가설: Tier 3 5-run noise floor가 절대 ms 차이의 ~10ms 범위 (예: 28 ± 5ms vs 51 ± 5ms는 ratio 측정에 큰 차이)

### 우선순위 조정의 필요

Cycle 2751 = 시퀀스 B (bmb-algo bench, M3-5) 가 HANDOFF 명시 계획. 하지만 lexer는 BMB 도그푸딩 핵심 도메인 (parser/lexer는 Track S 컴파일러 구성요소). publish narrative 일관성 위해:
- **신 계획**: Cycle 2751 = Tier 3 10-run noise-gate 재측정 (1 cycle) → 회귀 가설 검증
- Cycle 2752+ = 시퀀스 B (bmb-algo bench)

이는 Roadmap Revision으로 분류.

## Carry-Forward

### Actionable

- **Cycle 2751**: Tier 3 10-run noise-gate 재측정 (lexer/json_serialize/json_parse 회귀 가설 검증). 명령: `./scripts/benchmark.sh --tier 3 --repeat 10 --noise-gate` (또는 동등). 결과 ≤1.10x 재현 시 → 환경 변동 결론. 회귀 재현 시 → IR diff 시작.
- **Cycle 2751 (alt)**: 10-run 재측정으로 alloc-optimization 1.010x 재현 시 → ISSUE close
- **Cycle 2752 (조정)**: 시퀀스 B M3-5 bmb-algo bench 재실행 + README 정정 (회귀 검증 완료 후)

### Structural Improvement Proposals

- **benchmark.sh tier_all default → 10-run + noise-gate**: 현재 tier_all_c2729는 5-run, Tier 3 short benches에서 false-positive 1.310x. tier_all default를 10-run + noise-gate로 변경하면 매 사이클 측정 신뢰도 향상. **Trade-off**: ~2.5h → ~5h (2x). HUMAN 결정 권고 (자동화 cost vs measurement quality).
- **HANDOFF timezone notation**: "00:11"이 KST이고 JSON의 UTC와 9h 차이. Confusion 회피 위해 HANDOFF에 `(KST)` suffix 표준화 제안.

### Pending Human Decisions

- 신규 없음 (기존 큐 유지)
- alloc-optimization close 결정 (10-run 재측정으로 ≤1.05x 재현 시 자율 close 가능)

### Roadmap Revisions

ROADMAP `§ 4 M3 잔여 태스크` 순서 미변경 (M3-5 자율 잔여), 단 cycle 단위 우선순위 변경:
- **Cycle 2751** = Tier 3 10-run 재측정 (회귀 가설 검증, **NEW priority**)
- **Cycle 2752+** = 시퀀스 B bmb-algo (HANDOFF original Cycle 2751 plan, +1 cycle slip)

이유: lexer 1.310x 미해결로 M3-5 bmb-algo publish narrative 정합성 위협 (Track S 컴파일러 도메인 회귀가 publish 직전에 surfaced되면 fix-or-narrative-adjust 부담).

### Next Recommendation

**Cycle 2751: Tier 3 10-run noise-gate 재측정** — `./scripts/benchmark.sh --tier 3 --repeat 10` 실행, 결과 분석:
- 시나리오 A (회귀 재현 안 함): alloc-optimization close + or-chain ratio 정상화 → Cycle 2752 시퀀스 B 진입
- 시나리오 B (lexer 1.10x+ 재현): IR diff (historic build vs current) → codegen 회귀 추적 (분리 phase)
- 시나리오 C (json_serialize 1.10x+ 재현): or-chain과 무관한 별도 패턴 → 신규 ISSUE 등록 + 분석

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| ISSUE-20260413-hashmap-perf.md | `claudedocs/issues/` | gitignored |
| ISSUE-20260413-alloc-optimization.md | `claudedocs/issues/` | gitignored |
| ISSUE-20260511-or-chain-lowering.md | `claudedocs/issues/` | gitignored |
| ROADMAP.md § 5 (Cycle 2750 sub-section) | `claudedocs/` | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2750.md` | gitignored |
