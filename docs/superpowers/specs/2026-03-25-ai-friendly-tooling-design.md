# BMB AI-Friendly Tooling — 설계 문서

> **목표**: BMB 컴파일러의 에러 출력을 AI-native로 전환하고, 예제 은행을 내장하여
> LLM이 BMB 코드를 효과적으로 생성/수정할 수 있게 한다.

---

## 1. 배경 — 파일럿 데이터가 증명한 문제

ai-proof 파일럿 (3문제 × 2조건 × 1회):
- Type C (semantic error) = 전체 루프의 90%+ → LLM이 BMB 문법을 모름
- Type A (contract error) = 0% → 계약 효과 측정 불가
- 원인: BMB 학습 데이터 0, 에러 메시지가 AI-hostile (텍스트, 힌트 없음)

**근본 해결**: 문제를 쉽게 만드는 것이 아니라 **도구를 개선**한다.
BMB는 AI-native 언어 — 기본 출력이 AI용이어야 한다.

---

## 2. 컴파일러 에러 출력 개혁

### 2.1 기본 출력 = JSON

`bmb build/check`의 기본 에러 출력을 JSON으로 전환.

```
bmb build foo.bmb              → JSON (기본, AI/MCP용)
bmb build foo.bmb --error-format=text   → 사람용 텍스트 (ariadne)
bmb build foo.bmb --error-format=json-pretty → 들여쓰기 JSON
```

### 2.2 JSON 스키마

```json
{
  "success": false,
  "errors": [
    {
      "code": "E0421",
      "severity": "error",
      "message": "unknown function `Option::Some`",
      "location": {"file": "foo.bmb", "line": 5, "col": 12},
      "suggestion": "BMB uses `T?` for nullable types, not `Option<T>`",
      "example": {
        "wrong": "let x: Option<i64> = Some(42);",
        "correct": "let x: i64? = 42;",
        "pattern": "nullable_type"
      },
      "related_patterns": ["nullable_type", "option_syntax"]
    }
  ],
  "warnings": [],
  "summary": {"errors": 1, "warnings": 0}
}
```

성공 시:
```json
{
  "success": true,
  "errors": [],
  "warnings": [],
  "output": "foo.exe",
  "summary": {"errors": 0, "warnings": 0}
}
```

### 2.3 AI 실수 패턴 매핑

파일럿 데이터 + CLAUDE.md에서 도출한 LLM의 빈출 실수:

| 에러 패턴 | AI가 쓰는 잘못된 코드 | suggestion | example.correct |
|----------|---------------------|-----------|----------------|
| `Option<T>` | `let x: Option<i64> = Some(42)` | `T?` 사용 | `let x: i64? = 42;` |
| `&` 비트 연산 | `a & b` | `band` 사용 | `a band b` |
| 재할당 | `x = 5` | `set` 키워드 | `set x = 5;` |
| for 루프 | `for i in 0..n` | while 사용 | `while i < n { set i = i + 1; }` |
| println 매크로 | `println!("{}", x)` | 함수 호출 | `println(x);` |
| String 타입 | `String::from("hello")` | `&str` 사용 | `let s: &str = "hello";` |
| Vec 제네릭 | `Vec<i64>`, `vec.push()` | vec_* 함수 | `vec_push(v, 42);` |
| 메서드 호출 | `v.len()`, `v.push(x)` | 자유 함수 | `vec_len(v)`, `vec_push(v, x)` |
| 암묵적 반환 | `fn f() -> i64 { 42 }` | `= expr;` 패턴 | `fn f() -> i64 = 42;` |
| 타입 추론 | `let x = 42` | 명시적 타입 | `let x: i64 = 42;` |
| 구조체 문법 | `struct Foo { x: i64 }` | BMB struct | `struct Foo { x: i64 }` |
| 트레이트 impl | `impl Foo { fn bar() }` | 자유 함수 | `fn foo_bar(self: &Foo)` |
| 튜플 분해 | `let (a, b) = ...` | 개별 let | `let a = ...; let b = ...;` |
| Static method | `Type::method()` | 자유 함수 | `type_method()` |
| match 언더스코어 | `_ => ...` | else 사용 | `else { ... }` |
| 범위 연산자 | `0..n`, `0..=n` | 미지원 | while 루프 |

### 2.4 구현 위치

`bmb/src/error/` 기존 에러 시스템 확장:
- `mod.rs`: `ErrorFormat` enum 추가 (`Json`, `Text`, `JsonPretty`)
- `json_formatter.rs`: 새 파일 — JSON 직렬화 + suggestion 주입
- `suggestions.rs`: 새 파일 — 에러코드/메시지 → suggestion 매핑
- `main.rs`: `--error-format` CLI 플래그 추가, 기본값을 `Json`으로

**Rust 동결 정책과의 관계**: 이것은 새 언어 기능이 아닌 기존 에러 시스템의 출력 형식 개선.
AI-native 언어의 에러가 AI-hostile인 것은 결함(defect)이므로 수정 대상.

---

## 3. 예제 은행 (Pattern Bank)

### 3.1 구조

컴파일러에 내장. 에러 발생 시 관련 예제를 자동으로 JSON에 포함.

```
bmb/src/examples/
├── mod.rs              # PatternBank API
├── patterns.rs         # 에러 → 패턴 매핑 테이블
└── bank/               # 패턴별 예제 파일 (include_str!)
    ├── nullable_type.md
    ├── bitwise_ops.md
    ├── mutable_reassign.md
    ├── loop_while.md
    ├── vec_operations.md
    ├── io_read_print.md
    ├── array_contract.md
    ├── function_return.md
    ├── string_type.md
    ├── method_to_free_fn.md
    ├── type_annotation.md
    ├── struct_syntax.md
    ├── match_else.md
    ├── range_to_while.md
    └── ...  (~20 patterns)
```

### 3.2 패턴 파일 포맷

```markdown
---
id: vec_operations
triggers: ["Vec<", "vec.push", "vec.len", ".push(", ".pop(", "Vec::new"]
---
## Wrong
let v: Vec<i64> = Vec::new();
v.push(42);
let len = v.len();
## Correct
let v: i64 = vec_new();
vec_push(v, 42);
let len: i64 = vec_len(v);
## Notes
BMB vectors use free functions, not methods. Handle is i64.
vec_new() → vec_push(v, val) → vec_get(v, idx) → vec_set(v, idx, val) → vec_len(v) → vec_free(v)
```

### 3.3 데이터 소스

1. **골든 테스트 자동 추출** — `tests/golden/`에서 패턴별 작동하는 코드 조각 추출하여 bank/ 초기 데이터 생성
2. **파일럿 에러 역추적** — pilot Type C 에러 28건에서 누락 패턴 보충
3. **CLAUDE.md "미지원 문법"** — 이미 정리된 AI 실수 패턴에서 직접 도출

### 3.4 매칭 로직

```rust
pub struct PatternBank {
    patterns: Vec<Pattern>,
}

impl PatternBank {
    /// 에러 메시지에서 trigger 키워드 매칭
    pub fn find_by_error(&self, error_msg: &str) -> Vec<&Pattern> {
        self.patterns.iter()
            .filter(|p| p.triggers.iter().any(|t| error_msg.contains(t)))
            .collect()
    }

    /// 에러 코드로 직접 검색
    pub fn find_by_code(&self, code: &str) -> Option<&Pattern> { ... }
}
```

---

## 4. ai-proof 실험 프레임워크 통합

### 4.1 오케스트레이터 변경

`orchestrator/error_normalizer.py` → JSON 에러를 직접 파싱:

```python
def normalize_error_json(json_output: str, lang: str) -> dict:
    """Parse JSON compiler output, extract suggestion + example."""
    data = json.loads(json_output)
    if not data.get("errors"):
        return {"type": "success", ...}
    err = data["errors"][0]
    return {
        "type": "compile_error",
        "normalized": err["message"],
        "location": f"{err['location']['file']}:{err['location']['line']}:{err['location']['col']}",
        "suggestion": err.get("suggestion", ""),
        "example": err.get("example", {}),
        "raw": json_output,
    }
```

### 4.2 에러 피드백 프롬프트 변화

Before (텍스트, 힌트 없음):
```
compile_error: unknown function `Option::Some`
Location: foo.bmb:5:12
Fix the error.
```

After (JSON에서 추출한 suggestion + example 포함):
```
compile_error: unknown function `Option::Some`
Location: foo.bmb:5:12

Suggestion: BMB uses `T?` for nullable types, not `Option<T>`
Wrong: let x: Option<i64> = Some(42);
Correct: let x: i64? = 42;

Fix the error using the correct BMB syntax above.
```

### 4.3 측정 목표

파일럿 재실행으로 개선 효과 직접 비교:
- Before: Type C 루프 평균 ~5회
- Target: Type C 루프 평균 ≤ 2회

---

## 5. bmb-mcp 승격 경로

이 작업이 완료되면 bmb-mcp는 컴파일러를 호출하기만 하면 됩니다:

```
지금 구축 (컴파일러 내장)              → 향후 bmb-mcp (MCP 래퍼)
───────────────────────────────────────────────────────────
에러 JSON 출력                         → bmb_check tool
예제 은행 (PatternBank)                → bmb_example tool
에러→패턴 매핑                         → bmb_spec_lookup tool
ai-proof 오케스트레이터                → MCP 클라이언트 레퍼런스
```

기술부채 없음: 컴파일러에 내장된 기능을 MCP가 노출만 하는 구조.

---

## 6. 변경 범위 요약

| 파일 | 변경 | 위험도 |
|------|------|--------|
| `bmb/src/main.rs` | `--error-format` 플래그, 기본값 Json | LOW |
| `bmb/src/error/mod.rs` | `ErrorFormat` enum 추가 | LOW |
| `bmb/src/error/json_formatter.rs` | 새 파일 — JSON 직렬화 | LOW |
| `bmb/src/error/suggestions.rs` | 새 파일 — AI 실수 패턴 매핑 | LOW |
| `bmb/src/examples/mod.rs` | 새 모듈 — PatternBank | LOW |
| `bmb/src/examples/patterns.rs` | 패턴 레지스트리 | LOW |
| `bmb/src/examples/bank/*.md` | ~20 패턴 파일 | LOW |
| `ecosystem/ai-proof/orchestrator/` | JSON 파싱 + suggestion 피드백 | LOW |

**위험도 전체 LOW**: 기존 컴파일 파이프라인에 영향 없음. 에러 출력 형식만 추가.

---

## 7. 성공 기준

| 지표 | Before | Target |
|------|--------|--------|
| 파일럿 Type C 루프 | ~5회/문제 | ≤ 2회/문제 |
| 파일럿 전체 성공률 | 5/6 (83%) | 6/6 (100%) |
| 에러 메시지에 suggestion 포함률 | 0% | ≥ 80% |
| JSON 파싱 가능 에러 비율 | 0% | 100% |
