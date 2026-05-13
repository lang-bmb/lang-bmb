# BMB Session Handoff — 2026-05-13 (Cycles 2816-2819 — B축 인프라 개선)

> **HEAD**: commit 예정 (이 세션 종료 시)
> **이전 HEAD**: `e11e62b5` (Cycle 2811)
> **3-Stage Fixed Point**: ✅ S2 == S3 (Cycle 2792, 이번 세션 bootstrap 미변경)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2820

---

## 이번 세션 작업 요약 (Cycles 2816-2819)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2816 | 통계 검정 구현 | `analysis/stats.py` (Wilson CI + McNemar), CLI `stats` 서브커맨드 |
| 2817 | C/Python reference 추가 | `protocol/c_reference.md` + `protocol/python_reference.md`, `run_crosslang.py` 대칭 수정 |
| 2818 | problem.md 45종 전량 수정 | title-only → Input/Output/Example 완전 설명 (총 51종: Cycle 2812에서 6종 + 이번 45종) |
| 2819 | ISSUE 상태 갱신 | 4개 ISSUE RESOLVED/LARGELY RESOLVED 마킹, HANDOFF/ROADMAP 갱신 |

### B-track ISSUE 상태 (Cycle 2819 기준)

| ISSUE | 우선순위 | 상태 |
|-------|---------|------|
| `ISSUE-20260326-statistical-testing` | MEDIUM | ✅ **RESOLVED** (Cycle 2816) |
| `ISSUE-20260326-crosslang-reference-asymmetry` | HIGH | ✅ **RESOLVED** (Cycle 2817) |
| `ISSUE-20260326-first-shot-rate-low` | MEDIUM | 🔄 **LARGELY RESOLVED** (Cycle 2818, 재측정 HUMAN) |
| `ISSUE-20260326-type-d-failure-analysis` | HIGH | 🔄 **ROOT CAUSE RESOLVED** (Cycle 2818, 재측정 HUMAN) |
| `ISSUE-20260326-integration-category-weakness` | HIGH | OPEN (언어 개선 필요, 일부 HUMAN) |
| `ISSUE-20260326-external-problem-validation` | MEDIUM | OPEN (HUMAN) |
| `ISSUE-20260326-multi-model-validation` | HIGH | OPEN (HUMAN) |
| `ISSUE-20260326-problem-difficulty-bias` | LOW | OPEN (HUMAN) |

---

## B축 현재 상태

### 공식 baseline (2026-05-13)

| 필드 | 값 |
|------|-----|
| 총 runs | 300 (100문제 × 3회) |
| 성공 | 294 (98.0%) |
| 측정 시점 | Cycle 2810-2811 (stale 없음) |
| JSON | `claudedocs/measurements/b_baseline_2026-05-13_c2810.json` |

**⚠️ 재측정 권장**: Cycle 2812+2818에서 총 51개 problem.md 수정 완료. 재측정 시 99%+ 달성 예상.

### crosslang 통계 (2026-03-26 기준 — 비대칭 조건)

| 언어 | 통과율 | 95% CI |
|------|--------|--------|
| BMB | 90.0% | [86.1%–92.9%] |
| C | 82.0% | [77.3%–85.9%] |
| Python | 84.3% | [79.7%–88.0%] |

- McNemar BMB vs C: p=0.0863 (NOT significant at α=0.05)
- ⚠️ 이 데이터는 C/Python reference 없는 조건에서 측정됨 (Cycle 2817에서 수정됨)
- 공정한 비교를 위해 crosslang 재실험 필요 (HUMAN)

---

## 다음 세션 우선순위 (Cycle 2820+)

### 1순위 — HUMAN 결정 후 즉시 가능

**B축 재측정** (HUMAN: API key + 8-12h 실행):
```bash
bmb-ai-bench run --all --runs 3 --model claude-sonnet-4-6
```
- 51개 problem.md 수정 효과 검증
- 목표: 99%+ 달성

**crosslang 재실험** (HUMAN: API key + 24h 실행):
```bash
python scripts/run_crosslang.py --all --runs 3 --model claude-sonnet-4-6
```
- 이제 C/Python reference 포함 → 공정한 비교

### 2순위 — 자율 가능

- `ISSUE-20260326-integration-category-weakness`: BMB 언어 기능 개선 (integration 카테고리 63% → 개선)
- `bmb_reference.md` 확장: 현재 200줄 → 더 많은 BMB 관용 패턴
- problem.md 검증: 일부 스펙 추론이 모호할 수 있음 (특히 52_base_convert, 62_deep_nesting)

### 3순위 — HUMAN 결정 필요

- 다중 모델 crosslang 실험 (GPT-4o, Gemini 등 추가)
- 외부 문제 세트 검증
- 더 어려운 문제 추가 (difficulty bias 해소)

---

## 기술 상태 (변경 없음)

| 항목 | 상태 |
|------|------|
| Bootstrap 3-Stage Fixed Point | ✅ S2 == S3 (Cycle 2792) |
| `cargo test --release` | ✅ (BMB compiler 변경 없음) |
| `py -m pytest tests/ -x -q` (bmb-ai-bench) | ✅ 30/30 PASS |
| M1 Self-Validated | ✅ COMPLETE |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~99% |
| M4 Adopted | 🔄 ~50% |

---

## 파일 변경 목록 (이번 세션)

### 신규 생성
- `ecosystem/bmb-ai-bench/bmb_ai_bench/analysis/stats.py`
- `ecosystem/bmb-ai-bench/protocol/c_reference.md`
- `ecosystem/bmb-ai-bench/protocol/python_reference.md`
- `ecosystem/bmb-ai-bench/problems/*/problem.md` (45개 — 전체 100개 중 title-only 해소)

### 수정
- `ecosystem/bmb-ai-bench/bmb_ai_bench/run_cmd.py` (다중 실패 피드백, stdin 포함)
- `ecosystem/bmb-ai-bench/bmb_ai_bench/cli.py` (`stats` 서브커맨드 추가)
- `ecosystem/bmb-ai-bench/scripts/run_crosslang.py` (C/Python reference 포함)
- `ecosystem/bmb-ai-bench/problems/46_csv_parser/baseline.c` (스펙 수정)
- `ecosystem/bmb-ai-bench/problems/47_word_count/baseline.c` (스펙 수정)
- `ecosystem/bmb-ai-bench/problems/49_roman_to_int/tests.json` (UB 케이스 수정)
- `claudedocs/issues/ISSUE-20260326-*.md` (4개 상태 갱신)
- `claudedocs/cycle-logs/cycle-2812.md` through `cycle-2819.md`
