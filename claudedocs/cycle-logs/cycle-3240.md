# Cycle 3240: M11-C Phase 6 — `arr[i]` subscript for `[bool; N]` stack arrays

Date: 2026-05-28

## Re-plan

Plan valid: Inherited from Cycle 3239 Carry-Forward.
- Scope: complete stack array subscript story with `[bool; N]` (1-byte stride, same as u8)
- Phase 3 covers i64/f64 (8-byte), Phase 4 covers u8 (1-byte), Phase 5 covers i32 (4-byte), Phase 6 adds bool (1-byte)

## Scope & Implementation

### 설계

**핵심 접근**: Phase 4 (`[u8; N]`)와 동일한 post-parse rewrite 전략 + 새 MIR 명령어
- `@stack_bool_new` 빌트인: `alloca [N x i8]` + `memset(N)` + `ptrtoint` → i64 ptr (u8과 동일)
- `rewrite_stack_bool_index`: 함수 body AST에서 `@stack_bool_new` 바인딩 탐색 → `(index ...)` → `(ptr_index_bool ...)`
- 새 MIR 명령어: `gep_bool` (i8 stride GEP), `load_ptr_bool` (load i8 + **zext** i64), `store_ptr_bool` (trunc i64→i8 + store)
- bool은 `TK_BOOL()` 토큰 사용 (u8처럼 가상 토큰 불필요 — TK_BOOL이 기본 토큰)
- load는 `zext` (unsigned) — bool 0/1은 부호 없는 값, u8과 동일 선택

### `bootstrap/compiler.bmb` 변경사항 (14개)

1. **`parse_block_let_after_stack_array`**: `elem_kind == TK_BOOL()` → `"(call <stack_bool_new> ..."` 추가
2. **`fn_returns_ptr`**: `@stack_bool_new` 추가
3. **`get_call_arg_types`**: `"@stack_bool_new" => "i"` 추가
4. **`llvm_gen_call`**: `@stack_bool_new` inline codegen (`alloca [N x i8] + memset(N) + ptrtoint`) 추가
5. **`rewrite_stack_bool_index`**: 새 함수 (body AST 스캔 → ptr_index_bool 재작성)
6. **`lower_function_sb`**: `body_ast_i32` 중간 변수 도입 + `rewrite_stack_bool_index` 호출
7. **`lower_expr_sb`**: `ptr_index_bool`/`set_ptr_index_bool` 디스패치 추가
8. **`step_expr`**: `ptr_index_bool`/`set_ptr_index_bool` with "B" flag 추가
9. **`step_array_index_final`**: `else if flag == "B"` 분기 추가 (gep_bool + load_ptr_bool)
10. **`step_set_index_final`**: `else if flag == "B"` 분기 추가 (gep_bool + store_ptr_bool)
11. **`lower_ptr_index_bool_sb`**: 새 함수 (recursive: lower_expr_sb)
12. **`lower_ptr_set_index_bool_sb`**: 새 함수 (recursive: lower_expr_sb)
13. **LLVM gen 함수 3개 추가**: `llvm_gen_gep_bool_sb`, `llvm_gen_load_ptr_bool_sb`, `llvm_gen_store_ptr_bool_sb`
14. **Statement/RHS 디스패치**: `gep_bool`/`load_ptr_bool`/`store_ptr_bool`를 기존 패턴 앞에 배치

### 골든 테스트
`tests/bootstrap/test_golden_stack_bool_subscript.bmb`:
```bmb
fn main() -> i64 = {
    let flags: [bool; 4];
    let mut i = 0;
    while i < 4 { set flags[i] = true; i = i + 1 };
    set flags[1] = false;
    let b0 = if flags[0] { 1 } else { 0 };
    let b1 = if flags[1] { 1 } else { 0 };
    let b2 = if flags[2] { 1 } else { 0 };
    let b3 = if flags[3] { 1 } else { 0 };
    println(b0 + b1 + b2 + b3)
};
```
Expected output: `3` (true+false+true+true = 3)

## Verification & Defect Resolution

### 골든 테스트 결과

- Stage 1 빌드 ✅ (32G arena)
- `test_golden_stack_bool_subscript.bmb` → 3 ✅
- `test_golden_stack_u8_subscript.bmb` → 30 ✅ (Phase 4 regression check)
- `test_golden_stack_i32_subscript.bmb` → 30 ✅ (Phase 5 regression check)
- LLVM opt IR: `tail call void @println(i64 3)` — 전체 상수 폴딩 ✅

### 3-Stage Bootstrap

- Stage 1 (`compiler_3240_s1.exe`): ✅ (~42s compile + ~14s link)
- Stage 2 (`compiler_3240_s2.exe`): ✅ (~26s compile + ~13s link)
- Stage 3 (IR 검증): ✅ (~26s compile + ~13s link)
- **Fixed Point: S3 IR == S4 IR (0 diff)** ✅

### cargo test

✅ 3800+47+22+2390+23 tests (unchanged)

### bootstrap/compiler.exe

`compiler_3240_s2.exe` → `bootstrap/compiler.exe` 복사 완료 ✅

## Reflection

### Scope fit ✅
M11-C Phase 6 목표 달성. `arr[i]` subscript for `[bool; N]` stack arrays 완전 구현.

### 이중 Lowering 경로 준수 ✅
`lower_ptr_index_bool_sb` (recursive: `lower_expr_sb`) + `step_array_index_final` "B" 분기 (iterative: `step_expr`) 양쪽 모두 처리. CLAUDE.md Rule 3 준수.

### zext (unsigned) 선택 ✅
- `load_ptr_bool` → `zext i8 to i64` (bool 0/1은 unsigned: u8과 동일)
- `load_ptr_i32` → `sext i32 to i64` (i32는 signed)
bool 값은 0과 1만 가능하므로 zext가 올바름.

### TK_BOOL() 직접 사용 ✅
`bool` 키워드는 `TK_BOOL()` 기본 토큰이므로 u8처럼 가상 토큰(TK_U8_ELEM) 불필요.
`parse_block_let_after_stack_array`의 `TK_BOOL()` 분기를 직접 추가.

### DSA 안전성 ✅
`alloca [N x i8]` 형식 — DSA `"= alloca i64"` 패턴과 무충돌.

### LLVM 최적화 ✅
`alloca [N x i8]` + memset + getelementptr inbounds nuw i8 + zext 패턴을 LLVM이
완전 상수 폴딩. `println(3)` 직접 호출로 최적화.

### Stack Array Subscript 완성 ✅
| 타입 | Phase | MIR 명령어 | GEP stride | 확장 |
|------|-------|-----------|-----------|------|
| `[i64; N]`, `[f64; N]` | Phase 3 | gep/load_ptr/store_ptr | i64 (8B) | — |
| `[u8; N]` | Phase 4 | gep_u8/load_ptr_u8/store_ptr_u8 | i8 (1B) | zext |
| `[i32; N]` | Phase 5 | gep_i32/load_ptr_i32/store_ptr_i32 | i32 (4B) | sext |
| `[bool; N]` | Phase 6 | gep_bool/load_ptr_bool/store_ptr_bool | i8 (1B) | zext |

## Carry-Forward

- **Actionable**:
  - 다음 언어 갭 해소 작업 (ROADMAP.md § M11-C 방향)

- **Structural Improvement Proposals**:
  - `lower_ptr_index_u8_sb`, `lower_ptr_index_bool_sb` 코드 중복: 동일 구조, 다른 MIR 명령어. 향후 `generic_lower_ptr_index_sb(op_name)` 헬퍼로 통합 가능.
  - 마찬가지로 3종 LLVM gen 함수 (gep_u8/bool, load_ptr_u8/bool, store_ptr_u8/bool) 유사 구조. 단순 패턴이므로 현재는 허용.

- **Pending Human Decisions**: None

- **Roadmap Revisions**:
  - ROADMAP.md M11-C Phase 6: 신규 항목 추가 "✅ COMPLETE (Cycle 3240)"

- **Next Recommendation**:
  1. `[f32; N]` subscript (Phase 7): 4-byte stride, f64 double 타입 저장이 아닌 float 저장 — 별도 고려 필요
  2. 다음 언어 갭: `match` with guards (`if` guards in match arms)
  3. 또는 Generic functions in bootstrap compiler
