# Cycle 3047: run-ai-bench.bmb 개선 — context truncation + 실패 상세 피드백
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3046): (1) bmb_reference.md 단축 검토, (2) context truncation, (3) 실패 케이스 상세 피드백.
Python 원본(`run_cmd.py`) 정밀 비교 → 구현 우선순위 재조정.

## Scope & Implementation

**`scripts/run-ai-bench.bmb`** 변경:

### 1. `find_first_fail` 신규 함수
- `tests.json`을 재순회해 첫 번째 실패 케이스를 찾아 `"stdin=[...] expected=[...] got=[...]"` 반환
- `run_one_test` 재사용 (테스트 1회 더 실행, failure 시에만 호출이므로 허용)

### 2. `attempt_loop` 개선
- 시그니처: `init_msgs: String` 파라미터 추가
- **Context truncation**: `attempt >= 5` 시 `msgs` 대신 `init_msgs`를 base로 사용 → hard reset
  - Python은 `messages[0] + messages[-4:]` sliding window; BMB는 hard reset (구현 단순화 우선)
- **실패 상세 피드백**: test failure 시 `find_first_fail` 호출 → "First failure: stdin=[...] expected=[...] got=[...]" 포함

### 3. `main()` 호출 업데이트
`attempt_loop(..., init_msg, init_msg, 1, max_lps)` — init_msgs를 msgs와 동일하게 초기 전달

**Python 원본과의 차이 (확인 완료)**:

| 항목 | Python | BMB |
|------|--------|-----|
| 테스트 실행 방식 | `bmb build --release` → native binary | `bmb run` interpreter mode |
| 실패 피드백 | 최대 3개 failures | 1개 (Cycle 3047 개선) |
| Context truncation | sliding window (first + last 4) | hard reset at attempt ≥ 5 |
| error_normalizer | normalize + classify | raw check output |

**bmb_reference.md 단축**: 37KB 포함 시에도 GPUStack 100.0% 달성. 단축 불필요. 현행 유지.

## Verification & Defect Resolution

`bmb check scripts/run-ai-bench.bmb` → success (35 warnings only — postcondition/unused)
`bmb check scripts/run-all-ai-bench.bmb` → success (no errors)

retry loop 자체는 LLM 호출 없이는 테스트 불가. 코드 경로 정적 분석:
- `append_round(msgs, content, feedback)`: `str_substr(msgs, 0, msgs.len()-1) + "," + asst_msg + "," + user_msg + "]"` — 마지막 `]` 제거 후 새 메시지 삽입. ASCII `]` 1바이트 → `len()-1` 바이트 인덱싱 정확.
- `init_msgs`가 `"[sys,user]"` 형태 → `append_round(init_msgs, ...)` → `"[sys,user,asst,user]"` — 올바른 JSON 배열.
- `find_first_fail` 재귀: `test_loop`와 동일한 경계 조건(`sk > nb` 스킵) 적용 ✓

## Reflection
- Scope fit: carry-forward 3항목 중 2항목 완료, 1항목(ref 단축) 불필요로 판단.
- 실패 피드백 3개 수집(Python 방식)은 BMB에서도 구현 가능하나 추가 루프 필요. 현재 1개로도 LLM 재시도에 충분.
- Hard reset vs sliding window: attempt ≥ 5는 이미 다수 실패 상태 — fresh start가 오히려 효과적일 수 있음.
- Python 원본에서 발견된 차이: build 방식(native vs interp). interpreter mode는 더 느리지만 GEP codegen 버그를 우회하므로 현재 선택 유지.

## Carry-Forward
- Actionable:
  - Cycle 3048: run-all-ai-bench.bmb 중단 재개(resume) 지원 — 기존 JSONL 읽어서 완료된 problem 스킵
  - Cycle 3048: 100-problem 파일럿 실행 준비 (env 설정 확인 + dry-run)
  - Cycle 3049: retry loop 엔드투엔드 검증 (LLM 있는 환경에서 1회 실패 강제 후 retry 확인)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: M6-P2 3단계 완료 (개선 완료)
- Next Recommendation: Cycle 3048 — resume 지원 + 파일럿 실행 준비
