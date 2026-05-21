# Cycle 2812: 6 FAIL problem.md 수정 → B축 99%+ 준비

Date: 2026-05-13

## Re-plan

HANDOFF Carry-Forward: "잔여 6 FAIL 분석 (자율 가능)". 상속 범위 그대로 진행.

## Scope & Implementation

**6개 FAIL 문제 분석**:
- 모두 `loop_count=11` (10 attempt 소진), `loop_types.D=10` (논리 오류)
- 근본 원인: 6개 모두 `problem.md`가 제목 1줄만 있음 → Cycle 2811과 동일 패턴

| 문제 | 실패 run | 오류 |
|------|---------|------|
| `49_roman_to_int` | run3 | "expected '3000', got '3002'" |
| `69_overflow_detect` | run3 | "expected '1 10000000000', got '0 0'" |
| `72_alternating` | run2 | "expected '1', got '0'" |
| `75_longest_plateau` | run1 | "expected '2', got '1'" |
| `83_pipeline` | run3 | "expected '6 9 12', got '1 2 3'" |
| `85_registry_pattern` | run2 | 출력 1줄 초과 |

**수정**: solution.bmb + baseline.c + tests.json 분석 후 각 problem.md에 Input/Output/Example/Constraints/Algorithm 섹션 추가.

**검증**: `bmb-ai-bench validate` → 6개 모두 12/12 tests PASS ✅

**추가 개선 (Cycle 2813 선행)**:
- `run_cmd.py` 테스트 실패 피드백에 stdin 포함 (ISSUE-20260326-type-d-failure-analysis 처방 #1)
- 기존 30 테스트 전체 PASS ✅

## Verification & Defect Resolution

- validate 6개 모두 OK ✅
- py -m pytest tests/ → 30/30 PASS ✅

## Reflection

**Scope fit**: 완전 부합 (HANDOFF 권장 사이클 그대로).
**Latent defects**: 없음. stdin-in-feedback 개선은 Cycle 2813에서 별도 처리.
**Philosophy drift**: 없음. problem.md 수정은 B축 목표(99%+) 직결.
**Roadmap impact**: B축 재측정 후 294→298+/300 기대 (API 재실행 HUMAN 필요).

## Carry-Forward

- Actionable: Cycle 2813 — Type D 피드백 개선 + B-track ISSUE 추가 작업
- Structural Improvement Proposals: 없음
- Pending Human Decisions: B-axis 재측정 (API 키 사용) → 6개 수정 문제 PASS 확인
- Roadmap Revisions: B축 baseline 업데이트 필요 (재측정 후)
- Next Recommendation: B-track ISSUE들 중 자율 가능한 항목 진행 (type-d, first-shot-rate)
