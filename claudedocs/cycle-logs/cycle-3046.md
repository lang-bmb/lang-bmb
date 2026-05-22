# Cycle 3046: run-all-ai-bench.bmb — 전체 문제 일괄 실행 (M6-P2 구현 2단계)
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3045): 단일 문제 runner 완성. 전체 문제 일괄 실행 + 결과 저장 구현.

## Scope & Implementation

**run-all-ai-bench.bmb** (신규, `scripts/`):
- `list_dir(problems_root)` → `str_lines` → `svec_get` 패턴으로 100개 문제 순회
  - `run-all-bench-tests.bmb`와 동일 패턴 (기존 검증된 방법)
- `process_problems(entries, bmb, runner, problems_root, idx, jsonl)`: 재귀 루프
  - `file_exists(prob_dir + "/tests.json")` 확인 후 exec_with_stdin으로 단일 runner 실행
  - 결과 파싱: `is_pass()` + `scan_attempts()` → `result_json()` jsonl 라인 생성
- `write_summary(jsonl, ...)`: 
  - `count_lines(jsonl)` + `count_pass(jsonl)` → 집계
  - `write_file(results_file, jsonl)` → JSONL 파일 저장
- 출력 위치: `ecosystem/bmb-ai-bench/results/results-{BMB_DATE}.jsonl`
  - BMB_DATE env var (default: "latest")

**디버깅 과정**:
- 초기 실행 0/0 PASS: test directory가 비어있을 때 실행됨 (타이밍 이슈)
- list_dir + str_lines + svec 패턴 자체는 정상 동작 확인 (debug script)
- `problems_root` 출력 추가 → 문제 없음, 실제 파일 존재 후 재실행 시 정상

## Verification & Defect Resolution

**테스트 결과 (2개 문제 subset)**:
```
[1] 04_fibonacci ...
    PASS (1 attempt/s)
[2] 05_gcd ...
    PASS (1 attempt/s)
=== RESULT: 2/2 (100%) PASS ===
```

JSONL 출력:
```json
{"problem_id":"04_fibonacci","pass":true,"attempts":1}
{"problem_id":"05_gcd","pass":true,"attempts":1}
```

## Reflection
- `exec_with_stdin`으로 단일 runner 호출 시 child process stdout이 parent stdout에 합쳐지지 않음
  - parent의 `println` 호출은 parent stdout으로 실시간 출력됨 ✓
  - child의 `println` 호출은 exec_with_stdin return value에 포함됨 (화면 미노출)
  - 결과: 사용자는 "[1] problem_name ..." + "PASS/FAIL" 라인만 실시간으로 봄
  - 상세 LLM 대화 내용은 숨겨짐 → 일괄 실행에 적합한 UX
- context truncation 미구현: 긴 루프 시 messages 배열 팽창 → 후속 사이클 TODO
- `bmb check` success (only warnings)

## Carry-Forward
- Actionable:
  - Cycle 3047: bmb_reference.md를 더 짧은 핵심 reference로 교체 or 제한적 포함
  - Cycle 3047: context truncation 추가 (attempt > 5 시 messages 슬라이딩)
  - Cycle 3047: 실패 케이스 상세 피드백 (stdin/expected/got 표시)
- Structural Improvement Proposals: 없음 (현재 구조 충분)
- Pending Human Decisions: 없음
- Roadmap Revisions: M6-P2 2단계 완료 (전체 문제 일괄 runner working)
- Next Recommendation: Cycle 3047 — 개선 + 실제 100문제 pilot 실행 준비
