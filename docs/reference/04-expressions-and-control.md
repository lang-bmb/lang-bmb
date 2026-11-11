# 04 식·문장·제어 흐름

이 장은 함수 몸 안에 오는 것을 적는다: 식(리터럴, 경로, 호출, 연산자, 변환, 자리, 구조체·열거형 값, 블록, 클로저), 문장(`let`, 대입, 식 문장, `assert`), 제어 흐름(`if`, `match`, `while`, 범위 `for`, `loop`, `break`·`continue`, `return`, `?`), 그리고 받지 않는 Rust 습관과 그 자리에 쓰는 꼴(E02xx 전부)이다. 소유권은 05장, 계약은 06장, 증명 의무는 07장에 있다.

## 식의 문법과 우선순위

```
expr    = "return" [expr] | "break" [label] [expr] | "continue" [label] | binary
binary  = unary { binop unary }
unary   = ("-" | "!" | "*" | "&" ["mut"]) unary | postfix
postfix = primary { "." name | "." name "(" args ")" | "[" expr "]" | "?" }
primary = literal | path | path ["::<" types ">"] "(" args ")" | path "{" name ":" expr {"," name ":" expr} "}"
        | "[" expr {"," expr} "]" | "[" expr ";" N "]" | "(" expr ")" | "()" | block
        | if | match | loop | closure
```

이항 연산자는 느슨한 것부터 `||`, `&&`, 비교(`==` `!=` `<` `<=` `>` `>=`), `|`, `^`, `&`, `<<` `>>`, `+` `-`, `*` `/` `%`, `as` 순으로 묶인다(Rust와 같다, ADR-0048 §2: `a & b == c`는 `(a & b) == c`). `return`·`break`·`continue`가 가장 느슨하다. 명세 전용인 `==>`는 실행 코드에서 E0503이다. 비교는 이어 쓰지 못한다(E0211). 구문 중첩은 256단계까지이고 넘으면 E0107이다. 같은 단계의 연산자는 왼쪽부터 묶인다(`7 - 4 - 2`는 1). 피연산자와 호출 인자는 왼쪽부터 계산한다. 코퍼스: `tests/run/order.bmb`(단계마다 값 하나, 계산 순서), `tests/run/basics.bmb`, `tests/check/err/rust_habits.bmb`.

## 리터럴·경로·호출

- 리터럴의 꼴은 01장에 있다. `()`는 단위 값이다.
- 지역 이름은 `let`·매개변수·`match` 바인딩·`for` 변수가 만든다. 모르는 이름은 E0301이고, 보이는 이름을 가리는 새 이름은 E0303이다.
- 함수 호출은 경로로 쓴다: 같은 단위의 함수는 `f(a)`, 다른 단위의 함수는 뿌리부터 `unit::f(a)`, 타입의 함수는 `T::f(a)`. `use`가 없으므로(E0214) 다른 단위의 이름은 언제나 온전한 경로다. `crate::`·`super::`·`self::`는 E0242, 모르는 함수는 E0304, 인자 개수는 E0405, 인자 타입은 E0401이다.
- 제네릭 함수의 호출은 타입 인자를 적는다: `id::<i64>(x)`. 빠지면 E0237이다(ADR-0046 §3). 이름 없는 타입 자리(함수 값)는 `_`다.

코퍼스: `tests/run/generics.bmb`, `tests/check/err/rust_paths.bmb`, `tests/check/err/generics.bmb`.

## 메서드 호출

`x.f(a, ..)`는 `T::f(조정(x), a, ..)`다(ADR-0051). `T`는 `x`의 타입(참조면 가리키는 타입)이고, `f`는 `T`의 고유 `impl` 함수가 먼저, 없으면 `f`를 가진 trait 하나다. 내장 메서드는 같은 이름의 trait 메서드보다 먼저다. 조정은 한 단계이고 선언된 수신자가 정한다: `self: &T`면 `&x`, `self: &mut T`면 `&mut x`(쓸 수 있는 자리가 아니면 E0408), `self: T`면 `x`. `x`가 이미 참조면 그대로 넘긴다.

두 trait이 같은 이름을 가지면 E0313, 수신자가 없는 연관 함수나 자기 타입 매개변수가 있는 함수를 메서드로 부르면 E0314(둘 다 경로 호출로 쓴다), 모르는 메서드는 E0404다. 코퍼스: `tests/run/methods.bmb`, `tests/run/method_order.bmb`(고유 함수가 trait보다, 내장 메서드가 trait보다 먼저), `tests/check/err/methods.bmb`.

## 연산자와 피연산자

| 연산자 | 피연산자 | 값 |
| --- | --- | --- |
| `+` `-` `*` `/` `%` | 같은 정수 타입 둘, 또는 `f64` 둘(`%` 제외) | 그 타입 |
| 단항 `-` | 부호 있는 정수, `f64`. 부호 없는 정수는 E0411 | 그 타입 |
| `==` `!=` `<` `<=` `>` `>=` | 같은 타입의 정수·`f64`·`char` 둘. `bool`은 `==`·`!=`만 | `bool` |
| `&&` `\|\|` | `bool` 둘. 오른쪽은 필요할 때만 계산한다(`tests/run/order.bmb`) | `bool` |
| `!` | `bool`(논리 부정) 또는 정수(비트 반전) | 그 타입 |
| `&` `\|` `^` | 같은 정수 타입 둘, 또는 `bool` 둘(양쪽을 모두 계산) | 그 타입 |
| `<<` `>>` | 미는 값은 정수, 양은 어느 정수 타입이나 | 미는 값의 타입 |

- 폭이나 부호가 다른 정수를 섞으면 E0401이다. 암묵적 넓힘은 없고 `x as T`로 맞춘다(ADR-0048 §1).
- 구조체·열거형의 `==`는 명세에서만 뜻이 있다. 실행 코드에서는 E0425이고 필드를 비교하거나 `match`한다.
- 넘침, 0으로 나누기, 폭 이상의 시프트 양은 타입 오류가 아니라 증명 의무다(07장).

복합 대입 `t op= e;`(`+=` `-=` `*=` `/=` `%=` `&=` `|=` `^=` `<<=` `>>=`)는 `t = t op e;`의 입력 별칭이다(ADR-0095). 정규형은 펼친 꼴이다. 자리 `t`가 함수를 부르면 E0103이다. 코퍼스: `tests/run/bits.bmb`, `tests/check/err/bits.bmb`, `tests/check/err/widths.bmb`.

## 변환

- `x as T`는 정수 타입 사이의 변환이다(ADR-0028, ADR-0095 A4). 값을 그대로 두고, 대상이 원본의 범위를 담지 못하면 범위 의무가 생긴다(07장). `-x as u64`는 `(-x) as u64`다.
- `T::try_from(x).unwrap()`은 `x as T`의 입력 별칭이다. `T::try_from(x).ok()`는 `Option<T>`이고 의무가 없다. 그 밖의 `try_from` 쓰임과 정수가 아닌 원본(`bool`, `char`)은 E0412다.
- 정수에서 `f64`는 `f64::from(x)`다(ADR-0061). `x as f64`와 정수가 아닌 대상의 `as`는 E0205다. `f64`에서 정수로 가는 변환은 없다.
- `T::from(x)`·`x.into()`·`x.try_into()`는 E0212, `T::try_from(x).expect(..)`는 E0213이다.

코퍼스: `tests/run/conversions.bmb`, `tests/check/err/conversions.bmb`, `tests/check/err/float_literals.bmb`.

## 색인·필드·역참조

- `a[i]`: `a`는 배열·슬라이스·`Vec`, `i`는 `usize`다(다른 정수 타입은 E0401, 색인할 수 없는 값은 E0407). 범위 안임은 증명 의무다(07장).
- `p.x`: 구조체 값이나 그 참조의 필드다. 참조를 거친 읽기도 `p.x`로 쓴다(`(*p).x`는 E0231). 없는 필드는 E0306이다.
- `*r`은 참조가 가리키는 값, `&e`·`&mut e`는 인자 자리의 빌림이다(05장).
- 대입할 수 있는 자리는 변수·필드·원소·역참조를 이은 경로다: `g[i][j]`, `b.rows[i].cells[j]`, `*r`.

코퍼스: `tests/run/arrays.bmb`, `tests/run/fields.bmb`, `tests/check/err/places.bmb`.

## 구조체·열거형 값

- 구조체 값은 `T { a: e1, b: e2 }`다. 모든 필드를 선언 순서로 적는다: 빠지거나 겹치면 E0422, 순서가 다르면 E0434(적힌 순서가 평가 순서다, ADR-0080). 축약 `T { x }`는 E0222, 갱신 구문 `..p`는 E0223이다.
- 열거형 값은 `T::V` 또는 `T::V(e, ..)`다. 페이로드 개수가 다르면 E0405, 없는 변형은 E0307이다. 내장 변형은 맨이름으로 쓴다: `Some(e)`, `None`, `Ok(e)`, `Err(e)`(`Option::Some`은 E0221).
- 배열 값은 `[a, b, c]`와 `[v; N]`이다(03장). 튜플은 값도 타입도 없다(E0233, `tests/check/err/reserved_names.bmb`).

코퍼스: `tests/run/shapes.bmb`, `tests/run/points.bmb`, `tests/check/err/data_values.bmb`, `tests/check/err/field_order.bmb`.

## 블록과 꼬리 식

```
block = "{" { stmt } [ expr ] "}"
```

블록은 식이다. 값은 마지막의 `;` 없는 식(꼬리 식)의 값이고, 꼬리 식이 없으면 `()`다. 끝나지 않는 문장(`return`, `break`, `continue`)으로 끝나는 꼬리 없는 블록의 타입은 `!`이고 어떤 기대 타입에도 맞는다(ADR-0049). 함수 몸은 블록이고 그 값이 함수의 결과다. 블록 안의 이름은 밖에서 보이지 않는다(E0301). 코퍼스: `tests/run/scopes.bmb`, `tests/run/returns.bmb`.

## 클로저

```
closure = ["move"] "|" [ name ":" type {"," name ":" type} ] "|" "->" ret [requires ..,] [ensures ..,] block
```

클로저는 `Fn`·`FnMut`·`FnOnce` 경계를 가진 타입 매개변수의 인자 자리에만 온다(ADR-0053). `let`에 묶거나 돌려주면 E0264다. 매개변수와 결과의 타입을 적는다(E0263). 바깥 지역을 캡처하면 `move`를 쓴다(E0262): 캡처는 클로저 안으로 이동한다. 함수 항목도 같은 자리에 넘긴다. 받은 함수 값은 `f(a)`로 부른다.

```
#[effects(pure)]
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 { f(x) }

#[effects(pure)]
fn shifted(k: i64) -> i64 {
    apply::<_>(move |x: i64| -> i64 { x + k }, 1)
}
```

코퍼스: `tests/run/fn_values.bmb`, `tests/check/err/fn_values.bmb`.

## 문장

```
stmt = "let" ["mut"] name ":" type "=" expr ";"
     | place "=" expr ";"  |  place op"=" expr ";"
     | expr ";"
     | "assert" "(" expr ")" ["by" "(" "bit_vector" ")"] ";"
     | while | for
```

- `let`은 타입과 초깃값이 필수다. 타입이 없으면 E0201이다. 패턴 `let`·`let .. else`는 E0226, 구조 분해 `let T { x, y } = p;`는 E0230이다.
- 대입은 `let mut` 지역과 `&mut`을 거친 자리에만 한다. `let`으로 묶은 이름, 매개변수, `for` 변수에 대입하면 E0408이다(`mut` 매개변수는 E0210: `let mut` 지역에 복사한다). 대입은 식이 아니다(E0103).
- 식 문장 `e;`는 값을 버린다. 문장 자리의 `if`·`match`·`loop`는 닫는 `}`에서 끝난다(`if c { return; } *x = 1;`은 두 문장).
- `assert(e);`의 `e`는 `bool`이다. 실행 시점의 검사가 아니라 증명 의무다(07장). `assert!(..)` 같은 매크로는 E0202다. `by` 뒤의 증명기는 `bit_vector` 하나다(E0516).

코퍼스: `tests/check/err/rust_habits.bmb`, `tests/check/err/habits.bmb`, `tests/verify/pass/conditions.bmb`, `tests/verify/pass/bit_vector.bmb`.

## if와 match

```
if    = "if" expr block [ "else" (if | block) ]
match = "match" expr "{" { pat "=>" expr "," } "}"
pat   = path | path "(" bind {"," bind} ")"        bind = name | "_"
```

- `if`의 조건은 `bool`이다(E0401). 두 갈래의 타입이 같고 그것이 식의 타입이다. `else`가 없으면 타입은 `()`다. `if let`은 E0225다.
- `match`의 대상은 열거형 값 또는 그 참조다(그 밖은 E0423: `if`로 가른다). 참조를 `match`하면 바인딩도 참조다(ADR-0055).
- 갈래는 변형마다 정확히 하나씩, 선언 순서로 쓴다. 빠진 변형, 두 번 쓴 변형, 다른 순서는 모두 E0426이다. 와일드카드 갈래 `_ =>`는 E0224다(망라성은 변형을 적어서 보인다, ADR-0013).
- 패턴은 변형 하나와 그 페이로드 자리마다 새 이름이나 `_`다: `Shape::Dot`, `Shape::Rect(a, _)`, `Some(v)`, `None`. 자리 수가 다르면 E0427, 변형 경로가 아니면 E0307이다. 가드, or-패턴, 리터럴·튜플·참조 패턴, 중첩 패턴, 구조체 패턴은 E0232다.
- 갈래의 타입은 모두 같다. 타입이 `!`인 갈래(`None => return d,`)는 다른 갈래의 타입을 따른다.

```
#[effects(pure)]
fn pick(s: S, d: i64) -> i64 {
    match s {
        S::A => d,
        S::B(n) => n,
        S::C(_) => return 0,
    }
}
```

(`enum S { A, B(i64), C(P) }`) 코퍼스: `tests/run/shapes.bmb`, `tests/run/mut_match.bmb`, `tests/check/err/match.bmb`.

## 루프

```
while = [label ":"] "while" expr [invariant e {"," e} ","] "decreases" e "," block
for   = [label ":"] "for" name "in" expr (".." | "..=") expr [invariant e {"," e} ","] block
loop  = [label ":"] "loop" [invariant e {"," e} ","] "decreases" e "," block
```

- 문법은 정규형의 것이다: 절 끝의 쉼표와 블록 몸 갈래 뒤의 쉼표는 입력에서 빠져도 받고 정규형이 쓴다.
- `while`의 조건은 `bool`이다. `while`과 `loop`는 `decreases`가 필수다(E0506). 불변식과 감소식의 뜻은 06장과 07장에 있다. `while let`은 E0226이다.
- `for`는 정수 범위만 돈다(ADR-0049 §2). `lo..hi`는 `hi`를 빼고 `lo..=hi`는 넣는다. 변수의 타입은 두 끝의 정수 타입이고(둘 다 리터럴이면 `i64`), 두 끝은 들어갈 때 왼쪽부터 한 번 계산한다(`tests/run/order.bmb`). 변수는 루프 안에서만 보이고(밖에서는 E0301) 대입할 수 없다(E0408). 반복자(`for x in v.iter()`)는 E0252, `decreases`는 E0253이다(측도는 자동). 정수가 아닌 끝은 E0401이다.
- `while`과 `for`는 문장이고 값이 없다. `loop`는 식이다: 값은 `break e`의 값이고, 모든 `break`의 값은 한 타입이다. `break`가 없는 `loop`의 타입은 `!`다.

```
#[effects(pure)]
fn first(a: &[i64], t: i64) -> Option<usize> {
    'scan: for i in 0..a.len() {
        if a[i] == t { return Some(i); }
        if a[i] > t { break 'scan; }
    }
    None
}
```

코퍼스: `tests/run/ranges.bmb`, `tests/run/loop_exits.bmb`, `tests/check/err/ranges.bmb`, `tests/verify/pass/loops.bmb`.

## 조기 탈출: return, break, continue, ?

- `return e`·`return`은 함수를 떠난다. `e`의 타입은 함수의 결과 타입이다. 식 자리에도 오며 타입은 `!`다.
- `break`·`continue`는 가장 안쪽 루프에, `break 'l`·`continue 'l`은 레이블 `'l`이 붙은 둘레 루프에 작용한다. 루프 밖에서는 E0250, 둘레에 없는 레이블은 E0317이다. 값 있는 `break e`는 `loop`만 떠난다(`while`·`for`에서는 E0251).
- `x?`: `x`가 `Option<T>`이고 함수 결과가 `Option<U>`이면 `None`을, `x`가 `Result<T, E>`이고 결과가 같은 `E`의 `Result<U, E>`이면 `Err(e)`를 돌려주고, 그 밖에서 값은 `T`다. 다른 오류 타입, `Option`과 `Result`의 혼용, 그 밖의 결과 타입은 E0241이다(암묵적 변환이 없다). `.unwrap()`·`.expect()`는 E0229다.

코퍼스: `tests/run/returns.bmb`, `tests/run/tries.bmb`, `tests/check/err/loop_exits.bmb`, `tests/check/err/tries.bmb`.

## 받지 않는 Rust 습관

아래 꼴은 프로그램이 아니다(ADR-0013: 선택을 필수로). 진단은 대신 쓸 꼴을 `fix`에 싣고, 기계적으로 정해지는 것은 대체 문구(`replacement`)도 싣는다. 예는 `tests/check/err/rust_habits.bmb`, `tests/check/err/habits.bmb`와 표 아래의 파일에 있다.

### 선언과 시그니처

| 코드 | 받지 않는 꼴 | 대신 쓰는 꼴 |
| --- | --- | --- |
| E0201 | `let y = e;` | `let y: T = e;` |
| E0209 | 반환 타입 없는 `fn f()` | `fn f() -> ()` |
| E0210 | `fn f(mut x: T)` | 몸 안에서 `let mut` 지역에 복사 |
| E0208 | 수명 `'a` | 지운다(참조는 수명을 적지 않는다) |
| E0228 | `&self`, `&mut self`, `self` | `self: &T`, `self: &mut T`, `self: T` |
| E0220 | `#[derive(..)]` | 지운다 |
| E0261 | `static X`, `static mut X` | `const X`; 전역 가변 상태는 인자로 넘긴다 |
| E0207 | `pub`, const 제네릭, 제네릭 trait, 상위 trait, 단위가 담지 않는 항목, `spec fn`의 계약 절, trait 메서드의 `decreases` | `pub`은 지운다; 길이는 슬라이스와 `len()`; 타입 매개변수는 메서드에; 경계는 `T: A + B` |

코퍼스: `tests/check/err/items.bmb`, `tests/check/err/unsupported.bmb`, `tests/check/err/consts.bmb`.

### 이름·경로·매크로·탈출구

| 코드 | 받지 않는 꼴 | 대신 쓰는 꼴 |
| --- | --- | --- |
| E0214 | `use a::f;`와 그것으로 줄인 이름 | 쓰는 곳에 온전한 경로 `a::f(..)` |
| E0242 | `crate::`, `super::`, `self::` | 뿌리부터의 단위 경로 `unit::f` |
| E0221 | `Option::Some`, `Result::Ok` 등 | `Some`, `None`, `Ok`, `Err` |
| E0237 | 타입 인자 없는 제네릭 호출 `id(x)` | `id::<i64>(x)` |
| E0202 | 매크로 `name!(..)` | `println!`은 `print_i64(x)`; `vec!`은 `[a, b]`·`[v; N]`, `Vec::with_capacity(n)`과 `push`; `assert!`는 `assert(c);`; `panic!` 류는 그 경우가 없음을 증명 |
| E0215 | `unsafe` | 증명할 수 없는 몸은 근거를 붙인 `#[external_body]` 함수(08장) |
| E0216 | `#[verifier::external_body]` | `#[external_body]` |
| E0217 | `admit()` | 근거를 붙인 `assume(p(..));`(08장) |

코퍼스: `tests/check/err/rust_paths.bmb`, `tests/check/err/escapes.bmb`.

### 변환과 연산자

| 코드 | 받지 않는 꼴 | 대신 쓰는 꼴 |
| --- | --- | --- |
| E0205 | `x as f64`, 정수가 아닌 대상의 `as` | `f64::from(x)`; 정수 사이는 `x as T` |
| E0212 | `T::from(x)`, `x.into()`, `x.try_into()` | `x as T` |
| E0213 | `T::try_from(x).expect("..")` | `T::try_from(x).unwrap()`, 정본은 `x as T` |
| E0218 | 슬라이스·`&str` 자리에 `Vec`·`String` | `v.as_slice()`, `s.as_str()` |
| E0211 | `a < b < c` | `a < b && b < c` |

코퍼스: `tests/check/err/conversions.bmb`, `tests/check/err/strings.bmb`, `tests/check/err/vec.bmb`.

### 데이터와 패턴

| 코드 | 받지 않는 꼴 | 대신 쓰는 꼴 |
| --- | --- | --- |
| E0222 | `Point { x, y }` | `Point { x: x, y: y }` |
| E0223 | `Point { x: 1, ..p }` | 모든 필드: `Point { x: 1, y: p.y }` |
| E0224 | `_ => e` | 변형마다 갈래 하나(수정이 선언 순서의 `match` 전체를 싣는다) |
| E0225 | `if let Some(v) = o { a } else { b }` | `match o { None => { b }, Some(v) => { a } }` |
| E0226 | `while let`, `let .. else`, 패턴 `let` | 조건으로 도는 `while`과 안쪽의 `match`; 값을 `match` |
| E0229 | `o.unwrap()`, `o.expect(..)` | 두 경우를 `match`: `match o { None => .., Some(v) => v }` |
| E0230 | `let Point { x, y } = p;` | `let x: i64 = p.x; let y: i64 = p.y;` |
| E0231 | `(*p).x` | `p.x` |
| E0232 | 가드, or-패턴, 리터럴·튜플·참조·중첩·구조체 패턴 | 갈래 몸의 `if`; 변형마다 갈래; 이름을 묶고 몸에서 `match`; 필드는 `p.x` |
| E0233 | 튜플, 튜플 구조체, 구조체 꼴 변형 | 이름 붙은 필드의 구조체; 변형은 `V(Data)` |

### 제어 흐름

| 코드 | 받지 않는 꼴 | 대신 쓰는 꼴 |
| --- | --- | --- |
| E0241 | 함수가 돌려줄 수 없는 `None`·`Err`의 `?` | `match`하고 `return Err(..)`로 바꾸어 돌려준다 |
| E0250 | 루프 밖의 `break`·`continue` | 루프 안에서 쓴다; 함수를 떠나려면 `return` |
| E0251 | `while`·`for`의 `break e` | 루프 앞의 변수에 대입한 뒤 `break` |
| E0252 | `for x in v.iter()`, `for x in &v` | `for i in 0..v.len()`과 `v[i]` |
| E0253 | `for`의 `decreases` | 지운다(범위가 루프를 묶는다) |

### 소유·제네릭·함수 값

| 코드 | 받지 않는 꼴 | 대신 쓰는 꼴 |
| --- | --- | --- |
| E0219 | `drop(v)`, `std::mem::take` | 값은 블록 끝에서 해제된다; 바꾸려면 새 값을 대입 |
| E0238 | `Rc`, `RefCell`, `Cell`, `Arc` | 소유 트리(`Box`, `Vec`) 또는 `Vec`의 색인 |
| E0239 | `&mut` 뒤의 소유 값을 이동 | `std::mem::replace(&mut r.f, 새 값)`, `std::mem::swap` |
| E0235 | `impl<T> Trait for T` | 타입마다 `impl`, 또는 쓰는 곳의 경계 `T: Trait` |
| E0236 | `dyn Trait` | trait 경계의 타입 매개변수, 또는 닫힌 변형의 열거형 |
| E0240 | `where T: Copy` | 매개변수 옆의 경계 `<T: Copy>` |
| E0262 | 캡처하는 클로저에 `move` 없음 | `move \|x: T\| -> R { .. }` |
| E0263 | 타입 없는 클로저 `\|x\| x + 1` | `\|x: T\| -> R { .. }` |
| E0264 | 함수 포인터 타입 `fn(A) -> R`, 돌려주는 클로저, 인자 아닌 자리의 클로저 | 타입 매개변수 `F: Fn(A) -> R`의 인자로 직접 넘긴다 |
| E0265 | 인자 위치 `f: impl Fn(A) -> R` | `<F: Fn(A) -> R>`과 `f: F` |

코퍼스: `tests/check/err/mem_moves.bmb`, `tests/check/err/boxes.bmb`, `tests/check/err/traits.bmb`, `tests/check/err/fn_values.bmb`.

### 명세·유령 코드·상수

규칙은 06장과 02장에 있다.

| 코드 | 받지 않는 꼴 | 대신 쓰는 꼴 |
| --- | --- | --- |
| E0254 | `#[invariant(p)]`의 `p`가 `spec fn p(x: &T) -> bool`이 아님 | 그 꼴의 `spec fn`을 선언한다 |
| E0255 | 한 타입에 `#[invariant(..)]` 둘 이상 | 술어 하나에 `&&`로 잇는다 |
| E0256 | `proof fn`의 효과 속성이나 반환 타입 | 지운다(증명하는 것은 `ensures`다) |
| E0257 | 유령 코드 안의 실행 호출, 대입, `let mut`, `&mut`, 루프 | 호출은 `proof` 블록 밖에서; 값은 `let`으로 묶는다 |
| E0258 | `proof` 블록 밖에서 부른 `proof fn` | `proof { f(x); }` |
| E0259 | `const` 값에 든 함수 호출 등 상수 식이 아닌 것 | 리터럴, 다른 `const`, 연산자, `if`, `match`, `as`, 배열·구조체·변형(ADR-0052 §3) |
| E0260 | `const` 계산의 넘침, 0으로 나누기, 범위 밖 변환·색인 | 값을 고치거나 그 값을 담는 타입을 준다 |

코퍼스: `tests/check/err/invariants.bmb`, `tests/check/err/proofs.bmb`, `tests/check/err/consts.bmb`.
