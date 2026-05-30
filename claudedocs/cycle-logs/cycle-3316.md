# Cycle 3316: contracts-check forbid_function 규칙 (P4)
Date: 2026-05-30

## Re-plan
Cycle 3315 Carry-Forward에 따라 P4 forbid_function 구현. HANDOFF: "특정 함수 직접 호출 금지".

## Scope & Implementation
- `bc_check_forbid_fn(entries, forbidden_fn, sb, isfirst, pos)` 함수 추가
  - callers entries에서 forbidden_fn을 직접 호출하는 함수 탐지
  - `callers_calls_contain` 재사용으로 간결 구현
- `cc_build_json` 수정: `forbid_fn = bc_get(contracts, "forbid_function", 0)` + `bc_check_forbid_fn` 호출 (f3b)
- `.bmb-contracts` 에서 `forbid_function = fn_name` 형식 지원

## Verification & Defect Resolution
- cargo test: 3800+47+22+2390+23 = 6282 PASS, 0 FAILED ✅
- Stage 1 build (compiler_s1b.exe): 성공 ✅
- forbid_function 동작 확인: log_wrapper/bad_fn(2 violations), good_fn(통과) ✅
- Within-gen Fixed Point: fp3316a.ll == fp3316b.ll ✅

## Reflection
- 구현이 기존 forbid_effect 패턴과 일관적: 재사용성 높음
- callers_calls_contain으로 직접 호출만 탐지 (transitive 호출은 검사 안 함 — 의도된 설계)
- 향후: forbid_function_transitive가 필요하면 transitive_map 사용하는 별도 함수 추가

## Carry-Forward
- Actionable: P1 violations 형식 통일 (3-4 사이클)
- Structural Improvement Proposals: forbid_function_transitive (직접+간접 호출 금지 확장)
- Pending Human Decisions: None
- Roadmap Revisions: ROADMAP P4 forbid_function 완료 마킹 필요
- Next Recommendation: Cycle 3317 — P1 violations 형식 통일 시작
