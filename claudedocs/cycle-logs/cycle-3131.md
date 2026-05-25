# Cycle 3131: M8-A bool trivial 7개 교체 (is_string_fn_group1-6 + is_builtin_double_fn)
Date: 2026-05-25

## Re-plan
Plan valid. M8-B String trivial 완전 완료 (77개 skip 확정) → M8-A bool trivial 계속.
이전 세션 분석: 32개 bool trivial 잔여 중 7개 name-lookup 함수 교체 가능.

## Scope & Implementation
7개 교체 — `post it or not it` → `post it == (name == "X" or ...)` 정확한 열거 계약:

**Name-lookup 그룹 (7개)**:
- is_builtin_double_fn(name) L17208 → 27개 f64 math 함수 열거
- is_string_fn_group1(name) L17253 → 16개 core string 함수 열거
- is_string_fn_group2(name) L17262 → 10개 string builder/parser 함수 열거
- is_string_fn_group3(name) L17269 → 9개 gen/parse 함수 열거
- is_string_fn_group4(name) L17277 → 13개 llvm_gen 함수 열거
- is_string_fn_group5(name) L17285 → 20개 llvm_gen/step/util 함수 열거
- is_string_fn_group6(name) L17296 → 24개 bmb_string_*/exec/svec 함수 열거

**비교체 분류 (25개 잔여 주요 skip 이유)**:
- cf_can_fold/cf_can_simplify/ube_can_simplify: substring 검색 포함 (contains, starts_with) — 정확 열거 불가
- is_float_expr/is_pure_expr/is_trivial_value: 재귀적 구조 검사 — 이름 매칭 아닌 트리 탐색
- dce_var_in_rhs/dce_fn_has_side_effect: IR 패턴 분석 — 복합 조건
- has_annotation/is_annotated: find 계열 호출 포함 — 동적 조건
- is_ir_pure_param/needs_sret: type string 분석 — starts_with/contains 기반
- cont_is_exit/cont_is_entry: label 패턴 분석

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 3020 → 3013 (−7)
- bmb verify ✅ 953/953

## Reflection
- Bool trivials: 32 → 25 (−7 교체 확인)
- is_string_fn_group1-6 + is_builtin_double_fn: 순수 name == "X" or ... 형태라 Z3 trivially 검증
- warnings -7 (vs 편집 7개): 1:1 정확히 매핑 — 각 함수가 독립 semantic_trivial 경고였음
- 25개 잔여는 구조적으로 skip — substring/recursive/pattern 기반 술어

## Carry-Forward
- Actionable: 잔여 25개 bool trivial 전수 분석 완료 → 모두 skip 확정
  - cf_can_fold/simplify 계열: contains("phi") 등 substring 기반 → 정확 열거 불가
  - is_float_expr/is_pure_expr: 재귀 트리 탐색 → 계약으로 표현 불가
  - 전환: M8-C(it 타입 고정) 또는 i64 trivial 완성으로 방향 전환
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M8-A bool trivial 실질 완료 — 잔여 25개는 skip 확정
- Next Recommendation: Cycle 3132: i64 trivial 7개 분석 + 교체 가능 항목 적용
