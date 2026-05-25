# Cycle 3158: M9 Batch 24 — get_int/string/float_text/get_child/read_sexp/get_pipe_name/get_field/rename/get_exit_label 15개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. String 반환 함수군으로 전환.

## Scope & Implementation
15개 post 조건 추가:

**Group A: `post it.len() <= X.len()` — 입력에서 부분 추출 (10개)**
- `get_int_text(s, pos, tok)` — s.slice(...) ≤ s.len()
- `get_string_text(s, pos, tok)` — s.slice(p+1, endpos-1) ≤ s.len()
- `get_float_text(s, pos, tok)` — s.slice(...) ≤ s.len()
- `get_child(ast, idx)` — extract_paren_content 후 슬라이스 ≤ ast.len()
- `get_child_at(content, pos, idx)` — read_sexp_at 결과 ≤ content.len()
- `read_sexp_at(s, pos)` — s.slice(...) ≤ s.len()
- `get_pipe_name(names, idx)` — names.slice(...) ≤ names.len()
- `get_pipe_name_at(names, pos, idx)` — names.slice(...) ≤ names.len()
- `get_field(item, idx)` — item.slice(...) ≤ item.len()
- `get_field_at(item, pos, idx)` — item.slice(...) ≤ item.len()

**Group B: `post it.len() >= 0` — 변환/빌드 함수 (4개)**
- `rename_name_in_ast_at` — acc 누적 (len ≥ 0)
- `get_exit_label` — 라벨 문자열 반환 (len ≥ 0)
- `scan_exit_children` — 라벨 문자열 반환 (len ≥ 0)
- `build_tuple_bindings` — (let ...) 중첩 문자열 (len ≥ 0)

**Group C: `post it.len() > 0` — 항상 비어있지 않은 반환 (1개)**
- `method_to_runtime_fn` — 모든 분기 non-empty 반환 (최소 "bmb_abs" 등)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- missing_postcondition: 478 → **463** (−15)

## Reflection
- String 반환 함수에 `post it.len() <= X.len()` 패턴 확립
- trivially-true `post it.len() >= 0` 도 BMB verifier 수용
- 로드맵 영향: 없음

## Carry-Forward
- Actionable: missing_postcondition 463개 — llvm_gen_* 계열 + collect_*/scan_*/build_* + 기타
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→463 (−351 총계, 43.1% 감소)
- Next Recommendation: Cycle 3159: collect_lambda_params/get_lambda_body/scan_mir_for_free_vars/check_*/format_*/emit_fill_stores + llvm_gen_* 15개
