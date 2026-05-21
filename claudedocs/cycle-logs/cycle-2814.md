# Cycle 2814: Type D 피드백 강화 #2 — 복수 실패 표시

Date: 2026-05-13

## Re-plan

ISSUE-20260326-type-d-failure-analysis 처방 #2: "Show first 3 failures, not just the first one". 자율 가능.

## Scope & Implementation

**변경**: `ecosystem/bmb-ai-bench/bmb_ai_bench/run_cmd.py`

- 단일 fail_msg 수집 → 최대 3개 `failures` 리스트로 전환
- 루프를 break 없이 최대 3개 실패를 모두 수집
- 피드백 헤더: `"test_failure (N of M tests failed):\n{fail_msgs}"`
- `error_normalizer`/`loop_classifier`는 첫 번째 실패만 사용 (분류 정확도 유지)

**Before**:
```
test_failure: Test 4: expected '20\n', got '0\n'
Fix. ...
```

**After**:
```
test_failure (3 of 12 tests failed):
Test 0:
  stdin: '1 1'
  expected: '1\n'
  got: '0\n'
Test 1:
  stdin: '1 2'
  expected: '0\n'
  got: '1\n'
Test 2:
  stdin: '1 3'
  expected: '1\n'
  got: '0\n'
Fix the logic error. ...
```

이런 패턴이면 LLM이 `n % 2` 대신 `(n+1) % 2` 또는 시작이 0-indexed 오류임을 즉시 파악 가능.

## Verification & Defect Resolution

- `py -m pytest tests/ -q` → **30/30 PASS** ✅

## Reflection

**Scope fit**: 완전 부합.
**Latent defects**: 없음. 3개 이상 실패 시 루프는 `len(failures) >= 3`에서 멈추므로 성능 영향 없음.
**Philosophy drift**: 없음. AI-native 개선 직결.
**Roadmap impact**: 없음.

## Carry-Forward

- Actionable: Cycle 2815 — C baseline 전수 검증 (ISSUE-external-problem-validation 처방 #1)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: C baseline 전수 검증으로 숨은 test.json 오류 찾기
