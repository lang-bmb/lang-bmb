# Cycle 3125: M8-B 배치 2 — String trivial 19개 교체 (substring/slice 패턴)
Date: 2026-05-25

## Re-plan
Plan valid. M8-B 계속 — 5516-9000 라인 범위의 substring/extraction/concat 패턴 교체.

## Scope & Implementation
19개 교체:

**Substring/slice 패턴 (it.len() <= input.len())**:
- filter_out_locals(free_vars, locals) L5782 → free_vars
- extract_string_content(ast) L6415 → ast
- block_inner(ast) L6737 → ast
- seq_first(ast), seq_second(ast) L6759/6768 → ast
- assign_lhs_expr(ast), assign_rhs_expr(ast) L7316/7325 → ast
- extract_var_from_assign(ast), extract_for_varname_from_ast(ast) L7335/7133 → ast
- get_fn_return_type(ast), get_fn_body(ast) L7459/7496 → ast
- extract_phi_val(content) L7680 → content
- unpack_conv(pair), unpack_arg(pair) L8800/8806 → pair
- call_arg_conversions(formatted), call_arg_formatted(formatted) L8818/8823 → formatted
- trim_end(s), trim(s) L8840/8849 → s (핵심 유틸리티 함수!)

**Exact concat**:
- pack_conv_arg(conv, arg) L8795 → `post it.len() == conv.len() + 3 + arg.len()`

## Verification & Defect Resolution
- cargo test --release ✅ (6255 tests)
- bmb check ✅ warnings: 3092 → 3087 (−5)
- bmb verify ✅ 954/954
- 발견: missing_postcondition 815개 = pre-only 함수들 (M8 스코프 밖)
- 발견: semantic_trivial 경고 없음 — `post it.len() >= 0`는 다른 메커니즘으로 집계됨

## Reflection
- String trivials: 256 → 237 (−19 교체 확인)
- 경고 감소가 -19가 아닌 -5인 이유: semantic_duplication 쌍이 많아 하나 제거 시 부분적 감소
- trim_end, trim 등 핵심 유틸리티 함수의 length-bounded 계약 적용 완료

## Carry-Forward
- Actionable: 나머지 237개 String trivial 분석 계속 (다음 배치)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3126: 9160-13900 범위 분석
