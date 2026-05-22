# Cycle 3045: run-ai-bench.bmb — single-problem LLM loop (M6-P2 구현 1단계)
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3044): P1 GEP codegen 버그 ISSUE 등록 완료. M6-P2 착수.
이번 사이클: run-ai-bench.bmb 구현 — curl+LLM+bmb check+test 루프 단일 문제 runner.

## Scope & Implementation

**run-ai-bench.bmb** (신규, `scripts/`):
- Python bmb-ai-bench `_run_one_problem()` 로직을 BMB로 포팅
- `json_encode()`: `str_replace` 기반으로 구현 (Unicode-safe)
  - `jenc_loop` 방식 폐기 — `char_code_at` (char-index) vs `len()` (byte-count) 불일치
  - `str_replace(s, "\\", "\\\\")` → `str_replace(s1, "\"", "\\\"")` → `\n`, `\r`, `\t` 순
- `make_initial_msgs()`: Python 구조와 일치
  - `## BMB Reference\n{ref_doc}\n---\n{problem_md}\n## Examples\n{preview}`
  - ref_doc = `ecosystem/bmb-ai-bench/protocol/bmb_reference.md` (37KB)
- `build_test_preview()`: tests.json 첫 5개 테스트 추출 (`-> stdout:` ASCII 사용)
- `attempt_loop()`: check FAIL/test FAIL → LLM 재시도 루프
- `run_one_test()`: `exec_with_stdin(bmb, "run src", str_replace(stdin, " ", "\n"))`

**핵심 버그 수정**:
1. `jenc_loop` out-of-bounds: `len()` = 바이트 수, `char_code_at(i)` = char 인덱스 불일치
   - 문자 `→` (U+2192 = E2 86 92) → `len()` 3바이트, `char_code_at(i)` 1 char
   - `preview_one`에서 `→` → `->` 변경 + json_encode 전면 교체
2. `test_preview_loop` dead code 제거
3. debug println 제거

## Verification & Defect Resolution

**테스트 결과**:
- `04_fibonacci`: PASS (1/5 시도, 15/15 테스트)
- `05_gcd`: PASS (1/5 시도, 15/15 테스트)

**BMB 인터프리터 string builtin 불일치 확인**:
- `len()` → `s.len()` (Rust) = **바이트 수**
- `char_code_at(i)` → `s.chars().nth(i)` = **char 인덱스** (Unicode scalar)
- `str_substr(s, start, len)` → **바이트 인덱스**
- 결론: BMB string API가 일관되지 않음. ISSUE 별도 등록 대상.

## Reflection
- Python 버전과 주요 차이:
  - 인터프리터 모드 사용 (`bmb run` vs native build) — 속도 느리지만 GEP 버그 우회
  - context truncation 미구현 (Python은 5개 초과 시 앞 1개+뒤 4개 유지)
  - 피드백 상세도: Python은 최대 3개 실패 케이스, BMB 버전은 집계만 제공
  - stdin 변환: `str_replace(" ", "\n")` — Python은 stdin as-is 전달 (차이 없음: read_int가 모든 공백 수용)
- 37KB bmb_reference.md 포함 시에도 정상 동작 (json_encode str_replace 방식)

## Carry-Forward
- Actionable:
  - Cycle 3046: 모든 문제 일괄 실행 스크립트 (`run-all-ai-bench.bmb`) 구현
  - context truncation 추가 (긴 루프 시 메시지 배열 팽창 방지)
  - test 피드백 상세화 (실패 케이스 stdin/expected/actual 포함)
- Structural Improvement Proposals:
  - `len()` vs `char_code_at` 불일치 → ISSUE 등록 (P2: systems lang에서 일관성 필요)
  - `str_byte_at(s, i)` 또는 `str_char_len(s)` builtin 추가 고려
- Pending Human Decisions: 없음
- Roadmap Revisions: M6-P2 1단계 완료 (단일 문제 runner working)
- Next Recommendation: Cycle 3046 — run-all-ai-bench.bmb (결과 JSON 출력 포함)
