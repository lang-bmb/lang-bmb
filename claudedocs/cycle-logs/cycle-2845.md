# Cycle 2845: {{ 이스케이프 + %=
Date: 2026-05-14

## Re-plan
Carry-Forward (2844): ① `{{` 이스케이프 ② `%=` 구현 권장. 계획 유효.

## Scope & Implementation

**접근법**: desugar_string_interp에 `{{`/`}}` 이스케이프 추가 + `%=` 토큰/문법 추가.

변경 파일:
- `bmb/src/ast/expr.rs`:
  - `{{` → 리터럴 `{` 이스케이프 (i += 2)
  - `}}` → 리터럴 `}` 이스케이프 (i += 2)
  - `args.is_empty()` 분기를 `Expr::StringLit(s)` → `Expr::StringLit(template)` 로 변경 (이스케이프 처리 결과 반영)
- `bmb/src/lexer/token.rs`: `ModEq` 토큰 추가 + Display impl
- `bmb/src/grammar.lalrpop`: `%=` extern 선언 + BlockStmt `%=` 규칙 추가
- `bmb/tests/integration.rs`:
  - `test_interp_compound_assign`에 `%=` 케이스 추가
  - `test_interp_string_interp_escape` 4개 케이스 추가
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - Compound assignment 패턴 섹션 추가
  - `{{`/`}}` 이스케이프 Pitfall 항목 추가

## Verification & Defect Resolution
- test_interp_string_interp_escape: 4/4 케이스 통과 ✅ (bskcui63p, 17m 55s 컴파일 후)
- test_interp_compound_assign: 8/8 케이스 통과 ✅ (+=/-=/*=//=/%=)
- desugar_string_interp unit tests: 8/8 통과 ✅
- cargo test --release 전체: **2369 passed** ✅ (1 flaky 수정 포함)

**추가 결함 발견 및 수정**:
4. `test_index_write_and_read` 병렬 실행 시 고정 temp 경로 충돌 flaky 버그 → unique suffix (`SystemTime::now().subsec_nanos()`) 적용으로 수정 (integration.rs:8909)

**결함 발견 및 수정**: 
1. 초기 테스트에서 `str_contains(s, "{literal}")` 사용 → `{literal}` 자체가 interpolation 대상이 됨 → `str_len` 기반으로 수정
2. `{{` 이스케이프 구현 후 `}}` 없어서 `{literal}}` (10자) 생성 → `}}` → `}` 이스케이프 추가하여 9자 정상 생성
3. `args.is_empty()` 반환값을 `Expr::StringLit(s)` → `Expr::StringLit(template)` 로 수정 (이스케이프 변환이 args 없이도 작동하도록)

## Reflection
- ✅ `{{`/`}}` 이스케이프: `"{{key}}: {val}"` → `"{key}: <value>"` 정상 동작.
- ✅ `%=` 연산자: 5개 복합 할당 연산자 완성.
- **중요 인사이트**: `args.is_empty()` 반환값을 원본 `s`가 아닌 `template`으로 수정해야 했음 — 이스케이프 처리가 args 생성 없이도 문자열을 변환하기 때문.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `set x.field += e` 필드 복합 할당 (현재 `set` + 단순 할당만 지원)
  * `{expr}` 복잡 표현식 지원 (현재 ident-only)
  * HashMap<String, Value> 지원 (현재 i64 key/value only)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: HashMap<String, Value> 지원 or bmb_reference 패턴 추가
