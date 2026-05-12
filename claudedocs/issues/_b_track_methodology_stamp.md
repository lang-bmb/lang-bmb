# B-Track Methodology ISSUE 일괄 측정 stamp (Cycle 2730)

> 이 파일은 2026-03-26 bmb-ai-bench 실험에서 도출된 9 ISSUE의 공통 측정 stamp이다.
> 개별 ISSUE 파일에 측정 stamp 블록을 반복하는 대신, 이 단일 reference로 일괄 적용.

## 일괄 적용 대상 (Cycle 2730 표준화)

| ISSUE | Category |
|-------|----------|
| ~~`ISSUE-20260326-context-overflow-prevention.md`~~ | Experiment Infrastructure — **CLOSED Cycle 2739** (sliding window truncation 적용) |
| `ISSUE-20260326-crosslang-reference-asymmetry.md` | Experiment Methodology |
| `ISSUE-20260326-external-problem-validation.md` | Experiment Methodology |
| `ISSUE-20260326-first-shot-rate-low.md` | Compiler / Error Messages |
| `ISSUE-20260326-integration-category-weakness.md` | Language Design / Compiler |
| `ISSUE-20260326-multi-model-validation.md` | Experiment Methodology |
| `ISSUE-20260326-problem-difficulty-bias.md` | Experiment Methodology |
| `ISSUE-20260326-statistical-testing.md` | Experiment Analysis |
| `ISSUE-20260326-type-d-failure-analysis.md` | Experiment Analysis |

## 공통 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-03-26 (bmb-ai-bench 100문제 실험 — 비공식) |
| `stale_after` | **2026-04-26 (이미 STALE — 1.5개월 경과 + B 공식 baseline 미선언)** |
| `measurement_source` | `bmb-ai-bench/results/` (2026-03-26 ~100 problems, claude-text 단일 모델) |
| `observed_rate` | BMB 56.3% first-shot / 90% with feedback (vs C 64.7%/82%, Python 67.2%/87%) |
| `scope` | bmb-ai-bench 100문제 suite — Algorithm/Integration/Type-A~D 카테고리 |
| `env_hash` | v0.51.22 / claude-text 1 model / 2026-03-26 prompt template |

## 핵심 메타 발견

1. **B축 비공식 baseline만 존재** — ROADMAP § 5에서 "비공식 ~90.9%" 명시. 공식 baseline 미선언.
2. **HUMAN 결정 잠금**: `BMB_BENCH_API_KEY` 설정 + 고정 모델 + 결과 commit → 공식 baseline 선언 (M4-1)
3. **재실행 시 변경 가능성 큼**:
   - v0.98은 v0.51.22 대비 codegen 광범위 개선
   - 새 prompt template (BMB Quick Reference 정정 후)
   - 다중 모델 검증 (multi-model-validation ISSUE)

## 후속 조치 (다음 cycle 이후)

| ISSUE | 권장 |
|-------|------|
| 모든 9건 | M4-1 공식 baseline 실행 후 재측정값으로 일괄 갱신 |
| crosslang-reference-asymmetry | 단일 cycle 처리 가능 — Quick Reference 제거/축약 후 재측정 |
| problem-difficulty-bias | 100문제 → 200문제 확장 검토 |
| multi-model-validation | gpt-4 / gemini / claude-haiku 등 3 model 비교 |
| statistical-testing | bootstrap CI + paired t-test 추가 (분석 코드만 변경) |

## 양식 표준화 이력

- 2026-05-11 Cycle 2730: 일괄 stamp 적용 (이 파일 신규)
- 다음 갱신 트리거: M4-1 baseline 실행 후
