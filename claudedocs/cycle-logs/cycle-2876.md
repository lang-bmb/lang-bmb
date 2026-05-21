# Cycle 2876: 정수 수학 함수 native 포팅 + P0 버그 수정
Date: 2026-05-15

## Re-plan
Plan valid. Cycle 2875에서 f64 수학 함수 완료. 이번 사이클: pow_i64/gcd_i64/clamp_i64/popcount native 포팅.

## Scope & Implementation

### C 런타임 추가 (`bmb/runtime/bmb_runtime.c`, L941 이후):
- `bmb_pow_i64(base, exp)` — 이진 거듭제곱 (exp < 0 → 0)
- `bmb_gcd_i64(a, b)` — 유클리드 알고리즘, 음수 입력 처리
- `bmb_clamp_i64(x, lo, hi)` — 단순 min/max 분기

### llvm_text.rs 추가:
- IR 선언: `bmb_pow_i64(i64, i64)`, `bmb_gcd_i64(i64, i64)`, `bmb_clamp_i64(i64, i64, i64)`, `bmb_popcount(i64)` 4종
- name mapping: pow_i64/gcd_i64/clamp_i64/popcount → bmb_* 형태
- infer_call_return_type: 4종 → i64 반환 등록
- **runtime_param_type: 4종 모두 i64로 등록** (P0 버그 수정 포함)

## P0 버그 발견 및 수정: 음수 리터럴 i32 narrowing

**증상**: `clamp_i64(-3, 0, 5)` native에서 5 반환 (올바른 값 0 대신)

**원인 추적**:
1. AST에서 `-3`은 `Unary(Neg, IntLit(3))`으로 파싱
2. MIR lowering에서 임시 변수 `_t24` 생성 (타입 I64)
3. MIR 최적화에서 `UnaryOp(Neg, Int(3))` → `Const(Int(-3))`로 폴딩
4. codegen에서 `_t24`의 place_type이 `i32`로 잘못 설정 (원인 불명)
5. → `alloca i32` 생성 → `store i32 -3` → `load i32` → `call bmb_clamp_i64(i32 %..., ...)` (잘못된 IR)
6. LLVM이 `i32 -3` (0xFFFFFFFD)를 zero-extend → 4294967293 → `clamp(4294967293, 0, 5) = 5`

**수정**: `runtime_param_type` 함수에 4종 integer math 함수 등록 → `arg_ty == "i32" && param_ty == "i64"` 조건 만족 → `sext i32 to i64` 자동 생성

**영향**: 이 버그는 음수 리터럴을 runtime 함수 인수로 전달하는 모든 경우에 영향. P0 수정 정당.

## Verification & Defect Resolution
- `tests/native_int_math_builtins.bmb`: `bmb run` = `bmb build` = `100` ✅
  - pow_i64(2,10)=1024 ✅, pow_i64(3,4)=81 ✅, pow_i64(5,0)=1 ✅
  - gcd_i64(12,8)=4 ✅, gcd_i64(100,75)=25 ✅
  - clamp_i64(10,0,5)=5 ✅, clamp_i64(-3,0,5)=0 ✅, clamp_i64(3,0,5)=3 ✅
  - popcount(7)=3 ✅, popcount(255)=8 ✅
- `cargo test --release`: 진행 중

## Reflection
- Scope fit: ✅ 4종 정수 수학 함수 native 포팅 + P0 음수 arg 버그 수정
- P0 버그: 음수 리터럴이 i32 alloca로 생성되는 근본 원인은 미파악이나, runtime_param_type 등록으로 실용적 수정
- popcount: `bmb_popcount` C 함수 기존 존재, IR 선언 + name mapping만으로 native 포팅 완료

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. **음수 리터럴 i32 narrow 근본 수정** — place_type이 i32로 잘못 설정되는 codegen 버그의 근본 원인 파악 및 수정 (현재 runtime_param_type으로 우회 처리 중)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2877 — HANDOFF/ROADMAP 정리 + 커밋, 또는 str_split/str_lines svec 반환 native 포팅 (복잡)
