# Cycle 2938: `str_byte_at` + `println(String)` 언어 개선
Date: 2026-05-19

## Re-plan
Cycle 2937 Carry-Forward: 없음. 이번 사이클은 언어 갭 탐색 + 소형 빌트인 추가.

## Scope & Implementation

### 1. `str_byte_at(s: String, idx: i64) -> i64` 추가 (interpreter-only)

기존: `str_char_at(s, i)` → String (문자 자체, 인덱스 비교 불가)  
추가: `str_byte_at(s, i)` → i64 (바이트 값, ASCII 코드 비교 가능)

활용 패턴 (B축 경쟁 프로그래밍에서 빈번):
```bmb
fn is_digit(c: i64) -> bool = c >= 48 and c <= 57;
fn count_words(s: String) -> i64 = {
    let count = 0;
    let in_word = false;
    let i = 0;
    loop {
        if i >= str_len(s) { break count };
        let c = str_byte_at(s, i);
        if is_space(c) { set in_word = false }
        else { if not in_word { set count = count + 1; set in_word = true } };
        set i = i + 1
    }
};
```

변경 파일:
- `bmb/src/interp/eval.rs`: `builtin_str_byte_at` 함수 추가 + init 등록
- `bmb/src/types/mod.rs`: `str_byte_at: (String, i64) -> i64` 등록

### 2. `println(String)` / `print(String)` 타입 오버로드

기존: `println`은 `i64`만 허용 → `println("hello")` 타입 에러  
추가: String / f64 타입 인수도 허용 (타입 체커 특별 처리)

**타입 체커 변경** (`bmb/src/types/mod.rs`):
```rust
// v0.99 Cycle 2938: println/print accept String by dispatching to _str variant
if matches!(func.as_str(), "println" | "print") && args.len() == 1 {
    let arg_ty = self.infer(&args[0].node, args[0].span)?;
    if matches!(arg_ty, Type::String | Type::F64) {
        return Ok(Type::Unit);
    }
}
```

**인터프리터 변경** (`bmb/src/interp/eval.rs`):
`builtin_println` / `builtin_print` — String 인수를 따옴표 없이 출력:
```rust
if let Some(s) = arg.materialize_string() {
    print!("{}", s);  // "hello" → hello (따옴표 없음)
} else {
    print!("{arg}");  // 42 → 42
}
```

이제 `println(42)`, `println(3.14)`, `println("hello")` 모두 동작:
```
42
3.14
hello
```

## Verification & Defect Resolution

### 언어 갭 탐색 결과

| 기능 | 상태 |
|------|------|
| `for i in 0..n` 범위 반복 | ✅ 기존 지원 |
| `for i in 0..=n` 포함 범위 | ✅ 기존 지원 |
| match guard (`if cond`) | ✅ 기존 지원 |
| nested loop + break | ✅ 기존 지원 |
| `let (a, b) = expr` tuple destr. | ⚠️ bootstrap만 지원, Rust interp. 미지원 |
| `println(String)` | ✅ Cycle 2938 추가 |
| `str_byte_at` | ✅ Cycle 2938 추가 |

### 테스트 결과

```
cargo test --release -p bmb: 2388 passed (+ 3778 etc.) ✅
골든 테스트: println_types.bmb, break_with_value.bmb 추가
```

## Reflection

### Scope fit
- ✅ `str_byte_at` 인터프리터 구현
- ✅ `println(String)` 타입 오버로드 + 올바른 출력
- ⚠️ `let (a, b) = expr` — Rust interp grammar에 미추가 (복잡도 높음, 별도 사이클)

### `println` 개선 의의
B축 테스트에서 `println("Answer: " + str(n))` 같은 패턴이 자주 나온다. 이제 `println("hello")` 가 타입 에러 없이 동작하므로 출력 편의성 대폭 향상.

### str_byte_at 의의
ASCII 기반 문자 분류 (`is_digit`, `is_alpha`, `is_space`) 패턴이 자연스럽게 표현 가능. 이전에는 `str_char_at`이 String을 반환해 비교 불가.

## Carry-Forward

- Actionable:
  1. `let (a, b) = expr` Rust interp grammar 지원 — 복잡한 grammar 변경 필요, 별도 사이클
- Structural Improvement Proposals:
  1. **str_byte_at native codegen 지원**: C runtime `bmb_str_byte_at` 함수 추가. 현재 interpreter-only.
  2. **println(String) native dispatch**: native codegen에서 `println_str` 호출로 전환 필요.
  3. **Closure HOF unified representation** (Cycle 2935 유지)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2939 — `let (a, b) = expr` tuple destructuring 또는 native codegen println(String) 지원
