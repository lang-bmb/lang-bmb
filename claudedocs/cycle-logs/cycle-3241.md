# Cycle 3241: M11-C — `arr: *` raw pointer parameter syntax fix

Date: 2026-05-28

## Re-plan

Plan valid: Carried from Cycle 3240 Carry-Forward.
- Investigation: `[T; N]` array parameters mis-parsed when using `arr: *` syntax
- Root cause found: `parse_param` `TK_STAR()` branch consumed the following token (`,` or `)`)
  unconditionally, leaving position past the comma → next parse_params loop saw identifier
  instead of separator → parse error "expected ',' or ')'"

## Scope & Implementation

### 설계

**Root cause**: `parse_param` line 5654, TK_STAR branch:
```bmb
let t4 = next_token_raw(src, tok_end(t3));
let struct_name = get_ident_text(src, tok_end(t3), t4);
pack_result(skip_nullable(src, tok_end(t4)), "(param <" + name + "> *" + struct_name + ")")
```
`tok_end(t4)` — t4 is the `,` or `)` separator, consuming it causes the params loop to fail.

**Fix**: Only consume t4 when it is an ident token:
```bmb
let t4 = next_token_raw(src, tok_end(t3));
if tok_kind(t4) == TK_IDENT() {
    let struct_name = get_ident_text(src, tok_end(t3), t4);
    pack_result(skip_nullable(src, tok_end(t4)), "(param <" + name + "> *" + struct_name + ")")
} else {
    pack_result(skip_nullable(src, tok_end(t3)), "(param <" + name + "> *)")
}
```

**Result**:
- `arr: *` → `(param <arr> *)` ← matches `rewrite_ptr_index` check `param_type_raw == "*"`
- `arr: *MyStruct` → `(param <arr> *MyStruct)` ← unchanged behavior

**`rewrite_ptr_index`** already handles bare `*` params by rewriting `arr[i]` → `(ptr_index ...)` (i64 stride, no +2 header offset). So passing a `[i64; N]` stack array via `arr: *` parameter works correctly.

### 변경사항

- **`bootstrap/compiler.bmb`**: `parse_param` TK_STAR branch — 1개 수정 (ident-only token consumption)

### 골든 테스트

`tests/bootstrap/test_probe_ptr_param.bmb` (→ golden):
```bmb
-- Test: raw pointer parameter subscript
fn fill_arr(arr: *, n: i64) -> i64
  pre n >= 0
  post it == 0
= {
    let mut i = 0;
    while i < n {
        set arr[i] = i * i;
        i = i + 1
    };
    0
};

fn main() -> i64 = {
    let arr: [i64; 5];
    let _r = fill_arr(arr, 5);
    let sum = arr[0] + arr[1] + arr[2] + arr[3] + arr[4];
    println(sum)
};
```
Expected output: `30` (0²+1²+2²+3²+4² = 0+1+4+9+16 = 30)

## Verification & Defect Resolution

### 골든 테스트 결과

- Stage 1 빌드 ✅ (32G arena)
- `test_probe_ptr_param.bmb` → 30 ✅
- `test_golden_stack_bool_subscript.bmb` → 3 ✅ (Phase 6 regression check)
- `test_golden_stack_u8_subscript.bmb` → 30 ✅ (Phase 4 regression check)
- `test_golden_stack_i32_subscript.bmb` → 30 ✅ (Phase 5 regression check)

### 3-Stage Bootstrap

- Stage 1 (`compiler_3241_s1.exe`): ✅ (~2s Rust compile + S1 build)
- Stage 2 (`compiler_3241_s2.exe`): ✅ (~24s compile + ~12s link)
- Stage 3 (IR 검증): ✅ (~25s compile + ~12s link)
- **Fixed Point: S3 IR == S4 IR (0 diff)** ✅

### cargo test

✅ 3800+47+22+2390+23 tests (unchanged)

### bootstrap/compiler.exe

`compiler_3241_s2.exe` → `bootstrap/compiler.exe` 복사 완료 ✅

## Reflection

### Scope fit ✅
`arr: *` 파라미터 구문 파서 버그 수정 완료.
이제 스택 배열을 함수에 넘기는 관용 패턴이 동작한다:
```bmb
fn process(arr: *, n: i64) -> i64 = ...
let arr: [i64; 5];
process(arr, 5)
```

### 최소 패치 ✅
TK_STAR 분기 1개 조건 추가 — 인접 코드 무변경. `*StructName` 파라미터 기존 동작 보존.

### arr: * 의미론 ✅
- `parse_param` → `(param <name> *)` (bare star)
- `rewrite_ptr_index` → `param_type_raw == "*"` 매칭 → `arr[i]` = `(ptr_index ...)`
- `ptr_index` = i64 stride (8바이트) raw pointer subscript (no heap-struct +2 header)
- 결과: 스택 배열 포인터 값이 그대로 GEP base로 사용됨

### 한계 ✅ (문서화)
- `arr: *` subscript는 i64 stride만 지원 (8바이트 GEP)
- `[u8; N]`, `[i32; N]`, `[bool; N]` 배열을 `arr: *`로 넘기면 잘못된 stride 사용
- 해결 방안: `arr: *u8`, `arr: *i32`, `arr: *bool` 등 타입화된 포인터 파라미터 추가 (별도 사이클)

## Carry-Forward

- **Actionable**:
  - `*u8`/`*i32`/`*bool` 파라미터 타입별 stride 지원 (typed pointer parameter subscript)
  - 다음 언어 갭 해소 작업 (ROADMAP.md § M11-C 방향)

- **Structural Improvement Proposals**:
  - `rewrite_ptr_index`가 bare `*`만 처리하는 것은 의도적 설계. 타입별 stride 지원 시
    `*u8`/`*i32`/`*bool`에 대한 별도 rewrite 함수 필요 (`rewrite_ptr_u8_index` 등)
  - 또는 파라미터 타입 정보를 직접 인코딩하는 단일 rewriter 리팩토링 가능

- **Pending Human Decisions**: None

- **Roadmap Revisions**:
  - ROADMAP.md M11-C: `arr: *` parameter syntax fix 항목 추가

- **Next Recommendation**:
  1. `*u8`/`*i32`/`*bool` 파라미터 타입별 stride subscript (Phase 8)
  2. 또는 `match` with guards (`if` guards in match arms) 언어 갭
  3. 또는 f64 array subscript `[f64; N]` (Phase 7 — f32처럼 별도 고려 불필요, f64 = 8B = i64와 동일)
