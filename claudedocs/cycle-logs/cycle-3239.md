# Cycle 3239: M11-C Phase 5 — `arr[i]` subscript for `[i32; N]` stack arrays

Date: 2026-05-28

## Re-plan

Plan valid: Inherited from Cycle 3238 Carry-Forward.
- Scope: complete stack array subscript story with `[i32; N]` (4-byte stride)
- Phase 3 covers i64/f64 (8-byte), Phase 4 covers u8 (1-byte), Phase 5 adds i32 (4-byte)

## Scope & Implementation

### 설계

**핵심 접근**: Phase 4 (`[u8; N]`)와 동일한 post-parse rewrite 전략 + 새 MIR 명령어
- `@stack_i32_new` 빌트인: `alloca [N x i32]` + `mul N,4` + `memset(N*4)` + `ptrtoint` → i64 ptr
- `rewrite_stack_i32_index`: 함수 body AST에서 `@stack_i32_new` 바인딩 탐색 → `(index ...)` → `(ptr_index_i32 ...)`
- 새 MIR 명령어: `gep_i32` (i32 stride GEP), `load_ptr_i32` (load i32 + sext i64), `store_ptr_i32` (trunc i64→i32 + store)
- i32는 `TK_I32()` 토큰 사용 (u8과 달리 가상 토큰 불필요)
- load는 `sext` (signed), u8은 `zext` (unsigned) — 중요 차이

### `bootstrap/compiler.bmb` 변경사항 (14개)

1. **`parse_block_let_after_stack_array`**: `elem_kind == TK_I32()` → `"(call <stack_i32_new> ..."` (이전 fallback에서 분리)
2. **`fn_returns_ptr`**: `@stack_i32_new` 추가
3. **`get_call_arg_types`**: `"@stack_i32_new" => "i"` 추가
4. **`llvm_gen_call`**: `@stack_i32_new` inline codegen (`alloca [N x i32] + mul N,4 + memset(N*4) + ptrtoint`)
5. **`rewrite_stack_i32_index`**: 새 함수 (body AST 스캔 → ptr_index_i32 재작성)
6. **`lower_function_sb`**: `body_ast_u8` 중간 변수 도입 + `rewrite_stack_i32_index` 호출
7. **`lower_expr_sb`**: `ptr_index_i32`/`set_ptr_index_i32` 디스패치 추가
8. **`step_expr`**: `ptr_index_i32`/`set_ptr_index_i32` with "32" flag 추가
9. **`step_array_index_final`**: `else if flag == "32"` 분기 추가 (gep_i32 + load_ptr_i32)
10. **`step_set_index_final`**: `else if flag == "32"` 분기 추가 (gep_i32 + store_ptr_i32)
11. **`lower_ptr_index_i32_sb`**: 새 함수 (recursive: lower_expr_sb)
12. **`lower_ptr_set_index_i32_sb`**: 새 함수 (recursive: lower_expr_sb)
13. **LLVM gen 함수 3개 추가**: `llvm_gen_gep_i32_sb`, `llvm_gen_load_ptr_i32_sb`, `llvm_gen_store_ptr_i32_sb`
14. **Statement/RHS 디스패치**: `gep_i32`/`load_ptr_i32`/`store_ptr_i32`를 기존 패턴 앞에 배치

### 골든 테스트
`tests/bootstrap/test_golden_stack_i32_subscript.bmb`:
```bmb
fn main() -> i64 = {
    let arr: [i32; 5];
    let mut i = 0;
    while i < 5 { set arr[i] = i * i; i = i + 1 };
    let sum = arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
    println(sum)
};
```
Expected output: `30` (0²+1²+2²+3²+4²)

## Verification & Defect Resolution

### 골든 테스트 결과

- Stage 1 빌드 ✅ (32G arena 필요 — 기존 S2 컴파일러 한계)
- `test_golden_stack_i32_subscript.bmb` → 30 ✅
- `test_golden_stack_u8_subscript.bmb` → 30 ✅ (Phase 4 regression check)
- LLVM opt IR: `tail call void @println(i64 30)` — 전체 상수 폴딩 ✅

### 3-Stage Bootstrap

- Stage 1 (`compiler_3239_s1.exe`): ✅
- Stage 2 (`compiler_3239_s2.exe`): ✅ (~24s compile + ~12s link)
- Stage 3 (`compiler_3239_s3.exe`): ✅ (~24s compile + ~12s link)
- **Fixed Point: S3 IR == S4 IR (0 diff)** ✅

### cargo test

✅ 3800+47+22+2390+23 tests (unchanged)

### bootstrap/compiler.exe

`compiler_3239_s2.exe` → `bootstrap/compiler.exe` 복사 완료 ✅

### 부가 발견: test_golden_stack_u8_subscript.bmb 누락

Cycle 3238에서 생성한 골든 테스트 `.bmb` 소스 파일이 git에 추가되지 않았음.
이번 사이클에서 재생성 + 커밋 포함.

## Reflection

### Scope fit ✅
M11-C Phase 5 목표 달성. `arr[i]` subscript for `[i32; N]` stack arrays 완전 구현.

### 이중 Lowering 경로 준수 ✅
`lower_ptr_index_i32_sb` (recursive: `lower_expr_sb`) + `step_array_index_final` "32" 분기 (iterative: `step_expr`) 양쪽 모두 처리. CLAUDE.md Rule 3 준수.

### signed vs unsigned 확장 ✅
- `load_ptr_u8` → `zext i8 to i64` (u8은 unsigned: 부호 없음)
- `load_ptr_i32` → `sext i32 to i64` (i32은 signed: 부호 연장)
이 차이가 정확성에 중요.

### DSA 안전성 ✅
`alloca [N x i32]` 형식 — DSA `"= alloca i64"` 패턴과 무충돌.

### TK_I32() 직접 사용 ✅
i32는 `TK_I32()` 기본 토큰이므로 u8처럼 가상 토큰(TK_U8_ELEM) 불필요.
`parse_block_let_after_stack_array`의 기존 i32 분기를 그냥 재활용.

### Arena 메모리 32G 필요
이번 사이클부터 S1 빌드에 32G arena가 필요. `bootstrap/compiler.exe`가 35K+ LOC
로 성장하면서 컴파일 시 메모리 사용량이 증가. 기존 8G/16G 한계를 초과.
> `BMB_ARENA_MAX_SIZE=32G` 빌드 스크립트/workflow 업데이트 권고.

### LLVM 최적화 ✅
`alloca [N x i32]` + memset + getelementptr inbounds nuw i32 + sext 패턴을 LLVM이
완전 상수 폴딩. `println(30)` 직접 호출로 최적화.

### Stack Array Subscript 완성 ✅
| 타입 | Phase | MIR 명령어 | GEP stride | 확장 |
|------|-------|-----------|-----------|------|
| `[i64; N]`, `[f64; N]` | Phase 3 | gep/load_ptr/store_ptr | i64 (8B) | — |
| `[u8; N]` | Phase 4 | gep_u8/load_ptr_u8/store_ptr_u8 | i8 (1B) | zext |
| `[i32; N]` | Phase 5 | gep_i32/load_ptr_i32/store_ptr_i32 | i32 (4B) | sext |

## Carry-Forward

- **Actionable**:
  - 빌드 스크립트에 `BMB_ARENA_MAX_SIZE=32G` 명시 (S1 빌드 OOM 방지)
  - 다음 언어 갭 해소 작업 (ROADMAP.md § M11-C 방향)

- **Structural Improvement Proposals**:
  - `[bool; N]` subscript — 1-byte stride (u8과 동일 GEP), zext로 i64, Phase 6 가능

- **Pending Human Decisions**: None

- **Roadmap Revisions**:
  - ROADMAP.md M11-C Phase 5: 신규 항목 추가 "✅ COMPLETE (Cycle 3239)"

- **Next Recommendation**:
  1. 언어 갭 추가 해소: `for i in 0..n` 범위 기반 for 루프 (현재 `while` 사용 필요)
  2. `[bool; N]` subscript (Phase 6) — bool을 0/1로 인코딩, 1-byte stride
  3. P-track 재측정 확인 (Phase 3+4+5 변경 후 regression 없음 확인)
