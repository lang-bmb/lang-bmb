---
id: ISSUE-20260510-option-expr-position
title: Option::Some(x) 표현식 위치 지원 (페이로드 변형)
type: language-gap
priority: M4
status: open
created: 2026-05-10
---

## 문제

```bmb
let x: i64? = Option::Some(42);  // ❌ bootstrap 컴파일러에서 미지원
let y = MyOption::Some(value);   // ❌ bootstrap 컴파일러에서 미지원
```

현재 `parse_call_or_ident` 에서 `Name::Variant` 다음에 `(expr)` 가 오면
variant name까지만 파싱하고 `(expr)` 는 미처리됨.

패턴 위치: 이미 지원 (`match x { Option::Some(v) => ... }`)
표현식 위치: 미지원

## 영향

- `Option<T>` / `Result<T,E>` 스타일의 enum 생성이 불가
- 에러 처리, 옵셔널 값 반환 패턴에서 verbose 코드 필요
- Rust 컴파일러 경로는 이미 지원 (`tests/examples/valid/test_generics_stdlib.bmb` 참조)

## 근거 (Drift C)

LLM이 가장 자주 생성하는 패턴 중 하나.
bootstrap 컴파일러와 Rust 컴파일러 간 기능 불일치.

## 해결 방향 (Cycle 2622 스코프 분석 업데이트)

**파서 수준 — M4-4 이미 처리**: Cycle 2620에서 `Type::Variant(x)` 구문이 이미 `(call <Type_Variant> x)` 로 파싱됨 (static method call과 동일 처리). 파서 변경은 불필요.

**실제 필요 변경**:
1. `parse_enum_variants_to_registry` (line ~2909): 페이로드 타입 정보도 저장하도록 변경
2. enum 표현 재설계: ordinal → tagged union `{ discriminant: i64, payload: ptr }`
3. `resolve_enum_variants_in_ast`: unit variant는 ordinal, payload variant는 tagged union 생성
4. 패턴 매칭: payload 추출 `match x { Some(v) => ... }` → discriminant 검사 + payload GEP
5. LLVM codegen: 새 tagged union alloca/store/load/gep 패턴

주의:
- 현재 bootstrap 컴파일러는 unit 변형을 `(int N)` 으로 인코딩.
- 페이로드 있는 변형은 다른 인코딩 필요 → **전체 enum 레이어 재설계** 필요.
- 예상 규모: 5-10+ 사이클. M4 남은 사이클(6)에서 완성 불가.
- **재분류**: M5-1 (payload enum 설계 및 구현) 로 이동.

**M4-4 사이드 이펙트 주의**:
`Type::Variant(x)` 구문은 현재 파서에서 `(call <Type_Variant> x)` 로 처리됨.
`Type_Variant` 이름의 함수가 없으면 undefined variable 에러. payload enum constructor를
사용하려면 동일 이름의 자유 함수를 수동으로 정의해야 하는 임시 workaround가 가능하지만,
CLAUDE.md Principle 2에 따라 권장하지 않음.

## 테스트 기준

```bmb
enum Color { Red, Green(i64), Blue }
fn main() -> i64 = {
    let g = Color::Green(42);
    match g {
        Color::Red => 0,
        Color::Green(v) => v,
        Color::Blue => 99,
    }
};  // expected: 42
```
