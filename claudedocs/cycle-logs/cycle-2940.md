# Cycle 2940: native codegen — str_byte_at + println(String/f64) dispatch
Date: 2026-05-19

## Re-plan
Cycle 2939 Carry-Forward: str_byte_at native codegen + println(String) native dispatch.
두 항목 모두 이번 사이클에서 구현.

## Scope & Implementation

### 1. `bmb_str_byte_at` C runtime 함수 추가

`bmb/runtime/bmb_runtime.c`에 추가:
```c
int64_t bmb_str_byte_at(const BmbString* s, int64_t index) {
    if (!s || !s->data || index < 0 || index >= s->len) return 0;
    return (int64_t)(unsigned char)s->data[index];
}
```
- runtime 라이브러리 재빌드: `gcc -O2 -c bmb_runtime.c -o /tmp/bmb_runtime.o && ar rcs libbmb_runtime.a /tmp/bmb_runtime.o`

### 2. LLVM text backend 업데이트

`bmb/src/codegen/llvm_text.rs`:
- declare 추가: `declare i64 @bmb_str_byte_at(ptr nonnull nocapture readonly, i64) nocallback nounwind nofree nosync willreturn`
- function name mapping: `"str_byte_at" => "bmb_str_byte_at"`
- `runtime_param_type`: `("str_byte_at", 1) | ("bmb_str_byte_at", 1) => Some("i64")`
- void list에 추가: `"println_str" | "println_f64" | "print_f64"`
- `has_user_call` exclusion: `println_str`, `print_str`, `println_f64`, `print_f64`

### 3. MIR lowering println/print dispatch

`bmb/src/mir/lower.rs`에 println/print 인자 타입 검사 후 String/f64별 native 함수로 라우팅:
- `println(String)` → `println_str(BmbString*)`
- `print(String)` → `print_str(BmbString*)`
- `println(f64)` → `println_f64(f64)`
- `print(f64)` → `print_f64(f64)`

**배경**: 이전에는 `println`이 i64 전용으로 구현되어 있었음. `println("hello")` native에서 포인터 주소가 출력되던 버그 수정.

**변경 파일**:
- `bmb/runtime/bmb_runtime.c`: str_byte_at 함수 신규
- `bmb/src/codegen/llvm_text.rs`: 선언 + 매핑 + void list
- `bmb/src/mir/lower.rs`: println/print dispatch block

## Verification & Defect Resolution

```
cargo test --release -p bmb: 2388 passed ✅
str_byte_at("hello", 0) native: 104 ✅ ('h' ASCII)
println("hello native") native: hello native ✅ (이전: 포인터 주소 출력)
```

### 결함 없음

## Reflection

### Scope fit
- ✅ str_byte_at native 동작
- ✅ println(String) native 정상 출력

### 의의
- native 실행에서 문자열 처리 기본 기능 (`str_byte_at`, `println(String)`) 사용 가능
- MIR 타입 기반 dispatch 패턴 확립 (향후 print/println 확장 기반)

### 잠재적 개선
- `println(3.14)` native: `%.9f` 형식으로 출력 (interpreter의 smart format과 차이). 현재 허용 가능한 제한.

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  1. `println(f64)` 출력 포맷 통일 (native `%.9f` vs interpreter smart format)
  2. `str_byte_at` 범위 체크 동작을 BMB 계약으로 명시
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2941 — 추가 언어 갭 해소 또는 native codegen 개선
