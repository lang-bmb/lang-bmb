# Cycle 2842: String Interpolation 구현 (M4-11)
Date: 2026-05-14

## Re-plan
Carry-Forward (2841): M4-11 String interpolation 다음. 계획 유효.

## Scope & Implementation

**접근법**: 파서(grammar.lalrpop) 수준에서 desugar — `"Hello {name}"` → `format("Hello {0}", name)`.
- 기존 `format()` 인프라 완전 재사용
- 렉서 변경 없음
- 새 AST 노드 없음

변경 파일:
- `bmb/src/ast/expr.rs`: `pub fn desugar_string_interp(s: String) -> Expr` 추가 (50 LOC)
  - `{identifier}` 패턴 스캔 + 번호 치환 + `Expr::Call { func: "format", ... }` 생성
  - 패턴 없으면 `Expr::StringLit(s)` 반환 (기존 동작)
- `bmb/src/grammar.lalrpop`: Primary `"string" =>` 규칙을 `desugar_string_interp(<>)` 호출로 변경
- `bmb/tests/integration.rs`: `test_interp_string_interpolation` 6개 케이스 추가

**지원 패턴**: `{identifier}` — ASCII 문자/언더스코어로 시작, 영숫자/언더스코어 포함.
**미지원**: `{0}` 숫자 패턴 (그대로 보존), `{expr}` 복잡 표현식.

## Verification & Defect Resolution
- cargo test --release: ✅ 6141+ tests ALL PASS (3774 unit + 2367 integration)
- test_interp_string_interpolation: 6/6 케이스 통과

## Reflection
- ✅ M4-11 String interpolation 완료. `desugar_string_interp`로 깔끔한 구현.
- 파서 수준 desugar → 타입 체커/인터프리터 자동 처리 (format() 재사용).
- `{0}` 숫자 패턴 보존 = 기존 format() 코드와 호환.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `{{` → `{` 이스케이프 지원 (현재 미지원, 중괄호 리터럴 표현 불가)
  * `{expr}` 복잡 표현식 지원 (현재 ident-only)
  * 미정의 변수 참조 시 컴파일 에러 vs 런타임 에러 (현재 런타임)
- Pending Human Decisions: None
- Roadmap Revisions: M4-11 ✅ 표시 예정 (ROADMAP 갱신 포함)
- Next Recommendation: bmb_reference 패턴 추가 (string interpolation 패턴) + ROADMAP 갱신
