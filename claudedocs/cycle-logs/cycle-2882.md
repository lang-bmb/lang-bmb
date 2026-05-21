# Cycle 2882: while let native 포팅 + enum match 로컬 ptr 버그 수정
Date: 2026-05-15

## Re-plan
Carry-Forward: 없음. Cycle 2881 Next: interpreter-only 재검증 또는 for-in-vec 조사.
for-in-vec → native에서 channel recv로 오역되어 복잡. while let → MIR에서 Unit no-op으로 처리됨(native silent bug).
범위: while let native 포팅.

## Scope & Implementation

### STEP 1 진단:
- `while let` → lower.rs: `Expr::WhileLet { .. } => Operand::Constant(Constant::Unit)` (no-op)
- native 빌드는 성공하지만 결과 0 (interpreter는 10) → silent correctness bug
- `match` with function-return enum → `local ptr type` Switch emit 버그 발견
  - 증상: `'%_t2.disc_wlet_exit_2' defined with type 'ptr' but expected 'i64'`
  - 원인: local에서 ptr type 변수 switch 시 `.addr`에서 load → ptr 반환 → switch expects i64

### STEP 2 수정:
**lower.rs** (line 1252) — `Expr::WhileLet` → actual MIR lowering:
  - cond block: lower scrutinee + Switch terminator (same as Match)
  - body block: bind_pattern_variables + lower body + Goto(cond)
  - exit block: Constant::Unit
  - loop_context_stack push/pop for break/continue

**llvm_text.rs** (Switch terminator, line 8744-8747) — local ptr type fix:
  - When `local_ty == "ptr"`: two-level load
    1. `load ptr, ptr %p.addr` → enum pointer
    2. `load i64, ptr %enum_ptr` → tag
  - Previously: single `load ptr, ptr %p.addr` → ptr used as i64 → type error

### 검증:
- while let `Opt::Some(v) = decr(count)`: interp=10, native=10 ✅
- while let immediate exit (n=0): interp=0, native=0 ✅
- `match` with function-return enum (Opt): interp=2, native=2 ✅ (bonus fix)
- cargo test: 6249 PASS (0 FAIL) ✅

### bmb_reference.md 수정:
- line 30: `interpreter-only` → `native v0.98.9+`
- line 922: while let 항목을 interpreter-only 목록에서 제거

## Verification & Defect Resolution
- interpreter: 10 ✅
- native: 10 ✅
- cargo test: 6249 PASS (0 FAIL) ✅

## Reflection
- Scope fit: while let 포팅 성공. 추가로 enum match local ptr 버그 수정 (연관 결함).
- bonus fix: `match result_from_fn { Opt::Some(v) => ... }` 패턴도 이제 native에서 동작.
- for-in-vec: channel recv 방식으로 MIR에서 처리됨 — 이는 더 큰 변경 필요. 현재 범위 외.
- Philosophy drift: 없음. `while let` native 지원은 언어 완성도 향상에 직접 기여.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. **struct pass-by-value HUMAN-decision** — 스펙 침묵, 의미론 결정 필요
  2. **for-in-vec native 포팅** — 현재 channel recv 방식으로 MIR 처리됨, 벡터 이터레이션 전용 MIR 구현 필요
  3. **format() native 포팅** — varargs C 함수 + 템플릿 치환 로직 필요, 복잡도 높음
  4. **string interpolation native 포팅** — format()에 의존
  5. **to_string(bool) native** — bmb_bool_to_string("true"/"false") C 함수 추가
  6. **runtime_param_type 장기 해결** — types/mod.rs 시그니처 직접 참조
- Pending Human Decisions:
  - struct parameter semantics (값 vs 참조)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2883 — to_string(bool) native 포팅 또는 format() 조사
