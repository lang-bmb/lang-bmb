# Cycle 2813: Type D 피드백 강화 (stdin 포함)

Date: 2026-05-13

## Re-plan

ISSUE-20260326-type-d-failure-analysis (HIGH): "Enhanced test feedback: Include the stdin that caused the failure". 자율 가능. 즉시 진행.

## Scope & Implementation

**변경**: `ecosystem/bmb-ai-bench/bmb_ai_bench/run_cmd.py`

- Test failure 피드백에 stdin 포함:
  ```
  전: "Test 4: expected '20\n', got '0\n'"
  후: "Test 4:\n  stdin: '5 0 2 0 3 1 4 3'\n  expected: '20\n'\n  got: '0\n'"
  ```
- timeout 메시지에도 stdin 포함: `"Test {i}: timeout (stdin: {stdin!r})"`
- feedback 헤더: `"test_failure:\n{fail_msg}\nFix the logic error. ..."` (더 명확한 안내)

**동기**: 모든 Type D 실패의 근본 원인은 LLM이 어떤 입력이 실패했는지 모름 → 잘못된 패턴 수정.
이미 초기 프롬프트에 5개 테스트 예시가 있지만, 실패 피드백 시 어떤 케이스가 틀렸는지 명시하지 않았음.

## Verification & Defect Resolution

- `py -m pytest tests/ -q` → **30/30 PASS** ✅ (기존 테스트 모두 통과)
- 변경 후 실제 실행 검증: API 키 필요 (HUMAN); 코드 리뷰 관점에서 변경 최소, 명확, 부작용 없음.

## Reflection

**Scope fit**: 완전 부합 (ISSUE type-d 처방 #1 직접 구현).
**Latent defects**: 없음. `normalize_error` 함수는 `fail_msg` 내용에 무관하게 동작.
**Philosophy drift**: 없음. AI-native 언어 개선 = AI 사용성 개선.
**Roadmap impact**: 향후 B-axis 재실행 시 Type D 자기수정율 향상 기대.

## Carry-Forward

- Actionable: Cycle 2814 — ISSUE-20260326-type-d-failure-analysis 추가 처방 (3 failure 표시)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 실제 LLM 재실행으로 Type D 개선 검증 필요
- Roadmap Revisions: 없음
- Next Recommendation: type-d ISSUE 처방 #2 (복수 실패 표시) 또는 first-shot-rate 개선
