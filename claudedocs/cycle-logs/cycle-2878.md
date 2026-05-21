# Cycle 2878: 음수 리터럴 i32 narrow 근본 원인 분석 + runtime_param_type 완성
Date: 2026-05-15

## Re-plan
Carry-Forward: 음수 리터럴 i32 narrow 근본 수정 (Cycle 2876/2877 carry). 근본 원인 추적 + 방어적 수정.

## Scope & Implementation

### 근본 원인 분석 (Investigation):
1. `UnaryOp(Neg, Lit(3))` lowering: temp `_t24` → `func.locals.insert("_t24", I64)` (lower.rs:1063)
2. MIR 상수 폴딩: `_t24 = UnaryOp(Neg, 3)` → `_t24 = Const(-3)`
3. `ConstantPropagationNarrowing`: `_t24`가 func.locals에 I64로, Const(-3) 할당됨 → -3은 i32 범위 → `_t24` I32로 narrow
4. `build_place_type_map`: `_t24: "i32"` (func.locals에서), Const inst는 `is_declared_local=true`라 override 안 함
5. Call codegen: arg_ty = "i32", param expects i64 → runtime_param_type에 등록 없으면 sext 미생성 → UB

### 수정 (llvm_text.rs):
`runtime_param_type` 클로저에 누락된 함수 추가:
- `int_to_hex(n: i64)`, `int_to_bin(n: i64)` — 음수 hex/bin 표현 시 정확성 보장
- `hashmap_insert/get/contains/len/free(map: i64, key: i64, ...)` — hashmap i64 핸들/키 sext 보장
- `vec_clear(v: i64)` — vec 핸들 sext 보장

### 검증:
- `int_to_hex(-2)` interpreter="fffffffffffffffe", native="fffffffffffffffe" ✅ (기존 정상)
- `clamp_i64(-3, 0, 5)` = 0 ✅ (Cycle 2876 fix 유지)
- cargo test: 6249 PASS ✅

## Verification & Defect Resolution
- 6249 tests PASS (0 FAIL) ✅

## Reflection
- 근본 원인이 Cycle 2876에서 "원인 불명"이었지만 이번 사이클에서 완전히 규명
- `ConstantPropagationNarrowing` + `func.locals` i32 narrowing + `build_place_type_map` locals 우선 읽기 = 3-레이어 상호작용
- Cycle 2876 `runtime_param_type` 수정은 올바른 접근 (Proof: 정상 동작 확인)
- 방어적 추가 (hashmap/vec_clear): 실제 버그 트리거 케이스가 드물지만 안전망 확보

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. **필드 복합 할당 native 지원** — `set obj.field += e`가 llvm_text.rs에서도 동작하도록 (현재 interpreter-only)
  2. **str_char_at native 포팅** — String 반환 타입 처리 방법 연구 필요
  3. **runtime_param_type 장기 해결** — types/mod.rs의 함수 시그니처를 codegen에서 직접 참조하는 구조 (현재 중복 관리)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2879 — 필드 복합 할당 native 지원 (set obj.field += e)
