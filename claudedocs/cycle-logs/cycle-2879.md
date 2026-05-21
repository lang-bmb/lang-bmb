# Cycle 2879: 필드 복합 할당 native 지원 — 진단 + 문서 수정
Date: 2026-05-15

## Re-plan
Carry-Forward: 없음. Cycle 2878 Next Recommendation: 필드 복합 할당 native 지원 (`set obj.field += e`).

## Scope & Implementation

### STEP 1 진단:
- 그래머 확인: `set obj.field += e` → `Expr::FieldAssign { value: Binary { left: FieldAccess, op: Add, right } }` 로 desugar
- MIR lowering: `FieldAssign` → `FieldAccess` inst + `BinOp` inst + `FieldStore` inst (lower.rs:2081)
- llvm_text.rs: `MirInst::FieldStore` + `MirInst::FieldAccess` 모두 이미 구현됨 (line 6991, 6931)

### 실제 상태 (STEP 2 실험):
- 로컬 변수 compound: `set st.total += 100` → **interpreter=302, native=302** ✅ 이미 동작
- 파라미터 compound: `fn bump(c: Counter) -> i64 = { set c.val += 5; c.val }` → **interp=15, native=15** ✅

### 발견된 P0 버그 (별도 이슈):
- `fn modify(s: Stats, v: i64)` 내에서 `set s.total = v` → native는 원본 struct를 수정함
- 원인: heap-allocated struct를 call site에서 복사 없이 포인터로 전달 (pass-by-reference)
- 기댓값: BMB는 pass-by-value → callee 수정이 caller에 전파되어선 안 됨
- interpreter: 정상 (pass-by-value) / native: 버그 (pass-by-reference)
- 측정: `add_sample(st, 10)` → interp는 st 불변, native는 `st.total += 10` 적용됨

### 수정 (bmb_reference.md):
- "Pattern: Field compound assignment" 헤더 → "interpreter-only" 제거, "native-supported v0.98.9+" 추가
- 패턴 예시를 로컬 변수 전용으로 단순화 (파라미터 케이스 제거)
- Common Pitfalls: "interpreter-only" → "native-supported v0.98.9+ for local struct vars" 수정

### 새 테스트:
- `tests/native_field_compound.bmb`: Point + Counter 두 struct, 5종 연산 확인 → interp=325, native=325 ✅

## Verification & Defect Resolution
- interpreter: 325 ✅
- native build: 325 ✅
- cargo test: 6249 PASS (0 FAIL) ✅

## Reflection
- Scope fit: 조사 결과 "interpreter-only"가 잘못된 레이블임이 확인됨. 로컬 변수 케이스는 이미 동작.
- P0 버그 발견: struct 파라미터 pass-by-value 위반 — native가 포인터 전달로 callee 변경이 caller에 영향. 이는 compound assignment 자체 버그가 아닌 broader struct 전달 방식 문제.
- Philosophy drift: 없음. bmb_reference는 정확한 정보를 제공해야 하므로 수정이 적절.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. **struct pass-by-value P0 버그** — native에서 struct 파라미터를 포인터로 전달, callee 변경이 caller에 전파됨. 수정: call site에서 memcpy(heap alloca → new alloca) 후 전달. 복잡도: 중간. 영향: 모든 struct 파라미터 뮤테이션 코드.
  2. **str_char_at native 포팅** — String 반환 타입 처리 연구 필요 (Cycle 2877 carry)
  3. **runtime_param_type 장기 해결** — types/mod.rs 시그니처 직접 참조 구조
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2880 — struct pass-by-value P0 버그 수정 (call site memcpy) 또는 str_char_at native 포팅
