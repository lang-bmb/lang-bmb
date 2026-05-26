# Cycle 3199: non_snake_case lint SCREAMING_SNAKE_CASE 예외 (−108)
Date: 2026-05-27

## Re-plan
Plan valid. 상속 범위: non_snake_case 108개 lint 수정 (ALL_UPPER_CASE 예외 처리).

## Scope & Implementation

**원인 분석**:
- `bmb/src/util.rs:is_snake_case()`: 코드 내 주석 "FOO_BAR (unless all-caps which we allow)"가 있었으나 구현이 누락됨
- TK_FN, SEP, DSEP 등 모든 대문자 상수 함수가 non_snake_case 경고 발생

**수정 (bmb/src/util.rs)**:
- `is_snake_case()` 에 SCREAMING_SNAKE_CASE 예외 추가:
  ```rust
  if check.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
      return true;
  }
  ```
- doc comment 정확화: "Valid (SCREAMING_SNAKE_CASE): `FOO_BAR`, `TK_FN`, `SEP`"

**테스트 수정 (bmb/src/util.rs)**:
- `test_is_snake_case`: `assert!(!is_snake_case("FOO_BAR"))` → `assert!(is_snake_case("FOO_BAR"))` (의도된 동작)
- `test_is_snake_case_leading_underscore_numbers`: `assert!(!is_snake_case("A"))` → `assert!(is_snake_case("A"))`

## Verification & Defect Resolution
- `cargo test --release`: 6278 tests ✅ (3796+47+22+2390+23)
- `bmb check bootstrap/compiler.bmb`: non_snake_case **0** ✅, semantic_duplication 1016, 총 1017
- Stage 1 bootstrap: 이전 사이클에서 확인 ✅

## Reflection
- **Scope fit**: non_snake_case 108개 완전 소거 달성
- **구현의 의도 vs 실제**: `is_snake_case` 코드 주석에 "all-caps 허용" 의도가 명시되어 있었으나 구현에 반영되지 않은 것 — 단순 구현 누락 수정
- **SCREAMING_SNAKE_CASE 허용 범위**: 순수 대문자+숫자+밑줄. 혼합(fooBAR)은 여전히 경고
- **non_snake_case 완전 해소**: M10 Track B ✅ COMPLETE (non_snake_case 108→0)

## Carry-Forward
- Actionable: semantic_duplication 1016개 처리 — Cycle 3200부터 클러스터별 접근
- Structural Improvement Proposals: None
- Pending Human Decisions: TK_BREAK=TK_AS=127, TK_LOOP=TK_BXOR=131 토큰 ID 충돌 의도 확인
- Roadmap Revisions: M10 Track B ✅ COMPLETE
- Next Recommendation: Cycle 3200 — skip_to_eol(65) + scan_int(65) 클러스터 postcondition 개선
