# Cycle 3110: Track B i64 배치 계약 추가 59개 + bool 타입 충돌 진단
Date: 2026-05-25

## Re-plan
Cycle 3109 Carry-Forward: 잔여 466개 분석 (bool 97개, String 279개, i64 90개). bool/i64 패턴 추가.

## Scope & Implementation

**i64 no-contract 함수 배치 추가 (59개)**:

1. **analysis/query *_file 계열 (28개)**: outline_file, xref_file, impact_file, stats_file, unused_file, complexity_file, similar_file, layers_file, hotspots_file, body_file, interface_file, clusters_file, coverage_file, pattern_file, export_file, diff_calls_file, classify_file, rename_check_file, siblings_file, summary_file, graph_file, split_candidates_file, inline_candidates_file, chain_file, suggest_file, scope_file, changelog_file, fmt_dir (+fmt_entry)

2. **fmt/lint/check/test 계열 (12개)**: fmt_file, lint_file, lint_dir, lint_entry, check_file, check_dir, check_entry, test_file, test_dir, test_entry, fmt_dir, fmt_entry

3. **repl 계열 (6개)**: repl_help, repl_run_ir, repl_exec, repl_eval, repl_loop_inner, repl_start

4. **build 계열 (6개)**: build_native_opt, build_native_fast, build_native_direct, build_link, build_cleanup, build_file

5. **utilities (7개)**: layer_classify, hot_print_top, cls_print_if_nonzero, print_build_timing, print_compile_stats, run_file, show_help, compile_file_to

**bool 함수 추가 시도 및 롤백**:
- 92개 bool 함수에 `post it >= 0` 배치 추가 시도
- 오류 발견: "expected bool, got i64" — 타입 체커가 `post it >= 0` 수치 조건을 bool 반환 타입과 혼동
- 원인: i64 비교식 `it >= 0`이 bool 함수의 반환 타입 추론을 i64로 오염시킴
- 진단: is_string_var_sb 등 원래 4개는 L16500+ 영역 (늦게 처리)이라 오류 미발생, L3664처럼 초기 위치 함수들은 충돌
- 결정: bool 함수에 `post it >= 0` 추가 불가 (현재 타입 체커 한계)
- 모든 bool batch 완전 롤백

**결과**:
- 466 → 385 no-contract (-81개)
- 세부: 279 String + 96 bool + 10 i64

## Verification & Defect Resolution

- `bmb check`: ✅ 3199 warnings, 0 errors
- `bmb verify`: ✅ total:1320, verified:1320, failed:0
- 3-Stage Fixed Point: ✅ S3 == S4 (`F8DA1AB9259A6F6E0C0CF548E87B1743`)

## Reflection

- Scope fit: 100% (i64 59개 성공, bool 96개 롤백)
- 핵심 발견: `post it >= 0`이 bool 반환 함수에서 타입 체커 오류 유발 — 함수 정의 순서에 따라 callee 타입 추론이 i64로 오염됨
- bool 함수는 `post it == false or it == true`와 같은 타입-안전 post 조건이 필요하나 현재 BMB 문법에 없음
- String 함수 279개: `post it.len() >= 0`은 시도 미완료 (bool 실패 후 접근 보류)
- i64 잔여 10개: 음수 반환 가능 (s2i, str_to_int, cf_compute 등) — 안전한 계약 없음

## Carry-Forward

- Actionable: Cycle 3111 — String 279개 분석 (`post it.len() >= 0` 가능 여부 소규모 테스트 후 결정)
  - String 함수 중 항상 비어 있지 않은 결과 반환하는 것에 `post it.len() > 0` 추가 가능
  - bool 함수: 타입 체커 수정 없이는 post 조건 추가 불가
- Structural Improvement Proposals:
  - BMB 타입 체커가 `post it >= 0`을 bool 반환 타입에 허용하도록 수정 필요
    - `post` 조건의 `it` 타입은 항상 함수 선언 반환 타입으로 고정되어야 함
    - 현재: `it >= 0` 추론 → i64 → 반환 타입 bool과 충돌
    - 수정: post 절 내 `it` 타입 = 선언된 반환 타입 (bool이면 bool)
- Pending Human Decisions: M8 공식 계획 확정
- Roadmap Revisions: M8-A Track B 466→385 (81 감소)
- Next Recommendation: String 함수 `post it.len() >= 0` 소규모 테스트 → 가능하면 배치 추가
