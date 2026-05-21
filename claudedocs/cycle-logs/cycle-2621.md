# Cycle 2621: M4-3 Let-Tuple Destructuring 구현
Date: 2026-05-10

## Re-plan
Plan valid. M4-3 let-tuple 구현. Cycle 2620 Carry-Forward 이행.

## Scope & Implementation

**조사**: 
- bootstrap/compiler.bmb에 두 곳의 let 파싱 경로 존재:
  1. `parse_let_expr` (표현식 컨텍스트, 함수 body 등)
  2. `parse_block_let` (블록 컨텍스트, `{...}` 내부)
- 두 곳 모두에 `TK_LPAREN()` 처리 추가 필요
- 숫자 필드명 접근 이미 지원됨 (line 14453: is_digit 분기)

**데슈가링 전략**:
`let (a, b) = expr; body`
→ `(let <__tup_P> expr_ast (let <a> (field (var <__tup_P>) 0) (let <b> (field (var <__tup_P>) 1) body_ast)))`

**변경 파일**: `bootstrap/compiler.bmb`
- `parse_let_expr`: `k1 == TK_LPAREN()` 분기 추가
- `parse_block_let`: `tok_kind(t1) == TK_LPAREN()` 분기 추가
- 새 함수 추가:
  - `parse_let_tuple_pattern` / `parse_tuple_names_acc` / `parse_tuple_names_sep` / `parse_let_tuple_eq` (표현식 컨텍스트)
  - `parse_block_let_tuple` / `parse_block_tuple_names_acc` / `parse_block_tuple_names_sep` / `parse_block_tuple_eq` (블록 컨텍스트)
  - `build_tuple_bindings` / `get_pipe_name` / `get_pipe_name_at` (공통)
  - 파이프 구분 이름 목록 인코딩: `"a|b|c"`

**새 테스트**: `tests/bootstrap/test_golden_let_tuple.bmb`
- `let (a, b) = make_pair()` — 함수 반환 튜플 destructuring
- `let (x, y, z) = make_triple()` — 3-원소 destructuring
- `let (p, q) = (20, 22)` — 인라인 튜플 literal destructuring
- 예상 출력: 42

**golden_tests.txt**: `test_golden_let_tuple.bmb|42` 추가

## Verification & Defect Resolution
- `bootstrap/compiler.exe run test_golden_let_tuple.bmb` → 42 ✅
- `bootstrap/compiler.exe run test_golden_enum_variant.bmb` → 206 ✅
- `bootstrap/compiler.exe run test_golden_enum_match.bmb` → 610 ✅
- `bootstrap/compiler.exe run test_golden_static_method_call.bmb` → 139 ✅
- `cargo nextest run --release` → 6210/6210 ✅
- Stage 1 bootstrap → OK ✅

## Reflection

**Scope fit**: let-tuple destructuring 완전 구현. 두 파싱 경로 모두 처리.

**발견**: 
- 블록 컨텍스트 파싱 경로(parse_block_let)가 표현식 컨텍스트(parse_let_expr)와 별도 관리됨. 새 let 기능 추가 시 두 곳 모두 수정 필요 — CLAUDE.md Rule 2에 문서화 가치 있음.
- 파이프(`|`) 구분 이름 목록은 명확하지만, 이름에 파이프 문자가 포함되면 안됨 (일반 식별자에서 불가능하므로 안전).

**Roadmap impact**: M4-3 완료. 다음: M4-5 Option::Some(x) 표현식 지원 평가.

## Carry-Forward
- Actionable: Cycle 2622에서 M4-5 enum 페이로드 표현식 지원 실현 가능성 조사
- Structural Improvement Proposals: CLAUDE.md Rule 2에 "블록 컨텍스트 let 파싱 경로 별도 존재" 주의사항 추가 제안
- Pending Human Decisions: None
- Roadmap Revisions: M4-3 완료로 ROADMAP 업데이트
- Next Recommendation: M4-5 scope 분석 후 진행 여부 결정 (Cycle 2622)
