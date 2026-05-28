# Cycle 3238: M11-C Phase 4 — `tape[i]` subscript for `[u8; N]` stack arrays

Date: 2026-05-28

## Re-plan

Plan valid: Inherited from Cycle 3237 Carry-Forward.
- u8 subscript (`tape[i]`): `@stack_u8_new` + i8 GEP stride + i8 load/zext — 이번 사이클 범위

## Scope & Implementation

### 설계 (이전 세션에서 완료)

**핵심 접근**: Phase 3 (`[i64;N]`)와 동일한 post-parse rewrite 전략 + 새 MIR 명령어
- `TK_U8_ELEM()` 가상 토큰 (2000000000+168): `u8` IDENT를 `@stack_u8_new` 경로로 구별
- `rewrite_stack_u8_index`: 함수 body AST에서 `@stack_u8_new` 바인딩 탐색 → `(index ...)` → `(ptr_index_u8 ...)`
- 새 MIR 명령어: `gep_u8` (i8 stride GEP), `load_ptr_u8` (load i8 + zext i64), `store_ptr_u8` (trunc i64→i8 + store)
- DSA 안전: `alloca [N x i8]` → DSA `"= alloca i64"` 패턴과 무충돌
- 디스패치 순서: `gep_u8`/`load_ptr_u8`/`store_ptr_u8`을 `gep`/`load_ptr`/`store_ptr`보다 앞에 배치 (prefix 충돌 방지)

### `bootstrap/compiler.bmb` 변경사항 (16개)

1. **`TK_U8_ELEM()` 함수 추가** (2000000000+168) — u8 원소 타입용 가상 토큰
2. **`parse_block_let_array_type_aware`**: `k1 == TK_IDENT() and text == "u8"` → `TK_U8_ELEM()` 승격
3. **`parse_block_let_after_stack_array`**: `elem_kind == TK_U8_ELEM()` → `"(call <stack_u8_new> ...")`
4. **`fn_returns_ptr`**: `@stack_u8_new` 추가 (ptr 반환 빌트인 목록)
5. **`get_call_arg_types`**: `"@stack_u8_new" => "i"` 추가
6. **`llvm_gen_call`**: `@stack_u8_new` inline codegen (`alloca [N x i8] + memset(N) + ptrtoint`)
7. **`lower_ptr_index_u8_sb`**: 새 함수 (gep_u8 + load_ptr_u8 lowering)
8. **`lower_ptr_set_index_u8_sb`**: 새 함수 (gep_u8 + store_ptr_u8 lowering)
9. **`rewrite_stack_u8_index`**: 새 함수 (body AST 스캔 → ptr_index_u8 재작성)
10. **`lower_function_sb`**: `rewrite_stack_u8_index(body_ast_i64, 0)` 호출 추가
11. **`lower_expr_sb`**: `ptr_index_u8`/`set_ptr_index_u8` 디스패치 추가
12. **`step_expr`**: `ptr_index_u8`/`set_ptr_index_u8` with "U" flag 추가
13. **`step_array_index_final`**: `else if flag == "U"` 분기 추가 (gep_u8 + load_ptr_u8)
14. **`step_set_index_final`**: `else if flag == "U"` 분기 추가 (gep_u8 + store_ptr_u8)
15. **LLVM gen 함수 3개 추가**: `llvm_gen_gep_u8_sb`, `llvm_gen_load_ptr_u8_sb`, `llvm_gen_store_ptr_u8_sb`
16. **RHS/Statement 디스패치**: `gep_u8`/`load_ptr_u8`/`store_ptr_u8`을 기존 패턴 앞에 배치

### 골든 테스트
`tests/bootstrap/test_golden_stack_u8_subscript.bmb`:
```bmb
fn main() -> i64 = {
    let tape: [u8; 10];
    let mut i = 0;
    while i < 5 { set tape[i] = i * i; i = i + 1 };
    let sum = tape[0] + tape[1] + tape[2] + tape[3] + tape[4];
    println(sum)
};
```
Expected output: `30` (0²+1²+2²+3²+4²)

## Verification & Defect Resolution

### 골든 테스트 결과

- Stage 1 빌드 ✅
- `test_golden_stack_u8_subscript.bmb` → 30 ✅
- LLVM opt IR: `tail call void @println(i64 30)` — 전체 상수 폴딩 ✅

### 3-Stage Bootstrap

- Stage 1 (`compiler_3238_s1.exe`): ✅
- Stage 2 (`compiler_3238_s2.exe`): ✅ (~24s compile + ~13s link)
- Stage 3 (`compiler_3238_s3.exe`): ✅ (~23s compile + ~12s link)
- **Fixed Point: S3 IR == S4 IR (0 diff)** ✅

### cargo test

✅ 3800+47+22+2390+23 tests (same as before, Rust-level tests unchanged)

### bootstrap/compiler.exe

`compiler_3238_s2.exe` → `bootstrap/compiler.exe` 복사 완료 ✅

## Reflection

### Scope fit ✅
M11-C Phase 4 목표 달성. `tape[i]` subscript for `[u8; N]` stack arrays 완전 구현.

### 이중 Lowering 경로 준수 ✅
`lower_ptr_index_u8_sb` (recursive: `lower_expr_sb`) + `step_array_index_final` "U" 분기 (iterative: `step_expr`) 양쪽 모두 처리. CLAUDE.md Rule 3 준수.

### 디스패치 순서 ✅
`gep_u8` < `gep`, `load_ptr_u8` < `load_ptr`, `store_ptr_u8` < `store_ptr` 순서 보장.
`low_starts_with_at`이 prefix 매칭이므로 u8 변형을 먼저 확인해야 함.

### DSA 안전성 ✅
`alloca [N x i8]` 형식 — DSA `"= alloca i64"` 패턴과 무충돌 (i64가 아닌 i8).
`@stack_bytes_new`의 `alloca i8, i64 N` 형식과도 다름 (배열 형식 사용).

### LLVM 최적화 ✅
`alloca [N x i8]` + memset + getelementptr inbounds nuw i8 + zext/trunc 패턴을 LLVM이 완전 상수 폴딩.
컴파일 타임에 모든 배열 연산을 제거하고 `println(30)` 직접 호출로 최적화.

### Architecture soundness ✅
Phase 4는 Phase 3 (`rewrite_stack_i64_index`)와 정확히 동일한 패턴. 코드 중복 없음.
brainfuck tape (`[u8; 30000]`)에 직접 적용 가능 — 기존 `load_u8`/`store_u8` 대신 `tape[ptr]`/`set tape[ptr]` 사용 가능.

## Carry-Forward

- **Actionable**:
  - brainfuck tape 최적화: `main_inproc.bmb`의 `load_u8(tape + ptr)` → `tape[ptr]` 교체 (Phase 4 활용)
  - 다음 언어 갭 해소 작업 (ROADMAP.md § M11 확인)

- **Structural Improvement Proposals**:
  - `@stack_bytes_new`는 `alloca i8, i64 N` 형식 사용 (배열 형식 아님). DSA `= alloca i64` 패턴은 체크하지 않지만, 향후 `alloca [N x i8]` 형식으로 통일 가능 (선택적).

- **Pending Human Decisions**: None

- **Roadmap Revisions**:
  - ROADMAP.md M11-C Phase 4: "미래 스코프" → "✅ COMPLETE (Cycle 3238)"

- **Next Recommendation**:
  1. brainfuck tape 최적화: `tape[ptr]`/`set tape[ptr]` 문법 적용 + P-track 성능 재측정
  2. 기타 언어 갭 해소 (ROADMAP.md § M11 확인)
  3. 다른 M11-C 확장: `[i32; N]` subscript (4-byte stride)?
