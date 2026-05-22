# Cycle 3041: run-all-bench-tests.bmb — 전체 문제 일괄 실행
Date: 2026-05-22

## Re-plan
Cycle 3040에서 run-bench-tests.bmb 완성. 이번 사이클: 전체 100개 문제 일괄 실행 + 집계 점수 보고.

## Scope & Implementation

**`scripts/run-all-bench-tests.bmb`** 신규 작성:
- `list_dir(problems_root)` → `str_lines()` → SvecHandle 분할
- 각 문제 디렉토리에 대해 `exec_with_stdin(bmb, "run runner.bmb prob_dir", "")` 호출
- "Results: X/Y passed" 문자열 파싱으로 집계 (parse_digits 재귀 + search_from)
- 집계 후 `=== TOTAL: X/Y (N%) passed | skipped: M ===` 출력

**타입 수정**: `process_entries` 파라미터 `entries: i64` → `entries: SvecHandle`

## Verification & Defect Resolution

```
=== BMB AI Bench: ./ecosystem/bmb-ai-bench/problems ===

✓ 01_binary_search: 15/15
✓ 02_quicksort: 15/15
... (100개 전부)
✓ 99_bounded_queue_contract: 12/12

=== TOTAL: 1230/1230 (100%) passed | skipped: 0 ===
```

**100/100 문제, 1230/1230 테스트 케이스 전부 통과** ✅

## Reflection
- BMB 자체가 AI 벤치마크 100% 통과 — 해석기 실행 경로에서 언어 완성도 검증
- run-all-bench-tests.bmb는 3-level 인터프리터 스택 (all→run-bench-tests→solution)으로 느리지만 정확
- 네이티브 빌드 지원 시 실행 속도 대폭 개선 가능 (exec_with_stdin 네이티브 포팅 필요)
- str_lines + list_dir + SvecHandle 조합이 디렉토리 순회에 잘 작동

## Carry-Forward
- Actionable: 없음 (M6-P1 scripts 포팅 완료)
- Structural Improvement Proposals:
  - exec_with_stdin 네이티브 지원 (현재 인터프리터 only) — codegen에 already registered하나 C 구현 미검증
  - run-all-bench-tests.bmb 병렬화 고려 (현재 직렬)
- Pending Human Decisions: 없음
- Roadmap Revisions: M6-P1 scripts 완료 표시
- Next Recommendation: Cycle 3042-3043 — M6 dogfooding 현황 요약 + HANDOFF 업데이트
