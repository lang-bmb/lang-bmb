---
id: ISSUE-20260510-let-tuple-destructuring
title: let-tuple destructuring 파서 미지원
type: language-gap
priority: M4
status: open
created: 2026-05-10
---

## 문제

```bmb
let (a, b) = some_pair;   // ❌ 파서 에러
let (x, y, z) = triple;  // ❌ 파서 에러
```

현재 `parse_let_expr` 은 `let <ident> = expr` 만 지원한다.
`(` 다음에 오는 식별자 목록 패턴은 인식하지 못한다.

## 영향

LLM이 함수 반환값을 분해할 때 자연스럽게 사용하는 패턴.
미지원 시 임시 변수 도입 필요 → 코드 장황화.

## 근거 (Drift C)

AI-native 언어 선언은 LLM 자연 패턴 지원 의무를 수반한다.
tuple destructuring은 LLM이 매우 자주 생성하는 패턴이다.

## 해결 방향

`bootstrap/compiler.bmb`의 `parse_let_expr` 수정:
1. `let` 다음 `(` 발견 시 tuple 패턴 파싱
2. `(a, b) = expr` → `let __t = expr; let a = __t.0; let b = __t.1;` 등으로 탈슈가
3. 또는 `(let_tuple <a> <b> expr body)` AST 노드 추가

전제: tuple 타입 인코딩이 compiler.bmb 내부에 존재해야 함.

## 테스트 기준

`tests/bootstrap/` 에 골든 테스트 추가:
```bmb
fn make_pair() -> (i64, i64) = (1, 2);
fn main() -> i64 = {
    let (a, b) = make_pair();
    a + b  // expected: 3
};
```
