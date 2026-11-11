# 02 단위·항목·이름

이 장은 프로그램의 구성 단위(`mod`), 단위에 오는 항목(함수, 타입, trait, `impl`, 상수), 속성과 그 자리, 진입점 `main`, 이름이 정의로 풀리는 규칙을 적는다. 토큰과 구문의 뼈대는 01장, 항목 각각의 타입 규칙은 03장, 계약은 06장, 효과와 탈출구의 뜻은 08장에 있다. 이 장의 규칙을 어기면 E03xx와 E07xx가 나온다.

## 단위

```
unit = "#[summary(" string ")]" "mod" ident "{" item* "}"
```

- 프로그램은 단위의 나열이다. 항목은 단위 안에만 온다. 최상위의 함수는 E0703 오류다(`tests/check/err/items.bmb`).
- 단위에는 요약이 필수다. 없으면 E0704, `#[summary(..)]`의 인자가 문자열 하나가 아니어도 E0704 오류다(`tests/check/err/attrs.bmb`). 단위의 다른 속성은 E0605 오류다.
- 단위는 단위를 담는다. 하위 단위도 요약이 필수이고, 그 경로는 `바깥::안`이다(`tests/check/err/rust_paths.bmb`의 `outer::inner`).
- 최상위 단위의 이름은 프로그램에서 유일하다. 같은 이름의 둘째 단위는 E0302 오류다. 한 단위를 여러 블록으로 나눠 쓸 수 없다.
- 최상위 단위 `std`는 표준 라이브러리의 것이라 정의하지 못한다(E0309, `tests/check/err/std_unit.bmb`).
- `pub`은 없다(E0207). 단위의 모든 정의는 다른 단위에서 전체 경로로 쓸 수 있다.

## 항목

```
const  = "const" ident ":" type "=" expr ";"
struct = "struct" ident [generics] "{" ident ":" type ("," ident ":" type)* [","] "}"
enum   = "enum" ident [generics] "{" variant ("," variant)* [","] "}"
variant = ident [ "(" type ("," type)* ")" ]
trait  = "trait" ident "{" ( fn-head (";" | block) )* "}"
impl   = "impl" [generics] ident [type-args] "{" fn* "}"
       | "impl" [generics] path "for" type "{" fn* "}"
fn     = ["spec" | "proof"] "fn" ident [generics] "(" params ")" ["->" ret] clauses block
```

- 함수는 세 종류다: 실행 함수 `fn`, 명세 함수 `spec fn`, 증명 함수 `proof fn`(06장). `fn`과 `spec fn`은 반환 타입을 쓴다(`-> ()` 포함, 없으면 E0209). `proof fn`은 쓰지 않는다(E0256). 받는 꼴은 `tests/run/basics.bmb`, `tests/verify/pass/proofs.bmb`.
- `const`는 타입을 쓴다. 서로를 가리키는 `const`는 E0318 오류다(ADR-0052 §3, `tests/check/err/consts.bmb`). 값으로 올 수 있는 식은 03장에 있다.
- 단위 안에서 상수·타입·trait·하위 단위·함수의 이름은 한 이름 공간이다. 같은 이름이 둘이면 E0302 오류다(`tests/check/err/items.bmb`). 한 구조체의 필드, 한 열거형의 변형, 한 trait의 메서드, 한 목록의 타입 매개변수, 한 타입의 `impl` 블록들의 함수도 각각 이름이 유일하다(E0302, `tests/check/err/data_types.bmb`, `tests/check/err/impl_blocks.bmb`). 열거형의 변형은 단위의 이름 공간에 들지 않는다.
- 타입과 함수는 내장 이름 `Option`·`Result`·`Some`·`None`·`Ok`·`Err`·`Vec`·`Box`를 쓰지 못하고(ADR-0026), trait은 내장 trait의 이름 `Copy`·`Eq`·`Ord`를 쓰지 못한다(ADR-0046 §1). 둘 다 E0309 오류다(`tests/check/err/data_types.bmb`, `tests/check/err/traits.bmb`).

## `impl`과 수신자

- 고유 `impl T { .. }`은 `T`를 선언한 단위에만 온다. 다른 단위의 타입이나 없는 타입이면 E0308 오류다. 타입마다 경계 목록 하나에 블록 하나이고, 같은 경계의 둘째 블록은 E0308 오류다(ADR-0097, `tests/check/err/impl_blocks.bmb`, 받는 꼴 `tests/run/impl_blocks.bmb`).
- `impl`의 함수에서 첫 매개변수의 타입이 `T`, `&T`, `&mut T`이면 그것이 수신자이고 이름은 `self`다. 다른 이름이면 E0310 오류다. `self`는 수신자만 가리킨다: 둘째 뒤의 매개변수나 `impl` 밖 함수의 매개변수, 수신자 타입이 아닌 매개변수의 이름이면 E0310 오류다(`tests/check/err/data_types.bmb`). 수신자의 타입은 쓴다(`self: &T`. `&self`는 E0228).
- trait impl `impl Tr for T { .. }`은 `Tr`의 단위나 `T`의 단위에 온다. 둘 다 아니면 E0312 오류다. 한 trait에 대해 같은 타입이 될 수 있는 두 impl은 E0311 오류다(`tests/check/err/traits.bmb`).
- trait impl은 trait의 메서드를 모두 정의한다(몸이 있는 메서드는 생략할 수 있다: `tests/run/trait_defaults.bmb`). 빠진 메서드와 trait에 없는 함수는 E0315 오류다. 함수의 시그니처는 `Self`를 구현 타입으로 바꾼 trait의 것과 같아야 하고, trait 메서드가 선언하지 않은 효과를 갖지 못한다. 둘 다 E0316 오류다(`tests/check/err/traits.bmb`, `tests/check/err/effects.bmb`).
- `Self`는 trait 선언 안에서만 타입이다. 다른 자리에서는 E0305 오류다.

## 속성

속성은 그것이 붙는 항목 바로 앞에 온다. 이름은 닫힌 목록이고, 표에 없는 이름이나 표에 없는 자리는 E0605 오류다(`tests/check/err/attrs.bmb`, `tests/check/err/items.bmb`).

| 속성 | 자리 | 뜻이 있는 장 |
| --- | --- | --- |
| `#[summary("..")]` | 단위(필수), 함수·`struct`·`enum`·`trait`·`const`(선택) | 이 절 |
| `#[effects(..)]` | `fn`(필수, 없으면 E0601). `spec fn`에는 E0604, `proof fn`에는 E0256 | 08장 |
| `#[uses(..)]` | 함수 | 07장 |
| `#[external_body]` | `main`이 아닌 `fn`(E1004), 앞에 `#[rationale]`(E1001) | 08장 |
| `#[rationale(kind = K, "..")]` | 함수(`#[external_body]`의 근거), 문장 `assume(..)` 앞 | 08장 |
| `#[invariant(p)]` | `struct`·`enum`(타입마다 하나, E0255) | 06장 |

- 요약은 문자열 하나를 받는다. 그렇지 않으면 E0704 오류이고, 한 정의에 둘이면 E0605 오류다(ADR-0117, `tests/check/err/summaries.bmb`). 받는 꼴은 `tests/verify/pass/summaries.bmb`. `///`는 이 속성의 입력 꼴이다(01장). 요약은 정체성 해시와 증명에 들지 않는다.
- trait과 `impl` 안의 함수도 함수의 속성을 받는다. `impl` 블록 자체는 속성을 받지 않는다: 그 앞의 속성과 `///`는 E0605 오류다(`tests/check/err/attr_places.bmb`).
- 단위의 `#[summary]`도 한 번만 쓴다. 둘째는 E0605 오류다(같은 파일).
- 함수의 `#[rationale(..)]`는 `#[external_body]`와 함께만 온다. 혼자 오면 E1002 오류다(같은 파일).
- 함수 몸 안에서 속성이 오는 자리는 `assume(..)` 문장 앞의 `#[rationale(..)]`뿐이다. 그 밖은 E1002 오류다.
- 한 정의의 속성들은 어느 순서로 써도 같은 프로그램이다.

## 진입점

실행 함수 `main`이 진입점이다. 꼴은 `#[effects(io)] fn main() -> ()`로 정해져 있다: 매개변수가 없고 결과는 `()`이며 선언 효과에 `io`가 있다(다른 효과를 함께 적을 수 있다). 다르면 E0701 오류다. 어느 단위(하위 단위 포함)에 있든 프로그램에 `main`은 하나이고, 둘째부터는 E0702 오류다(`tests/check/err/effects.bmb`). `main`이 없는 프로그램도 검사와 증명의 대상이다. `spec fn main`은 진입점이 아니다. 받는 꼴은 `tests/run/basics.bmb`의 `app::main`.

## 경로와 이름 찾기

`use`가 없다(E0214). 이름은 다음 순서로 풀린다.

- 이름 하나: 그 이름이 쓰인 단위의 정의다. 값 자리에서는 지역 이름이나 그 단위의 `const`, 타입 자리에서는 그 단위의 타입이나 내장 타입, 호출에서는 그 단위의 함수이고 없으면 내장 함수(`print_i64` 등)다.
- `T::f`, `Tr::f`, `T::V`: `T`·`Tr`이 쓰인 단위의 타입·trait이면 그 함수·메서드·변형.
- 그 밖의 경로는 최상위 단위에서 시작하는 전체 경로다: `unit::f`, `unit::T`, `unit::T::f`, `unit::N`, `outer::inner::f`. 상대 경로는 없다. 하위 단위에서 바깥 단위의 이름을, 바깥 단위에서 하위 단위의 이름을 쓸 때도 전체 경로다. 자기 단위의 이름을 전체 경로로 써도 된다. `crate::`·`super::`·`self::`는 E0242 오류다(`tests/check/err/rust_paths.bmb`).

풀리지 않으면 종류에 따라 코드가 다르다.

| 코드 | 풀리지 않은 것 | 코퍼스 |
| --- | --- | --- |
| E0301 | 값 자리의 이름(지역, `const`), 값이 아닌 경로(`unit::f`에 호출 인자가 없음) | `tests/check/err/types.bmb`, `tests/check/err/consts.bmb` |
| E0304 | 호출의 함수 | `tests/check/err/types.bmb` |
| E0305 | 타입, trait, 내장 타입의 상수(`i64::MID`) | `tests/check/err/data_types.bmb`, `tests/check/err/traits.bmb`, `tests/check/err/types.bmb` |
| E0306 | 구조체의 필드(생성, 읽기) | `tests/check/err/data_values.bmb` |
| E0307 | 열거형의 변형(생성, 패턴) | `tests/check/err/data_values.bmb`, `tests/check/err/match.bmb` |
| E0317 | `break`·`continue`의 레이블(둘러싼 루프의 것만) | `tests/check/err/loop_exits.bmb` |

```
#[summary("도형")]
mod geo {
    struct P { x: i64 }

    #[effects(pure)]
    fn origin() -> P { P { x: 0 } }
}

#[summary("진입점")]
mod app {
    #[effects(io)]
    fn main() -> () {
        let p: geo::P = geo::origin();
        print_i64(p.x);
    }
}
```

## 메서드 호출의 이름

`x.f(..)`의 `f`는 `x`의 타입의 고유 `impl` 함수가 먼저이고, 없으면 그 이름의 메서드를 가진 trait 하나다(ADR-0051, `tests/run/methods.bmb`). 둘 이상의 trait이 그 이름을 가지면 E0313 오류이고 경로(`Tr::f(..)`)로 부른다. 수신자가 없는 함수와 자기 타입 매개변수가 있는 함수는 메서드로 부르지 못하고 경로로 부른다(E0314). 둘 다 `tests/check/err/methods.bmb`. 수신자 조정과 타입 규칙은 03·05장에 있다.

## 지역 이름

- 지역 이름(`let`, 패턴의 바인딩, `for` 변수)은 그 자리에서 보이는 이름을 가리지 못한다: 매개변수, 바깥 블록의 지역, 결과 이름(`-> (r: T)`의 `r`), 쓰인 단위의 `const`. 가리면 E0303 오류다(`tests/check/err/types.bmb`, `tests/check/err/consts.bmb`, `tests/check/err/match.bmb`). 같은 이름의 두 매개변수도 E0303 오류다.
- 범위가 겹치지 않는 자리(형제 블록, 두 `for`, 다른 `match` 갈래)는 같은 이름을 다시 쓸 수 있다. 다만 한 함수에서 이름 하나의 타입은 하나다. 타입이 다르면 E0303 오류다(ADR-0095 A3, 받는 꼴 `tests/run/scopes.bmb`).
- 지역 이름은 함수·타입·단위의 이름과 겹쳐도 된다. 자리로 구별된다.
- `for` 변수는 루프 안에서만 보인다. 루프 뒤에서 쓰면 E0301 오류다(`tests/check/err/ranges.bmb`).

## 표준 라이브러리

표준 라이브러리의 단위는 `lib/std/`의 파일 하나씩이고(`lib/std/str.bmb`가 `std::str`), 최상위 단위 `std` 아래의 단위로서 `std::NAME::..` 전체 경로로 쓴다(ADR-0092). 선언이나 가져오기는 없다. 프로그램이 경로로 쓴 단위와 그 단위가 쓰는 단위만 프로그램에 더해지고, 프로그램 단위와 같은 규칙으로 검사·증명된다. 없는 단위나 없는 이름은 다른 경로와 같이 E0304·E0305 오류다. 받는 꼴은 `tests/run/std_str.bmb`. API는 이 문서의 범위 밖이다.
