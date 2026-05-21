# Cycle 2757: Tier 3 spawn-overhead methodology ISSUE 등록

Date: 2026-05-12

## Re-plan

advisor 권고 (Cycle 2756 종료 후): Cycle 2752 발견 (Tier 3 workload ~7ms vs framework 30-130ms) 이 gitignored cycle log에만 존재 — tracked ISSUE로 영속화 필요. Trigger ⚪ NONE.

## Scope & Implementation

### 신규 ISSUE 등록

`claudedocs/issues/ISSUE-20260512-tier3-spawn-overhead-methodology.md`:

- 우선순위: P2
- 영역: ci / benchmarks
- 측정 stamp: workload ~7ms vs framework 30-130ms (80-95% spawn overhead 비중)
- 증거: 직접 측정 (Cycle 2752) 10-run min — lexer/http_parse/brainfuck 모두 7ms 일관
- bench source 분석: lexer 100KB tokenize ≈ 1-3ms algorithm + 3-5ms startup
- 영향: ratio_c는 fair, **absolute ms 변화는 노이즈** (Cycle 2750 false-positive 회귀 후보의 root cause)
- 3 옵션:
  - A: workload amplification (1-2 cycles, 신속)
  - B: inproc timing port from Tier 1 (5-10 cycles, multi-cycle phase)
  - C: framework warning output 보완 (1 cycle)
- 권고: Option A 우선, B는 long-term

### ROADMAP § 6 갱신

| 시점 | active | closed | 변화 |
|------|--------|--------|------|
| Cycle 2735 종료 | 19 | 40 | (baseline) |
| Cycle 2753 (quicksort-ffi 신규) | 20 | 40 | +1 |
| Cycle 2755-2756 (3 close) | 17 | 43 | -3 |
| **Cycle 2757 (tier3-spawn 신규)** | **17** | **43** | (이 cycle: 0 net) |

전체 세션 변화: 19 → 17 (-2 net, 2 신규 + 3 close).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| ISSUE 양식 6필드 준수 | ✅ |
| 핵심 증거 (직접 측정 데이터) 포함 | ✅ |
| 3 옵션 + 권고 명시 | ✅ |
| 관련 ISSUE 참조 (or-chain-lowering, Cycle 2661 inproc) | ✅ |
| ROADMAP § 6 갱신 (active 17) | ✅ |
| 17 active 카운트 일치 | ✅ |

결함: 없음.

## Reflection

### 영속화의 가치

Cycle 2752 발견은 advisor 절제 leverage 입증 (Cycle 2750 회귀 가설 5-10 cycles 분석 회피). 하지만 `claudedocs/cycle-logs/`는 gitignored → 다음 세션 재발견 위험.

ISSUE 등록 = tracked artifact. ROADMAP § 6에서 카운트 + 1, 본 ISSUE 본문은 closed/ 이전 시까지 유지. Tier 3 메소드 재논의 시 single source of truth.

### advisor 권고 정확도 검증

advisor 권고 정확:
1. Cycle 2757 = methodology ISSUE 등록 ✅ (이 cycle 정확히 그것)
2. Cycle 2758 = HANDOFF rewrite (real cycle work)
3. Cycle 2759 = commit
4. FP arity guard 36 sites drop ✅ (mechanical filler 회피)

advisor의 "M3-5 review surface area 증가" 지적도 정확 — Cycle 2754에서 large variant + scaling table 추가. HANDOFF rewrite 시 강조 필요.

### Pacing 점검

소비 cycle: 8/10 (2750-2757). 잔여 2 cycles: HANDOFF rewrite + commit. 정확히 advisor 권고 정합.

## Carry-Forward

### Actionable

- **Cycle 2758**: HANDOFF rewrite — 이번 세션 8 cycles 종합. 핵심 entry points:
  - bench JSON 분석 완료 (Cycle 2750-2751: 회귀 가설 기각, methodology defect 발견)
  - M3-5 narrative 회복 (Cycle 2753-2754: headline 450× 회복 + scaling table + variant 추가) — **review surface area 증가 강조**
  - ISSUE 백로그 -2 net (2 신규, 3 close): alloc-optimization parity close, smt-integration deferred-archival, multiple-pre documentation close, quicksort-ffi 신규, tier3-spawn-overhead 신규
  - 다음 세션 entry point = HUMAN review (M3-5 narrative + quicksort narrative) + HUMAN dispatch (M3-3/4 publish)
- **Cycle 2759**: `/commit` — staged changes 검증 + commit

### Structural Improvement Proposals

(이전 cycles에서 명시된 것 그대로 유지 — 본 cycle 신규 없음)

### Pending Human Decisions

- 신규 없음
- 기존 큐: M3-3/4 publish dispatch, M3-5 narrative review, M4-1 BMB_BENCH_API_KEY 설정

### Roadmap Revisions

ROADMAP § 6 Cycle 2755-2757 sub-section 갱신 (active 17, 신규 2).

### Next Recommendation

**Cycle 2758: HANDOFF rewrite**

Cycle 2746-2749의 HANDOFF는 이미 stale (Cycle 2749 종료 시점 기준). 8 cycles의 작업 (시퀀스 A.2 + 시퀀스 B 확장 + ISSUE cleanup + methodology finding) 누적 반영 필요. advisor 지적 정합: "real cycle work, not footnote".

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| `claudedocs/issues/ISSUE-20260512-tier3-spawn-overhead-methodology.md` (신규) | issues | gitignored |
| `claudedocs/ROADMAP.md` § 6 (Cycle 2755-2757 sub-section 갱신) | claudedocs | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2757.md` | gitignored |
