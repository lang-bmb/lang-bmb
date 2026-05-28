# Cycle 3242: M11-C Phase 7 — `arr[i]` subscript for `[f64; N]` stack arrays

Date: 2026-05-28

## Re-plan

Plan valid: Carried from Cycle 3241 Carry-Forward.
- Scope: `[f64; N]` stack array subscript (Phase 7)
- Root cause: `[f64; N]` was using `@stack_i64_new` (shared with i64), generating invalid IR:
  - Stores: `store double 1.5, ptr %gep` (accidentally correct — double_marker in store path)
  - Loads: missing `load double` temps → `add nsw i64 1.5, 2.5` (invalid LLVM IR)
- Fix: Implement separate `@stack_f64_new` pipeline (same pattern as Phase 4/5/6)

## Scope & Implementation

### 설계

**핵심 접근**: Phase 4 (`[u8; N]`) / Phase 6 (`[bool; N]`)과 동일한 post-parse rewrite 전략 + 새 MIR 명령어

- `@stack_f64_new` 빌트인: `alloca [N x double]` + memset(N*8) + `ptrtoint` → i64 ptr
- `rewrite_stack_f64_index`: 함수 body AST에서 `@stack_f64_new` 바인딩 탐색 → `(index ...)` → `(ptr_index_f64 ...)`
- 새 MIR 명령어: `gep_f64` (double stride GEP), `load_ptr_f64` (load double + push_double_marker), `store_ptr_f64` (store double directly)
- `load_ptr_f64` 결과: double_marker 추가 → 이후 `+` binop이 `fadd double` 생성

### `bootstrap/compiler.bmb` 변경사항 (14개)

1. **`parse_block_let_after_stack_array`**: `TK_I64() or TK_F64()` → 분리: `TK_I64()` → `@stack_i64_new`, `TK_F64()` → `@stack_f64_new`
2. **`fn_returns_ptr`**: `@stack_f64_new` 추가
3. **`get_call_arg_types`**: `"@stack_f64_new" => "i"` 추가
4. **`llvm_gen_call`**: `@stack_f64_new` inline codegen (`alloca [N x double] + mul*8 + memset + ptrtoint`) 추가
5. **`rewrite_stack_f64_index`**: 새 함수 (body AST 스캔 → ptr_index_f64 재작성)
6. **`lower_function_sb`**: `body_ast_bool` 중간 변수 도입 + `rewrite_stack_f64_index` 호출
7. **`lower_expr_sb`**: `ptr_index_f64`/`set_ptr_index_f64` 디스패치 추가
8. **`step_expr`**: `ptr_index_f64`/`set_ptr_index_f64` with "F" flag 추가
9. **`step_array_index_final`**: `else if flag == "F"` 분기 추가 (gep_f64 + load_ptr_f64)
10. **`step_set_index_final`**: `else if flag == "F"` 분기 추가 (gep_f64 + store_ptr_f64)
11. **`lower_ptr_index_f64_sb`**: 새 함수 (recursive: lower_expr_sb)
12. **`lower_ptr_set_index_f64_sb`**: 새 함수 (recursive: lower_expr_sb)
13. **LLVM gen 함수 3개 추가**: `llvm_gen_gep_f64_sb`, `llvm_gen_load_ptr_f64_sb`, `llvm_gen_store_ptr_f64_sb`
14. **Statement/RHS 디스패치**: `gep_f64`/`load_ptr_f64`/`store_ptr_f64`를 기존 패턴 앞에 배치

### 골든 테스트

`tests/bootstrap/test_golden_stack_f64_subscript.bmb` (→ golden):
```bmb
fn main() -> i64 = {
    let arr: [f64; 3];
    set arr[0] = 1.5;
    set arr[1] = 2.5;
    set arr[2] = 3.0;
    let sum = arr[0] + arr[1] + arr[2];
    println(sum as i64)
};
```
Expected output: `7` (1.5+2.5+3.0 = 7.0 → i64 7)

## Verification & Defect Resolution

### 골든 테스트 결과

- Stage 1 빌드 ✅ (32G arena)
- `test_golden_stack_f64_subscript.bmb` → 7 ✅
- `test_golden_stack_bool_subscript.bmb` → 3 ✅ (Phase 6 regression check)
- `test_golden_stack_u8_subscript.bmb` → 30 ✅ (Phase 4 regression check)
- `test_golden_stack_i32_subscript.bmb` → 30 ✅ (Phase 5 regression check)
- LLVM opt IR: `tail call void @println(i64 7)` — 전체 상수 폴딩 ✅

### 생성된 LLVM IR (test_golden_stack_f64_subscript.ll)

```llvm
%_t1_arr = alloca [3 x double], align 8
...
%_t5 = getelementptr inbounds nuw double, ptr %_t1_arr, i64 0
store double 1.5, ptr %_t5, align 8
...
%_t19 = getelementptr inbounds nuw double, ptr %_t1_arr, i64 0
%_t20 = load double, ptr %_t19, align 8
...
%_t25 = fadd double %_t20, %_t24
...
%_t30 = fadd double %_t25, %_t29
...
%_t32 = fptosi double %_t31 to i64
```

### 3-Stage Bootstrap

- Stage 1 (`compiler_3242_s1.exe`): ✅ (~2s Rust compile + S1 build)
- Stage 2 (`compiler_3242_s2.exe`): ✅ (~27s compile + ~13s link)
- Stage 3 (IR 검증): ✅ (~25s compile + ~13s link)
- **Fixed Point: S3 IR == S4 IR (0 diff)** ✅

### cargo test

✅ 3800+47+22+2390+23 tests (unchanged)

### bootstrap/compiler.exe

`compiler_3242_s2.exe` → `bootstrap/compiler.exe` 복사 완료 ✅

## Reflection

### Scope fit ✅
M11-C Phase 7 목표 달성. `arr[i]` subscript for `[f64; N]` stack arrays 완전 구현.
이전의 잘못된 공유 (`@stack_i64_new` for f64) 해소 — 전용 `@stack_f64_new` 파이프라인 구축.

### 이중 Lowering 경로 준수 ✅
`lower_ptr_index_f64_sb` (recursive: `lower_expr_sb`) + `step_array_index_final` "F" 분기 (iterative: `step_expr`) 양쪽 모두 처리. CLAUDE.md Rule 3 준수.

### double_marker 연동 ✅
- `load_ptr_f64` → `load double` + `push_double_marker` → 이후 `+` binop이 자동으로 `fadd double` 생성
- `store_ptr_f64` → val은 이미 double temp (float 리터럴이 `fadd double 0.0, 1.5` 형태) → `store double` 직접 생성
- 별도의 bitcast/conversion 불필요

### alloca [N x double] 형식 ✅
DSA `"= alloca i64"` 패턴과 충돌 없음. 이전의 `alloca [N x i64]`에서 double로 변경 — GEP stride가 double (8B)로 올바르게 설정.

### LLVM 최적화 ✅
`alloca [3 x double]` + memset + `getelementptr inbounds nuw double` + `load double` + `fadd double` + `fptosi` 패턴을 LLVM이 완전 상수 폴딩. `println(7)` 직접 호출로 최적화.

### Stack Array Subscript 완성 (Phase 7 추가)
| 타입 | Phase | MIR 명령어 | GEP stride | 확장 |
|------|-------|-----------|-----------|------|
| `[i64; N]` | Phase 3 | gep/load_ptr/store_ptr | i64 (8B) | — |
| `[u8; N]` | Phase 4 | gep_u8/load_ptr_u8/store_ptr_u8 | i8 (1B) | zext |
| `[i32; N]` | Phase 5 | gep_i32/load_ptr_i32/store_ptr_i32 | i32 (4B) | sext |
| `[bool; N]` | Phase 6 | gep_bool/load_ptr_bool/store_ptr_bool | i8 (1B) | zext |
| `[f64; N]` | Phase 7 | gep_f64/load_ptr_f64/store_ptr_f64 | double (8B) | double_marker |

## Carry-Forward

- **Actionable**:
  - `*u8`/`*i32`/`*bool`/`*f64` 파라미터 타입별 stride subscript (typed pointer parameter subscript)
  - 다음 언어 갭 해소 작업 (ROADMAP.md § M11-C 방향)

- **Structural Improvement Proposals**:
  - Phase 4-7의 rewrite_stack_*_index 함수들이 동일 구조 (needle만 다름). 향후 `generic_rewrite_stack_index(body, pos, builtin_name, node_name)` 헬퍼로 통합 가능.
  - 마찬가지로 3종 LLVM gen 함수 그룹 (gep/load/store)도 타입별 패턴. 현재는 유지 — 명시성 우선.

- **Pending Human Decisions**: None

- **Roadmap Revisions**:
  - ROADMAP.md M11-C: Phase 7 `[f64; N]` subscript 항목 추가 "✅ COMPLETE (Cycle 3242)"

- **Next Recommendation**:
  1. `match` with guards (`if` guards in match arms) 언어 갭
  2. 또는 `*u8`/`*i32`/`*bool`/`*f64` typed pointer parameter subscript (Phase 8)
  3. 또는 [f32; N] subscript (4-byte float stride — unique, no existing parallel)
