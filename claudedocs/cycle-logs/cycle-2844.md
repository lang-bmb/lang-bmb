# Cycle 2844: 복합 할당 연산자 구현 (+=, -=, *=, /=)
Date: 2026-05-14

## Re-plan
Carry-Forward (2843): `+=`/`-=` 복합 할당 연산자 구현 권장. 계획 유효.

## Scope & Implementation

**접근법**: 파서(grammar.lalrpop) 수준에서 desugar — `x += e` → `Expr::Assign { name: x, value: Expr::Binary { Add, Var(x), e } }`.
- 기존 `Expr::Assign` / `Expr::Binary` 인프라 완전 재사용
- 인터프리터/타입체커/코드젠 변경 없음
- 4개 연산자: `+=`, `-=`, `*=`, `/=`

변경 파일:
- `bmb/src/lexer/token.rs`: `PlusEq`, `MinusEq`, `MulEq`, `DivEq` 토큰 추가 + Display impl
- `bmb/src/grammar.lalrpop`: extern 토큰 선언 + `BlockStmt` 4개 규칙 추가 (각 연산자별 desugar)
- `bmb/tests/integration.rs`: `test_interp_compound_assign` 7개 케이스 추가

## Verification & Defect Resolution
- cargo test --release: ✅ ALL PASS (3774 unit + 2368 integration = 6142 total)
- test_interp_compound_assign: 7/7 케이스 통과

## Reflection
- ✅ 4개 복합 할당 연산자 모두 구현. `set s = s + x` → `s += x` 패턴 이제 가능.
- 파서 수준 desugar → 타입체커/인터프리터 자동 처리 (string interpolation과 동일 패턴).
- for 루프 누적 패턴(`for i in 0..5 { s += i; }`)이 자연스럽게 동작.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `%=` 모듈로 할당 (현재 미구현, 드물게 사용)
  * `set x.field += e` 형식 (필드 복합 할당, 현재 `set` + 단순 할당만 지원)
  * `while let` 패턴의 bmb_reference 문서화 보강
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `%=` 추가 또는 다음 M4 언어 갭 (HashMap, 구조체 개선 등) 검토
