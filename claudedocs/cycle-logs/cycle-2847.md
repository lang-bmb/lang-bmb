# Cycle 2847: Field Compound Assignment (set obj.field += e)
Date: 2026-05-14

## Re-plan
Carry-Forward (2846): `set x.field += e` 필드 복합 할당, `{expr}` 복잡 표현식 보간, str_hashmap keys iterator.
필드 복합 할당 우선 처리.

## Scope & Implementation

**접근법**: grammar.lalrpop의 `BlockExpr` nonterminal에 `"set" SpannedUnaryExpr "+=" SpannedExpr` 계열 5개 규칙 추가.

**LR(1) 충돌 발견 및 수정**:
- 초기 시도: `BlockStmt`에 `"set" RawIdent "." Ident "+=" ...` 규칙 추가 → 빌드 실패
- 원인: `"set" RawIdent` 이후 `.` 토큰에서 shift-reduce 충돌
  - Shift: 새 필드 복합 할당 규칙 진행
  - Reduce: `RawIdent → Primary → ... → SpannedUnaryExpr` (기존 `"set" SpannedUnaryExpr "=" ...` 경로)
- 수정: `BlockExpr`에 `"set" SpannedUnaryExpr "+="` 규칙 추가 — `SpannedUnaryExpr` 파싱 완료 후 연산자 토큰(`+=` vs `=`)으로 구분하므로 충돌 없음

변경 파일:
- `bmb/src/grammar.lalrpop`: `BlockExpr`에 5개 규칙 추가 (`+=`, `-=`, `*=`, `/=`, `%=`)
  - `Expr::Var`: `Assign { value: Binary(Var(name), op, rhs) }` 로 desugar
  - `Expr::FieldAccess`: `FieldAssign { object, field, value: Binary(FieldAccess, op, rhs) }` 로 desugar
- `bmb/tests/integration.rs`: `test_interp_field_compound_assign` 6개 케이스 추가
  - `+=` 기본, `-=`, `*=`, `/=`, `%=`, 다중 필드 누적
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - "Pattern: Field compound assignment (v0.98.5+)" 신규 섹션
  - Common Pitfalls의 compound assignment 항목 갱신

## Verification & Defect Resolution
- test_interp_field_compound_assign: 6/6 케이스 통과 ✅
- cargo test --release 전체: **2371 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ 필드 복합 할당 5종 구현 완성 — `set obj.field += e` 등 모든 산술 복합 연산 지원
- ✅ LR(1) 충돌 패턴 문서화 — `BlockStmt`에서 prefix 공유 시 shift-reduce 발생; `BlockExpr`에서 파싱 완료 후 operator로 구분하는 방식이 정석
- **인사이트**: `"set" RawIdent "." Ident "+="` 형태의 BlockStmt 규칙은 LR(1) 한계 때문에 불가. `SpannedUnaryExpr` 전체를 파싱한 후 operator로 분기하는 패턴이 안전.
- 제한: interpreter-only (v0.98.5) — `bmb build` (native) 미지원. Desugar 결과가 `FieldAssign { value: Binary(...) }` 이므로 codegen이 지원하면 자동으로 동작할 것.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `{expr}` 복잡 표현식 지원 (현재 ident-only) — 인터폴레이션에서 `"{x + 1}"` 불가
  * str_hashmap keys iterator (현재 key 나열 방법 없음)
  * 필드 복합 할당 native 지원 (codegen 확장 — v0.98.5 이후)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `{expr}` 복잡 표현식 보간 (문자열 내 `"{x + 1}"` 지원)
