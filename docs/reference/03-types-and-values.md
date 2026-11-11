# 03 타입과 값

이 장은 값이 가질 수 있는 타입과 그 타입 위의 연산을 적는다: 스칼라 타입, 배열·슬라이스·텍스트, 구조체와 열거형, 제네릭과 trait, 함수 값, 소유 타입, 변환, 상수의 타입, 그리고 검사기가 타입을 맞추는 규칙이다. 값이 이동하는지 복사되는지와 참조의 규칙은 05장, 넘침·경계·변환 범위의 증명 의무는 07장에 있다.

## 스칼라 타입

| 타입 | 값 | 비고 |
| --- | --- | --- |
| `i8` `i16` `i32` `i64` `isize` | 부호 있는 정수 | `isize`는 64비트 |
| `u8` `u16` `u32` `u64` `usize` | 부호 없는 정수 | `usize`는 64비트 |
| `f64` | IEEE 754 배정밀도 | 부동소수는 `f64` 하나 |
| `bool` | `true`, `false` | |
| `char` | 유니코드 스칼라 값 | `'a'`, `'\u{1F600}'` |
| `()` | 값 하나 | 결과가 없는 함수의 반환 타입 |

- `i128`·`u128`·`f32`는 타입이 아니다(E0402, ADR-0048·ADR-0061).
- 정수 리터럴에는 접미사가 없고(E0104) 타입은 기대 타입에서 온다. 그 타입에 들어가지 않는 리터럴은 E0406 오류다(`let z: u8 = 256;`). `-128`은 `i8`의 리터럴 하나다.
- 부호 없는 값에 단항 `-`를 쓰면 E0411 오류다.
- 정수 타입마다 상수 `MIN`·`MAX`가 있다(`i64::MAX`).
- 코퍼스: `tests/run/widths.bmb`, `tests/run/floats.bmb`, `tests/check/err/widths.bmb`.

## 타입을 적는 자리

타입은 적는다. 검사기가 추론하는 것은 리터럴의 타입과, 생성 식의 타입 인자뿐이다.

| 자리 | 규칙 | 어기면 |
| --- | --- | --- |
| `let` | `let x: T = e;` | E0201 |
| 함수 결과 | `-> T`, 결과가 없으면 `-> ()` | E0209 |
| 클로저의 매개변수와 결과 | `\|x: T\| -> R { .. }` | E0263 |
| `const` | `const N: T = e;` | E0103 |
| 제네릭 호출의 타입 인자 | `f::<i64>(x)`, `Cell::<u64>::new(v)` | E0237 |
| 구조체 생성·변형 값의 타입 인자 | 쓰지 않는다. 기대 타입이나 필드·페이로드 값에서 정한다 | E0301(쓴 경우), E0424(정할 수 없는 경우) |

- 함수 값을 넘기는 호출만 타입 인자 자리에 `_`를 쓴다(`apply::<_>(inc, 10)`).
- 코퍼스: `tests/check/err/types.bmb`, `tests/check/err/data_values.bmb`.

## 연산자와 피연산자

기대 타입과 식의 타입이 다르면 E0401 오류다. 암묵적 변환은 없다.

| 연산 | 피연산자 | 결과 |
| --- | --- | --- |
| `+` `-` `*` `/` | 같은 정수 타입 둘, 또는 `f64` 둘 | 그 타입 |
| `%` | 같은 정수 타입 둘 | 그 타입 |
| 단항 `-` | 부호 있는 정수, `f64` | 그 타입 |
| `&` `\|` `^` | 같은 정수 타입 둘, 또는 `bool` 둘 | 그 타입 |
| `!` | `bool`(논리 부정), 정수(비트 반전) | 그 타입 |
| `<<` `>>` | 미는 값은 정수, 양은 어느 정수 타입이나 | 미는 값의 타입 |
| `==` `!=` | 같은 정수 타입, `bool`, `char`, `f64`, 텍스트 | `bool` |
| `<` `<=` `>` `>=` | 같은 정수 타입, `char`, `f64`, 텍스트 | `bool` |
| `&&` `\|\|` | `bool` 둘 | `bool` |

- 섞인 폭(`i32 + i64`), `bool`의 `<`, `char`의 `+`, `f64`의 `%`, 조건 자리의 정수도 같다.
- 인덱스의 타입은 `usize`다(E0401).
- 구조체·열거형·배열·`Box`의 `==`는 명세에만 있다(E0425, 06장). `Vec`의 `==`는 E0514 오류다.
- 타입 매개변수 `T`의 값은 실행 코드에서 `T: Eq`가 있어야 `==`·`!=`를, `T: Ord`가 있어야 순서 비교를 쓴다(E0428).
- 정수 타입의 메서드(`wrapping_add`, `saturating_sub`, `checked_mul`, `rotate_left`, `count_ones` 등)와 `f64`의 메서드(`sqrt`·`abs`·`min`·`max`·`is_nan`·`is_finite`)는 닫힌 목록이다. 없는 메서드는 E0404, 메서드나 함수의 인자 개수가 다르면 E0405 오류다.
- 넘침, 0으로 나누기, 시프트 양의 범위는 증명 의무다(07장).
- 코퍼스: `tests/run/bits.bmb`, `tests/run/arith_methods.bmb`, `tests/check/err/bits.bmb`, `tests/check/err/floats.bmb`.

## 변환

| 꼴 | 뜻 |
| --- | --- |
| `x as T` | 정수에서 정수 타입 `T`로. 값은 그대로이고, `T`가 원본의 범위를 담지 못하면 범위 의무가 생긴다(07장) |
| `T::try_from(x).ok()` | 정수에서 `Option<T>`. 의무 없음 |
| `f64::from(x)` | 정수에서 `f64` |
| `u32::from(c)` | `char`에서 `u32` |
| `char::from_u32(x)` | `u32`에서 `Option<char>` |

- `as`의 대상이 정수 타입이 아니면 E0205 오류다(`u as char`, `x as f64`).
- `as`와 `try_from`의 원본이 정수가 아니거나(`bool`, `char`, `f64`), `T::try_from(x)`를 `.ok()`·`.unwrap()` 없이 쓰면 E0412 오류다.
- `T::from(x)`·`x.into()`·`x.try_into()`는 E0212 오류다. 명세 안의 변환은 E0513 오류다(명세에서는 정수 타입이 섞인다, 06장).
- 코퍼스(ADR-0028): `tests/run/conversions.bmb`, `tests/check/err/conversions.bmb`.

## 배열과 슬라이스

- `[T; N]`은 길이가 타입에 든 배열이다. 값은 `[a, b, c]`와 `[v; N]`, 읽기는 `a[i]`, 길이는 `a.len()`(`usize`)이다. `N`은 10진 리터럴이나 `usize` 상수이고, 너무 큰 길이는 E0406 오류다.
- 원소는 스칼라, 구조체·열거형, `Box`, 고정 배열(`[[T; M]; N]`)이다. `Vec`과 `String`은 배열의 원소가 아니다(E0402).
- `&[T]`·`&mut [T]`는 슬라이스 참조이고 매개변수의 타입으로만 쓴다(05장). 참조 없는 `[T]`는 타입이 아니다(E0402). 인자는 `&arr`·`&mut arr`, `v.as_slice()`·`v.as_mut_slice()`, 받은 슬라이스 매개변수, 정수 원소의 부분 `&v[lo..hi]`다.
- 인덱스할 수 없는 값(`i64`, `&str`)은 E0407 오류다.
- 범위 식 `lo..hi`는 값이 아니다. `for`의 범위, `copy_within`의 첫 인자, `&v[lo..hi]`·`&s[lo..hi]` 인자에만 오고 다른 자리는 E0435 오류다.
- 코퍼스: `tests/run/arrays.bmb`, `tests/check/err/data_arrays.bmb`, `tests/check/err/places.bmb`, `tests/check/err/subslices.bmb`.

## 텍스트

- `String`은 소유하는 UTF-8 텍스트, `&str`은 빌린 텍스트, `char`는 글자 하나다(ADR-0050).
- 리터럴 `"..."`의 타입은 `&str`이고, 호출 인자와 비교의 피연산자 자리에만 온다. 그 밖의 자리(`let t: String = "x";`)는 E0409 오류다. 소유하려면 `String::from("..")`을 쓴다.
- `&str` 자리의 인자는 리터럴, `s.as_str()`, `&s[a..b]`, 받은 `&str` 매개변수다. `String` 값이나 `&s`를 그 자리에 쓰면 E0218 오류다. `&mut str`은 타입이 아니다(E0402).
- 연산: `String::new()`·`with_capacity(n)`·`from(t)`·`from_utf8(v)`, `push_str`·`push`·`clone`, `len`·`is_char_boundary`, `as_str`·`as_bytes`, `c.len_utf8()`. 비교는 바이트 사전순이다.
- 텍스트를 인덱스하면 E0407(`s.as_bytes()[i]`로 바이트를 읽는다), `to_string`·`chars` 같은 목록 밖의 메서드는 E0404, `char`의 산술은 E0401 오류다.
- 코퍼스: `tests/run/strings.bmb`, `tests/check/err/strings.bmb`, `tests/check/err/bytes.bmb`.

## 구조체와 열거형

```
struct Pt { x: i64, y: i64 }
enum Shape { Dot, Sq(i64) }
#[effects(pure)]
fn side(s: Shape, p: Pt) -> i64 {
    let q: Pt = Pt { x: p.y, y: 0 };
    match s {
        Shape::Dot => q.x,
        Shape::Sq(n) => n,
    }
}
```

- 구조체는 이름 붙은 필드를 하나 이상, 열거형은 변형을 하나 이상 가진다(E0420). 변형의 페이로드는 위치로 적은 값이다. 튜플, 튜플 구조체, 필드 이름을 가진 변형은 없다(E0233).
- 필드와 페이로드는 값 타입이다. 참조와 슬라이스는 담지 못한다(E0403, 05장).
- 타입이 자기를 값으로 담으면 E0421 오류다. `Box`나 `Vec` 아래에서는 담는다(`Cons(i64, Box<List>)`). 유한한 값이 없는 타입, 자기를 다른 타입 인자로 담는 타입도 E0421 오류다.
- 생성 식 `T { f: e, .. }`는 모든 필드를 한 번씩, 선언 순서로 적는다. 구조체가 아니거나 필드가 빠지거나 두 번 나오면 E0422, 순서가 다르면 E0434(적힌 순서가 평가 순서다, ADR-0080), 없는 필드는 E0306 오류다.
- `match`의 피검사 값은 열거형이다(E0423). 갈래는 변형마다 하나씩, 선언 순서로 쓴다(E0426). 패턴은 `T::V`나 `T::V(a, b)`이고 이름의 개수는 페이로드의 개수다(E0427). 없는 변형은 E0307, 그 밖의 패턴 꼴은 E0232 오류다.
- 타입 불변식: `#[invariant(p)]`를 구조체·열거형 앞에 쓴다. `p`는 그 타입의 `&T`를 받아 `bool`을 돌려주는 `spec fn`이다(E0254). 타입마다 하나다(E0255). 뜻과 의무는 06·07장(ADR-0060).
- `std::process::Child`는 생성 식으로 만들거나 필드로 읽지 못한다(E0436).
- 자기를 담고 `Vec`을 가진 타입의 `==`(E0431)와 그런 구조체의 생성 식(E0432)은 명세에 쓰지 못한다.
- 코퍼스: `tests/run/shapes.bmb`, `tests/run/points.bmb`, `tests/check/err/data_types.bmb`, `tests/check/err/match.bmb`, `tests/check/err/field_order.bmb`, `tests/check/err/rec_structs.bmb`, `tests/check/err/child.bmb`.

## `Option`과 `Result`

- `Option<T>`(`None`, `Some(v)`)와 `Result<T, E>`(`Ok(v)`, `Err(e)`)는 내장 열거형이다. 이 여섯 이름과 `Box`는 다시 정의하지 못한다(E0309). 타입 인자는 값 타입이어서 `Option<&T>`와 `Result<&str, E>`는 E0403 오류다(`tests/check/err/attr_places.bmb`). 타입 인자의 개수가 다르면 E0420 오류다.
- 변형은 경로 없이 쓰고(`Option::Some`은 E0221), 갈래의 순서는 `None`·`Some`, `Ok`·`Err`다(E0426).
- `.unwrap()`·`.expect()`는 없다(E0229). `match`로 두 경우를 쓰거나 `?`를 쓴다(04장).
- 코퍼스: `tests/run/tries.bmb`, `tests/check/err/tries.bmb`, `tests/check/err/data_values.bmb`.

## 제네릭과 trait

```
trait Area { #[effects(pure)] fn area(self: &Self) -> i64; }
struct Sq { s: i64 }
impl Area for Sq { #[effects(pure)] fn area(self: &Sq) -> i64 { self.s } }
#[effects(pure)]
fn larger<T: Copy + Ord>(a: T, b: T) -> T { if a < b { b } else { a } }
#[effects(pure)]
fn both<T: Area>(x: &T, q: Sq) -> i64 { larger::<i64>(x.area(), q.area()) }
```

- 함수·구조체·열거형·`impl`은 타입 매개변수를 가진다(`<T>`, `<T: Copy + Ord>`, ADR-0046). 경계는 매개변수 옆에 적는다(`where`는 E0240).
- 내장 경계는 `Copy`·`Eq`·`Ord`다. `Copy`가 없는 매개변수의 값은 이동한다(05장). 정수 타입·`bool`·`char`·`String`은 `Eq`와 `Ord`를 가지고, `f64`와 구조체·열거형은 갖지 않는다.
- 타입 인자는 값 타입이다(참조는 E0403). 개수가 다르면 호출은 E0429, 타입은 E0420 오류다. 인자가 경계를 채우지 못하면 E0428 오류다(`Vec<i64>`는 `Copy`가 아니다). 정의가 `Vec<T>`로 쓰는 `T`에 `Vec`의 원소가 못 되는 타입을 넣어도 E0428 오류다.
- 재귀 호출이 자기 타입 매개변수를 더 큰 타입으로 감싸면 E0430 오류다(`f<T>` 안의 `f::<[T; 2]>`).
- trait은 메서드의 시그니처를 선언하고 `impl Tr for T { .. }`가 구현한다. `Self`는 구현하는 타입이다. 선언과 구현의 일치는 02장.
- 한 trait의 impl 둘이 같은 타입일 수 있으면 E0311 오류다. 타입 매개변수는 무엇과도 겹친다고 본다. 모든 타입에 대한 `impl<T> Tr for T`는 E0235, `dyn Tr`은 E0236 오류다. `impl`의 타입 매개변수는 구현하는 타입에 순서대로 나와야 한다(E0420).
- 메서드 호출 `x.f(a)`는 `x`의 타입의 고유 `impl` 함수, 없으면 `f`를 가진 trait 하나를 가리킨다(ADR-0051). 없는 이름은 E0404, 두 trait이 같은 이름을 가지면 E0313, 수신자가 없는 함수는 E0314 오류다.
- 코퍼스: `tests/run/generics.bmb`, `tests/run/traits.bmb`, `tests/run/methods.bmb`, `tests/check/err/generics.bmb`, `tests/check/err/traits.bmb`, `tests/check/err/vec_params.bmb`.

## 함수 값

- 함수를 받는 매개변수는 경계가 `Fn(A, ..) -> R`·`FnMut(..) -> R`·`FnOnce(..) -> R`인 타입 매개변수다: `fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64`(ADR-0053). 몸에서 `f(x)`로 부른다.
- 인자는 함수 항목의 이름이나 클로저 `|x: T| -> R { .. }`이고, 타입 인자는 `_`로 쓴다.
- 클로저는 그 자리에만 온다. `let`에 묶은 클로저, 함수 포인터 타입 `fn(i64) -> i64`, 돌려주는 `impl Fn`은 E0264, 인자 위치의 `impl Fn(..)`은 E0265 오류다.
- 캡처의 규칙은 05장.
- 코퍼스: `tests/run/fn_values.bmb`, `tests/check/err/fn_values.bmb`.

## 소유 타입

| 타입 | 담는 것 | 만들기 | 읽기와 쓰기 |
| --- | --- | --- | --- |
| `Vec<T>` | 스칼라, 배열 없는 구조체·열거형, 그것들의 `Vec`·`Box`, `String` | `Vec::new()`, `Vec::with_capacity(n)` | `v[i]`, `len`, `push`, `pop`(`Option<T>`), `truncate`, `clone` |
| `String` | UTF-8 바이트 | 위 "텍스트" | 위 "텍스트" |
| `Box<T>` | 값 타입(스칼라, 구조체·열거형, 배열, `Box`, `Vec`) | `Box::new(v)` | `*b`, `b.f`, `*b = v` |

- `Vec`의 원소가 못 되는 타입(배열, 배열을 가진 구조체), `Box<()>`는 E0402 오류다(ADR-0047).
- `Vec::new()`의 원소 타입은 기대 타입에서 온다(없으면 E0401).
- 목록에 없는 메서드(`is_empty`, `first`, `get`, `iter`)는 E0404, 이동하는 원소의 `Vec`에 대한 `clone`도 E0404 오류다.
- `spec fn`은 `Vec`이나 `String`을 받거나 돌려주지 않는다(E0402). 슬라이스와 `&str`을 받는다.
- `Rc`·`RefCell`·`Cell`·`Arc`는 없다(E0238).
- 코퍼스: `tests/run/vecs.bmb`, `tests/run/boxes.bmb`, `tests/check/err/vec.bmb`, `tests/check/err/boxes.bmb`, `tests/check/err/nested_vecs.bmb`.

## 상수의 타입

- `const NAME: T = e;`의 `T`는 값 타입이다: 정수, `bool`, `char`, 그리고 그것들의 배열·구조체·열거형(`Option`·`Result` 포함). `Vec`·`String`·`Box`·참조를 담으면 E0437 오류다(ADR-0052).
- 초기화 식은 선언한 타입으로 검사한다(E0401, E0406, E0422).
- `usize` 상수는 배열 길이 `[T; N]`과 반복 `[e; N]`의 `N` 자리에 쓴다.
- 코퍼스: `tests/run/consts.bmb`, `tests/run/consts_data.bmb`, `tests/check/err/consts.bmb`.
