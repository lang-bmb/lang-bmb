# Cycle 2818: Problem.md 45종 전량 수정 (ISSUE-first-shot-rate-low)
Date: 2026-05-13

## Re-plan
ISSUE-20260326-first-shot-rate-low 분석 → 근본 원인: 45개 추가 title-only problem.md (이전 Cycle 2812에서 6개만 수정, 나머지 45개 미발견).
Scope 확장: 45개 전량 수정 (original 계획인 bmb_reference.md 확장보다 더 근본적 해결).

## Scope & Implementation
- `problems/*/problem.md` 45개 전량 수정:
  - 기존 1-line title → 명확한 Input/Output/Algorithm/Example 설명으로 교체
  - 분석 방법: tests.json에서 3-5개 테스트 케이스를 역분석하여 스펙 추론
  - 검증: 전체 100문제 중 0개 title-only 남음 (이전: 45개)
  
- 분석에서 발견한 특이 사항:
  - 60_checksum: 단순 합산이 아닌 sum mod 256
  - 52_base_convert: 각 자릿수를 10진수로 출력 후 연결 (base>10에서 이상한 출력)
  - 50_calculator / 79_mini_interpreter: 가변 길이 명령어 스트림
  - 62_deep_nesting: log10(n) 층수, 음수 pass-through
  - 76_multi_function: 5번째 출력이 sign(sum), 4번째가 abs(min)

## Verification & Defect Resolution
- `py -m pytest tests/ -x -q` → 30/30 PASS
- 전체 100문제 title-only 스캔: 0개 남음 ✓

## Reflection
**임팩트 추정**:
- Cycle 2812: 6개 problem.md 수정 → 6/6 → 98.0%
- 이번 45개 추가 수정 → 예상: first-shot rate 크게 향상 (API 재실행 필요)
- multi-loop success의 73/300이 edge+integration category에 집중
- 재측정 시 B축 99%+ 달성 가능성 있음

**주의**: 일부 problem.md 스펙이 애매한 경우 (62_deep_nesting의 음수 처리 등)은 추후 검증 필요.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - 모든 problem.md에 baseline.c와의 일관성 검증 필요 (일부 문제의 baseline.c가 구형 스펙일 수 있음)
- Pending Human Decisions:
  - B축 재측정 필요 (45개 problem.md 개선 + 다중 실패 피드백 포함) — HUMAN
- Roadmap Revisions: ISSUE-20260326-first-shot-rate-low → 주요 원인(title-only) 해소됨
- Next Recommendation: B-track ISSUE 점검 및 최종 commit
