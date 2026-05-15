# Cycle 2855: {fn_call(args)} 보간 구현
Date: 2026-05-15

## Re-plan
Carry-Forward (2854): `{fn_call(args)}` 보간 — InterpMini 함수 호출 파싱 추가. Plan valid, inherited scope.

## Scope & Implementation

**InterpMini 확장** (Cycle 2855, interpreter-only):
- `primary()`: ident 후 `(` 발견 시 `call_args()` 호출 → `Expr::Call` 생성
- `call_args()`: `,`-구분 인수 목록 파싱, `)` 로 종료
- 인수 각각은 `self.expr()` 재귀 호출 → 중첩 함수 호출도 지원

예시: `"{to_string(x)}"` → parse_time에 `Expr::Call { func: "to_string", args: [Var("x")] }` 생성 → format call로 desugar → eval 시 실행

변경 파일:
- `bmb/src/ast/expr.rs`: `InterpMini::call_args()` 신규 + `primary()` 함수 호출 분기
- `bmb/tests/integration.rs`: `test_interp_string_interp_fn_call` (3케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: interpolation notes 갱신

## Verification & Defect Resolution
- test_interp_string_interp_fn_call: 3/3 통과 ✅
  - `"{to_string(n)}"` → "result: 42" ✅
  - `"{str_to_upper(s)}"` → "upper: HELLO" ✅
  - `"{to_string(x * 2)}"` → "doubled: 10" ✅
- cargo test --release 전체: **2380 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ `{fn_call(args)}` 보간 완성 — 중첩 가능 (args 내 다시 expr)
- ✅ `consume()` 메서드는 여전히 dead_code 경고 — 기능에 영향 없으나 정리 가능
- 이전 docs: "NOT function calls" 제거 → "function calls {fn(args)} supported" 업데이트

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * 필드 복합 할당 native 지원 (codegen)
  * InterpMini `consume()` dead_code 경고 정리 (minor)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: 수학 builtins 확장 또는 `for x in svec {}` (Value::SvecHandle 타입)
