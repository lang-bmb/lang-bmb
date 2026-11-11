# 01 어휘와 구문

이 장은 소스 텍스트가 토큰으로 나뉘는 규칙(식별자, 예약어, 리터럴, 주석, `///`)과 구문의 뼈대(최상위와 `mod` 안에 오는 것), 중첩 한도를 적는다. 항목 각각의 뜻은 02장, 타입·식·계약의 문법은 03~06장에 있다. 이 장의 규칙을 어기면 E01xx가 나온다.

## 문자와 공백

소스는 UTF-8 텍스트다. 토큰 사이의 공백은 ASCII 공백 문자(빈칸, 탭, 줄바꿈 등)이고 뜻이 없다. 비ASCII 문자는 주석과 문자열·문자 리터럴 안에만 온다. 어느 토큰도 시작하지 못하는 문자는 E0101 오류다(`` ` ``: `tests/check/err/lexing.bmb`). 빈 파일과 주석만 있는 파일은 단위가 없는 프로그램이다.

## 식별자와 예약어

```
ident = [A-Za-z_] [A-Za-z0-9_]*      (예약어 제외)
```

식별자는 ASCII 글자·숫자·`_`만 쓴다. 비ASCII 글자는 E0101 오류다. Rust의 날 식별자(`r#fn`)는 없다. 예약어는 이름 자리(단위·항목·매개변수·지역 이름 등)에 쓰지 못하고, 쓰면 E0103 오류다.

```
fn spec proof mod let mut while if else true false requires ensures decreases
invariant forall exists old assert assume return for loop match break continue
as pub struct enum impl trait use const static type unsafe self Self crate
super where ref move dyn in async await
```

예약어 가운데 BMB의 구문에 쓰이지 않는 것(`pub`, `use`, `static`, `where` 등)은 Rust 습관을 알아보고 E02xx로 거부하기 위한 것이다(04장). 예외는 둘이다: `self`는 수신자 매개변수의 이름이고(02장), `invariant`는 속성 이름 자리에서 이름이다(`#[invariant(p)]`, ADR-0060). 속성 이름(`summary`, `effects` 등)과 내장 타입·함수의 이름은 예약어가 아니다. `'이름`은 `break`·`continue` 뒤와 `:` 앞에서만 루프 레이블이고(`tests/run/loop_exits.bmb`), 다른 자리에서는 수명 표기로 읽혀 E0208 오류다.

## 구두점

```
==>  ..=
::  ->  =>  ==  !=  <=  >=  &&  ||  ..
( ) { } [ ] , ; : . | # ! & = < > + - * / % ? @ ^ ~ $
```

가장 긴 것부터 맞춘다. `<<`와 `>>`는 토큰이 아니다. 붙어 있는 `<` `<`, `>` `>`를 파서가 시프트로 읽는다(ADR-0048, `tests/run/bits.bmb`). 목록의 토큰이 문법상 올 수 없는 자리에 오면 E0103 오류다.

## 정수 리터럴

```
int = dec | "0x" hex | "0o" oct | "0b" bin      (숫자 사이와 끝의 "_"는 구분자)
```

- 10진수와 `0x`·`0o`·`0b`(접두는 소문자). `_`는 배치일 뿐이고 값에 들지 않는다(ADR-0048 §3, `tests/run/bits.bmb`).
- 그 진법 밖의 숫자(`0b102`)와 숫자 없는 접두(`0x`)는 E0108 오류다(`tests/check/err/bits.bmb`).
- 접미사는 없다. 숫자 바로 뒤에 글자·`_`가 이어지면 E0104 오류다(`5i64`, `0XFF`). 타입은 문맥이 정한다(03장).
- 값이 128비트에 들지 않으면 E0105 오류다(`tests/check/err/lexing.bmb`). 타입의 범위 검사는 03장의 것이다.
- 음수 리터럴은 없다. `-1`은 단항 `-`와 리터럴이다.
- 정규형(`bmb fmt`)은 쓴 진법을 유지하고 숫자를 소문자로, 구분자와 앞의 0 없이 쓴다(`0xFF_00` → `0xff00`).

## 부동소수 리터럴

```
float = digits "." digits [ ("e" | "E") ["+" | "-"] digits ]
```

점 양쪽에 숫자가 있어야 하고 지수에는 숫자가 있어야 한다(ADR-0061 §2). `1.`, `.5`, `1e5`, `1.0e`는 E0109 오류다(`tests/check/err/float_literals.bmb`). 접미사(`1.0f64`)는 E0104 오류다. 숫자 뒤의 `.`은 다음 글자가 식별자의 시작이거나 `.`이면 리터럴의 일부가 아니다: `0..n`은 범위, `1.f()`는 메서드 호출로 읽는다. 받는 꼴은 `tests/run/floats.bmb`에 있다.

## 문자·문자열·바이트 리터럴

```
char  = "'" (문자 | escape) "'"            타입 char
str   = '"' (문자 | escape)* '"'           타입 &str
byte  = "b'" (ASCII | bescape) "'"         타입 u8
bytes = 'b"' (ASCII | bescape)* '"'        타입 &[u8]
escape  = \n \r \t \\ \' \" \0 \xHH (7f까지) \u{H..}
bescape = \n \r \t \\ \' \" \0 \xHH
```

- `char`는 유니코드 스칼라 값 하나다(ADR-0050 §1). 문자 둘 이상(`'ab'`)과 목록 밖의 이스케이프는 E0101 오류다(`tests/check/err/strings.bmb`).
- 문자열은 비ASCII 문자와 줄바꿈을 그대로 담는다. 닫는 `"`가 없으면 E0102 오류이고 파일 끝까지가 리터럴이다(`tests/check/err/lexing.bmb`).
- 바이트 리터럴은 ASCII 문자와 이스케이프만 담는다(ADR-0044 §2). 비ASCII 문자, 목록 밖의 이스케이프, 한 바이트가 아닌 `b'..'`는 E0101, 닫히지 않으면 E0102 오류다(`tests/check/err/bytes.bmb`).
- 문자열·바이트 문자열 리터럴이 올 수 있는 식의 자리는 03장에 있다. 받는 꼴은 `tests/run/strings.bmb`, `tests/run/bytes.bmb`.

## 주석

주석은 `//`부터 줄 끝까지뿐이고 토큰이 되지 않는다. `////`로 시작하는 줄과 `//!`도 주석이다. 블록 주석 `/* .. */`는 E0106 오류다(`tests/check/err/rust_habits.bmb`). 주석은 프로그램의 뜻에 들지 않는다.

## `///`와 요약

`///`(바로 뒤가 `/`가 아닌 것)는 주석이 아니라 `#[summary("..")]`의 입력 꼴이다(ADR-0117). 이어진 `///` 줄들은 한 묶음이고, 각 줄의 앞뒤 공백을 뗀 글을 빈칸 하나로 이어 문자열 하나로 삼는다. 빈 줄이 끼면 묶음이 끊긴다. 글이 없는 묶음은 아무 토큰도 내지 않는다. 묶음은 그 자리에서 속성 토큰 `#[summary("..")]`으로 읽히므로 속성이 올 수 있는 자리에만 온다: 문장 앞이면 E1002(`tests/check/err/summaries.bmb`), 항목이 뒤따르지 않으면 E0103 오류다. 속성의 규칙(E0704)은 02장에 있다. 정규형은 속성 꼴로 쓴다.

```
/// 길이 단위의 변환.
mod len {
    /// 1미터의 센티미터 수.
    const CM: i64 = 100;

    // 줄 주석은 버려진다.
    #[effects(pure)]
    fn doubled() -> i64 {
        CM + 0x64
    }
}
```

## 구문의 뼈대

```
program = unit*
unit    = attr* "mod" ident "{" item* "}"
item    = attr* ( unit | const | struct | enum | trait | impl | fn )
fn      = ["spec" | "proof"] "fn" ident [generics] "(" params ")" ["->" ret] clauses block
ret     = type | "(" ident ":" type ")"
attr    = "#" "[" name [ "(" arg ("," arg)* [","] ")" ] "]"
arg     = string | path | "<" type "as" path ">" "::" ident | ident "=" (ident | string)
```

- 최상위에는 단위만 온다. 최상위의 `fn`·`spec fn`·`proof fn`은 E0703, 그 밖의 토큰은 E0103(`` expected `mod` ``)이다(`tests/check/err/items.bmb`).
- `mod` 안에는 위의 일곱 가지 항목만 온다. 그 밖의 토큰은 E0103 오류다. Rust의 항목(`use`, `pub`, `static`, `type`, `unsafe` 등)은 알아보고 E02xx로 거부한다.
- `impl`과 `trait`의 몸에는 `fn`과 `spec fn`만 온다(E0103). trait의 메서드는 몸 대신 `;`로 끝날 수 있다.
- 함수의 계약 절은 `requires`, `ensures`, `decreases` 순서다. 순서가 다르면 E0103 오류다(`tests/check/err/specs.bmb`). 절과 몸의 문법은 04·06장에 있다.
- 기대한 토큰이 하나일 때 E0103 진단의 `fix`는 그 토큰을 넣으라는 안내다. 구문 오류가 난 항목은 건너뛰고 다음 항목부터 다시 읽는다.
- 전체를 받는 예는 `tests/run/basics.bmb`.

## 중첩 한도

구문 중첩은 256단계까지다. 괄호·단항 식, 블록, `if`·`else if`, 루프, `==>` 사슬, 이항·후위 연산의 사슬, 타입, 단위가 한 깊이를 함께 센다. 넘으면 E0107 오류다(`tests/check/err/nesting.bmb`). 단위만 겹치면 256겹까지 받는다.
