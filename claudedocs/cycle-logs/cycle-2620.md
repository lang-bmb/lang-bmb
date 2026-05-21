# Cycle 2620: M4-4 Static Method Call 구현
Date: 2026-05-10

## Re-plan
Plan valid. Cycle 2619 Carry-Forward: "M4-4 static method call 구현 시작".
STEP 1 설계: 아키텍처 조사에서 `parse_ident_or_call` 함수 내 `::` 처리 부분(line ~782)이 타깃.
단순 파서 변환으로 완성 가능하다고 판단 → 즉시 실행.

## Scope & Implementation

**변경 파일**: `bootstrap/compiler.bmb` lines 782-789

**핵심 변경**: `Name::method(args)` 처리 로직 추가
- `::` 다음 IDENT 파싱 후, 후행 토큰이 `TK_LPAREN()` 이면 static call로 처리
- `(call <Name_method> args)` AST 생성 — 이름 망글링: `Type::method` → `Type_method`
- `TK_LPAREN()` 없으면 기존 `(enum_variant <Name> <Variant>)` 유지

**새 테스트**: `tests/bootstrap/test_golden_static_method_call.bmb`
- `Math::double(21)` → `Math_double(21)` = 42
- `Math::add(40, 2)` → `Math_add(40, 2)` = 42
- `Math::square(7)` → `Math_square(7)` = 49
- 기존 unit enum variants 동시 테스트 (Color::Red/Green/Blue)
- 예상 출력: 139

**golden_tests.txt**: `test_golden_static_method_call.bmb|139` 추가

## Verification & Defect Resolution
- `bootstrap/compiler.exe run test_golden_static_method_call.bmb` → 139 ✅
- `bootstrap/compiler.exe run test_golden_enum_variant.bmb` → 206 ✅ (기존 회귀 없음)
- `bootstrap/compiler.exe run test_golden_enum_match.bmb` → 610 ✅ (기존 회귀 없음)
- `cargo nextest run --release` → 6210/6210 ✅
- Stage 1 bootstrap → OK (10923ms) ✅

## Reflection

**Scope fit**: M4-4 static method call 기본 구현 완료. LLM이 `Type::fn(args)` 패턴을 사용하면 bootstrap 컴파일러가 이제 `Type_fn(args)` 자유 함수로 처리한다.

**한계점 관찰**:
1. Rust 컴파일러는 이 문법을 지원하지 않는다 (`Math`가 enum이 아니면 타입 에러). 두 컴파일러 간 파티차 존재 — bootstrap-only 기능.
2. 이름 망글링 규칙(`::` → `_`)이 암묵적이다. 사용자가 함수를 `Type_method` 이름으로 직접 정의해야 함.
3. `Type::method()` 와 `Type::Variant()` 의 구분이 순전히 함수 존재 여부에 따름 — 타입 정보 없이 파서 수준에서 처리.

**Roadmap impact**: M4-4 완료. 다음: M4-3 (let-tuple). 

## Carry-Forward
- Actionable: Cycle 2621에서 M4-3 let-tuple destructuring 구현
- Structural Improvement Proposals: 이름 망글링 규칙 문서화 (사용자 혼란 방지)
- Pending Human Decisions: None
- Roadmap Revisions: M4-4 완료로 ROADMAP.md 상태 업데이트
- Next Recommendation: M4-3 let-tuple (파서 + MIR 변환 필요, 2 cycles 예상)
