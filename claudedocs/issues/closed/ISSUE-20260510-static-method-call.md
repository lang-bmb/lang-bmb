---
id: ISSUE-20260510-static-method-call
title: Type::method() 정적 메서드 호출 파서 미지원
type: language-gap
priority: M4
status: open
created: 2026-05-10
---

## 문제

```bmb
let s = String::from("hello");   // ❌ 파서 처리 안됨
let v = Vec::new();               // ❌ 파서 처리 안됨
let r = MyType::create(arg);     // ❌ 파서 처리 안됨
```

현재 `parse_call_or_ident` 에서 `Name::Variant` 처리 시 variant name 이후
`(args)` 가 오는 경우를 처리하지 않는다. 결과적으로:
1. `(enum_variant <Name> <method>)` AST 생성됨
2. 뒤의 `(args)` 부분이 별도로 파싱되어 문법 오류 발생

## 영향

LLM이 associated function / factory method 패턴에서 자주 사용.
미지원 시 `fn create() = ...` 형태의 우회 코드 필요.

## 근거 (Drift C)

`Type::method()` 는 Rust / C++ 에서 표준 패턴.
LLM이 생성하는 코드의 상당 비율이 이 패턴을 포함한다.

## 해결 방향

`bootstrap/compiler.bmb`의 `parse_call_or_ident` 수정 (line ~782):
1. `::` 다음 variant/method name 파싱
2. 그 다음 `(` 이면 → `(static_call <Type> <method> args)` AST 생성
3. 그 다음 `(` 없으면 → 기존 `(enum_variant <Type> <Variant>)` 유지

## 테스트 기준

`tests/bootstrap/` 에 골든 테스트 추가:
```bmb
fn MyType_create(x: i64) -> i64 = x * 2;
// static call: MyType::create(21) should call MyType_create(21)
fn main() -> i64 = MyType::create(21);  // expected: 42
```
