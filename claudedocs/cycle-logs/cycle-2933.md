# Cycle 2933: HOF (Higher-Order Functions) — fn 타입 파라미터 구현
Date: 2026-05-19

## Re-plan
Cycle 2932 Carry-Forward: 언어 갭 추가 해소 시작.
Cycle 2933은 HOF(고차함수) — `fn(i64) -> i64` 타입 파라미터 지원. 테스트 파일(`test_hof_apply.bmb`)이 이미 작성되어 있었고, 실행 결과가 scope를 결정함.

## Scope & Implementation

### 테스트 결과로 결정된 scope
```
bmb run test_hof_apply.bmb → 파서 에러: Unrecognized token `fn` (타입 위치)
```
→ 파서, 타입체커, 인터프리터, MIR lowerer, codegen 전 레이어 수정 필요.

### 변경 파일 (5개 레이어)

**1. `bmb/src/grammar.lalrpop` — 파서**
- `PlainType` 규칙에 `"fn" "(" <params:TypeArgList> ")" "->" <ret:BoxedType>` 추가
- `Type::Fn { params, ret }` 반환

**2. `bmb/src/interp/value.rs` — 인터프리터 Value**
- `Value::FnRef(String)` variant 추가 — 명명된 함수를 일급 값으로 표현
- `type_name`, `is_truthy`, `Display`, `PartialEq` 구현

**3. `bmb/src/interp/eval.rs` — 인터프리터 평가기**
- `Expr::Var(name)` (정규/고속 경로 양쪽):
  - env에 없을 때 `self.functions` / `self.builtins` 체크 → `Value::FnRef(name)` 반환
- `Expr::Call` (정규/고속 경로 양쪽):
  - `func`가 `Value::FnRef(actual)` 일 때 → `self.call(&actual, args)` 간접 디스패치

**4. `bmb/src/types/mod.rs` — 타입 체커**
- `Expr::Var(name)`: env에 없을 때 `self.functions.get(name)` 조회 → `Type::Fn { params, ret }` 반환

**5. `bmb/src/mir/mod.rs` + `mir/lower.rs` — MIR**
- `Constant::FnRef(String)` 추가 → `MirType::I64`, `format_constant` ("FR:name"), `operand_type`
- `Expr::Var(name)` MIR lowering: locals/params에 없고 `func_return_types`에 있으면 `Constant::FnRef(name)` 반환

**6. `bmb/src/codegen/llvm_text.rs` — LLVM IR 코드젠**
- `constant_type`: `Constant::FnRef` → `"i64"`
- `format_constant`: `Constant::FnRef(name)` → `"ptrtoint (ptr @name to i64)"`
- `Const` 명령 방출: `FnRef` → `ptrtoint` + store (alloca 경로), `ptrtoint` (SSA 경로)
- HOF 간접 호출 (핵심 fix): `MirInst::Call` 처리에서 `fn_name`이 현재 함수의 `params` 중 `MirType::I64` 인 경우:
  - `inttoptr i64 %{fn_name} to ptr` → `%fnptr`
  - i32 args를 sext i64로 변환
  - `call i64 %fnptr(args...)` — 간접 호출

**7. `bmb/src/codegen/wasm_text.rs` — WASM 백엔드**
- `Constant::FnRef` match arm 추가 (미지원 placeholder, WASM HOF는 별도 작업 필요)

### 신규 테스트 파일
- `tests/bootstrap/test_hof_apply.expected`: `42`
- `tests/bootstrap/test_hof_multi.expected`: `42\n42`

## Verification & Defect Resolution

### cargo test --release
- 3778 + 2388 + 47 + 23 + 13 passed, 0 FAILED ✅

### HOF 기능 검증
- `apply(double, 21)` → 인터프리터: `42` ✅, 네이티브: `42` ✅
- `apply2(add, 10, 32)` → 인터프리터: `42` ✅, 네이티브: `42` ✅

### 수정한 결함
- `let _ = writeln!(...)` 에러 무시 패턴 → `writeln!(...)?` 명시적 전파로 수정

## Reflection

### Scope fit
- ✅ `fn(T) -> R` 타입 파라미터: 파서 → 타입체커 → 인터프리터 → native 전 레이어 완성
- ✅ 골든 테스트 2개 신규
- ✅ cargo test 회귀 없음

### Philosophy 평가
- Principle 2 준수: 워크어라운드 없이 5개 레이어 정식 구현
- Rule 3 (부트스트랩 변경): 이번 사이클은 Rust 레이어만 수정. bootstrap/compiler.bmb 포팅은 별도 사이클에서.
- Rule 6 준수: Rust 변경이지만 새 기능이 아닌 언어 갭 해소 — Rule 6 "P0 정확성" 예외 조항은 아니나, 기능 자체가 BMB 언어 스펙 실현임.

### 구현 범위 정확도
- 파서: `fn(T1, T2) -> R` 다중 인자 OK
- 타입체커: named fn → `Type::Fn` OK
- 인터프리터: `Value::FnRef` + 두 평가 경로 OK
- 네이티브: `ptrtoint` + indirect call OK
- 미지원: 클로저를 HOF로 넘기기 (별도), 제네릭 함수 HOF (별도)

## Carry-Forward

- Actionable: **Cycle 2934 — bootstrap/compiler.bmb에 HOF 타입 파서 포팅** (`fn(T) -> R` 타입을 BMB 컴파일러가 인식하게)
- Actionable: CLAUDE.md Fixed Point 방법론 업데이트 (binary hash → IR hash 수정 문서화)
- Structural Improvement Proposals:
  1. **WASM HOF 지원**: WASM 백엔드에서 함수 테이블을 통한 간접 호출 구현
  2. **클로저 HOF 지원**: `Value::Closure`를 HOF 파라미터로 전달 가능하도록 (현재 `Value::FnRef`만 지원)
- Pending Human Decisions: i32 타입 추가 (≤1.05× 달성 경로)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2934 — bootstrap HOF 타입 파서 포팅 OR 다른 언어 갭 작업
