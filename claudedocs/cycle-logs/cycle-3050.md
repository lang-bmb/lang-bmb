# Cycle 3050: run-ai-bench.bmb test_loop 최적화 — dead code 제거
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3049): 파일럿 모드 완료. GPUStack 실행은 HUMAN-gated. 자율 작업으로 코드 품질 개선.

## Scope & Implementation

**`scripts/run-ai-bench.bmb`** 변경:

### `test_loop` dead code 제거

Python `run_cmd.py` 정밀 비교 결과 발견:
- `test_loop`의 `first_fail: String` 파라미터와 `fail_desc` 수집 로직이 반환값(`i64` 카운트)에 포함되지 않는 dead code
- 실패한 테스트마다 `exec_with_stdin` 호출 1회 추가 발생 (불필요한 서브프로세스)
- 실제 failure detail은 `find_first_fail`이 별도로 수집 (attempt_loop에서 사용)

**Before** (비효율):
```bmb
fn test_loop(..., first_fail: String) -> i64 =
    ...
    else {
        let fail_desc = if first_fail.len() == 0 {
            "..." + exec_with_stdin(...)  // 실패마다 프로세스 추가 실행 (dead)
        } else { first_fail };
        test_loop(..., fail_desc)  // fail_desc는 반환값에 포함 안 됨
    };
```

**After** (최적화):
```bmb
fn test_loop(...) -> i64 =
    ...
    else {
        test_loop(..., passed, failed + 1)  // 카운트만 증가
    };
```

- `first_fail: String` 파라미터 제거
- `fail_desc` 수집 + 이중 `exec_with_stdin` 제거
- 호출부: `test_loop(tests_json, bmb, temp_src, 0, 0, 0)` (6 args → 5 args)

## Verification & Defect Resolution

`bmb check scripts/run-ai-bench.bmb` → success (35 warnings, no errors) ✓

## Reflection
- 불필요한 서브프로세스 1회/실패케이스 제거 — 실패가 많은 초기 시도(attempt 1-3)에서 효과적
- 코드 의미 명확화: `test_loop`은 카운트 전용, `find_first_fail`은 feedback 상세 전용으로 책임 분리
- Python `run_cmd.py` vs BMB 갭: 3 failures 수집(Python) vs 1 failure(BMB)는 유지. 현재 1개로 충분.

## Carry-Forward
- Actionable:
  - Cycle 3051: JSONL 결과 분석 BMB 스크립트 (`analyze-bench-results.bmb`) 작성
  - Cycle 3052: GPUStack 파일럿 실행 (3문제, HUMAN 승인 후) → 재확인
- Structural Improvement Proposals: 없음
- Pending Human Decisions: GPUStack 파일럿 실행 승인
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3051 — analyze-bench-results.bmb 작성 (JSONL 통계)
