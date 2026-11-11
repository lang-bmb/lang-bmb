# 05 소유권과 참조

이 장은 값의 소유를 적는다: 어떤 타입의 값이 복사되고 어떤 것이 이동하는가, 이동한 값을 다시 쓰지 못하는 규칙, 갈래와 루프에서의 이동, 해제 지점, 그리고 참조의 규칙이다. 참조는 호출 인자로만 만들고 매개변수로만 받는다. 지역 변수·필드·결과에 담기지 않으므로 수명 표기가 없다(ADR-0015). 타입 자체는 03장, 증명 의무는 07장에 있다.

## 복사와 이동

타입이 복사인지 이동인지는 구조로 정해진다(ADR-0047). 선언에 표시하지 않는다.

| 타입 | 값 쓰임 |
| --- | --- |
| 정수 타입, `f64`, `bool`, `char`, `()` | 복사 |
| 복사 타입만 담는 배열·구조체·열거형(`Option<i64>`, `[u8; 4]`) | 복사 |
| `Vec<T>`, `String`, `Box<T>` | 이동 |
| 이동 타입을 필드·페이로드·원소·타입 인자로 담는 구조체·열거형·배열 | 이동 |
| `Copy` 경계가 없는 타입 매개변수 `T` | 이동 |
| `Copy` 경계가 있는 타입 매개변수 | 복사. 이동 타입은 그 타입 인자가 못 된다(E0428) |
| `std::process::Child` | 이동 |

- 값 쓰임은 `let`의 초기화 식, 대입의 오른쪽, 값으로 넘기는 인자, 수신자가 `self: T`인 메서드 호출, 블록·`if`·`match` 갈래의 값, `return`의 값, 구조체 생성의 필드 값이다. 이동 타입의 값은 그 자리에서 옮겨 가고, 복사 타입의 값은 복사된다.
- 읽기는 이동이 아니다: `v.len()`, `v[i]`, `b.f`, `*b`(복사 타입의 값), `&v` 인자, `v.as_slice()`.
- 복제는 `v.clone()`이다(`Vec`, `String`).
- 코퍼스: `tests/run/values.bmb`, `tests/run/vecs.bmb`, `tests/run/boxes.bmb`, `tests/run/generic_moves.bmb`, `tests/check/err/boxes.bmb`.

## 이동한 값의 사용

이동한 지역 변수나 매개변수를 다시 쓰면 E0413 오류다. 다시 대입하면(`let mut`) 소유로 돌아온다.

```
#[effects(pure)]
fn take(v: Vec<i64>) -> usize { v.len() }
#[effects(alloc)]
fn twice() -> usize {
    let v: Vec<i64> = Vec::new();
    let n: usize = take(v);
    n + v.len()
}
```

`n + v.len()`의 `v`가 E0413 오류다. 아래는 받는 프로그램이다.

```
#[effects(alloc)]
fn again() -> usize {
    let mut v: Vec<i64> = Vec::new();
    let n: usize = take(v);
    v = Vec::new();
    n + v.len()
}
```

- 이동 페이로드를 묶는 `match`는 피검사 값을 이동한다. `?`의 피연산자도 이동한다. 그 뒤의 사용은 E0413 오류다.
- 수신자가 `self: T`인 메서드를 두 번 부르면 둘째 호출이 E0413 오류다. `FnOnce` 경계의 함수 값을 두 번 부르는 것도 같다.
- 명세 문맥(`assert`, `assume`, 불변식, `proof` 블록)은 이동한 값을 읽는다. 이동 상태를 보지도 바꾸지도 않는다(ADR-0098, `tests/verify/pass/moved_specs.bmb`).
- 코퍼스: `tests/check/err/moves.bmb`, `tests/check/err/move_traits.bmb`, `tests/check/err/child.bmb`, `tests/run/move_traits.bmb`.

## 자리에서 꺼내기와 부분 이동

통째로 옮길 수 있는 것은 지역 변수와 매개변수, 그리고 지역 구조체의 필드다.

| 꺼내는 자리 | 판정 |
| --- | --- |
| 지역 구조체의 필드 `b.v` | 부분 이동. 그 필드를 다시 쓰면 E0413, 구조체 전체를 쓰면 E0433, 다른 필드는 쓴다 |
| 원소 `g[0]` | E0413. `pop`으로 꺼내거나 읽는다 |
| `*r`, `*b`(이동 타입의 값) | E0413 |
| 공유 참조 뒤나 `Box` 안의 필드 `r.v` | E0413 |
| `&mut` 뒤의 필드 `m.v` | E0239. `std::mem::replace(&mut m.v, Vec::new())`로 바꿔 꺼낸다 |

- 꺼낸 필드에 다시 대입하면 구조체는 온전해진다. 통째로 이동한 구조체의 필드에는 대입하지 못한다(E0413).
- 지역 구조체의 필드에 대한 `match`(`match l.e`)도 그 필드만의 부분 이동이다.
- `std::mem::replace(&mut p, v)`는 `p`의 옛 값을 돌려주고 `p`에 `v`를 둔다. `std::mem::swap(&mut a, &mut b)`는 두 자리의 값을 바꾼다. 두 자리가 같은 변수 안에 있으면 E0410 오류다. `std::mem::take`와 `drop(v)`는 없다(E0219).
- 코퍼스: `tests/run/owned_fields.bmb`, `tests/run/mem_moves.bmb`, `tests/check/err/owned_fields.bmb`, `tests/check/err/mem_moves.bmb`, `tests/check/err/nested_vecs.bmb`.

## 갈래와 루프의 이동

경로가 합쳐지는 곳에서 값의 상태는 하나여야 한다. 해제 지점이 프로그램 텍스트로 정해지기 때문이다(ADR-0031, ADR-0081).

| 합류점 | 한 경로만 이동했을 때 |
| --- | --- |
| `if` .. `else`, `match`의 갈래 | 받는다. 이동하지 않은 갈래가 끝에서 그 값을 해제하고, 합류 뒤에 값은 이동한 상태다 |
| `else` 없는 `if` | E0414 |
| `&&`·`\|\|`의 오른쪽 | E0414 |
| 루프의 몸(`while`, `loop`, `for`) | E0414. 같은 반복 안에서 다시 대입하면 받는다 |
| `break`·`continue`가 모이는 루프의 출구와 머리 | 상태가 다르면 E0414 |

```
#[effects(alloc)]
fn branch(c: bool) -> usize {
    let v: Vec<i64> = Vec::new();
    if c { take(v) } else { 0 }
}
```

- 합류 뒤에 이동한 상태의 값을 쓰면 E0413 오류다(어느 갈래로 왔든).
- `return`이나 `!` 타입의 식으로 끝나는 경로는 합류에서 빠진다.
- 코퍼스: `tests/run/branch_drops.bmb`, `tests/run/loop_exits.bmb`, `tests/check/err/moves.bmb`, `tests/check/err/loop_exits.bmb`.

## 해제

소유한 값은 다음 지점에서 해제된다. 지점은 이동 검사가 정하고 프로그램이 직접 쓰지 않는다(`drop(v)`는 E0219).

- 블록의 끝: 그 블록에서 선언되어 소유로 남은 지역(부분 이동된 구조체는 남은 필드만). 함수의 끝에서는 값으로 받은 매개변수도.
- 대입: 소유 값을 가진 자리(변수, 필드, 원소, `*b`)에 대입하면 옛 값.
- `return`·`?`·`break`·`continue`: 그 출구가 벗어나는 범위의 소유 지역.
- 한 갈래만 이동한 `if`·`match`: 다른 갈래의 끝.
- 값을 버리는 식 문장의 값. 읽기만 한 임시값(`mk().x`, `&mk()` 인자)은 그 영역의 끝.
- 구조체는 필드 순서로, 열거형은 변형의 페이로드 순서로, `Vec`은 원소를 해제한 뒤 버퍼를, `Box`는 값을 해제한 뒤 메모리를 푼다.
- 코퍼스: `tests/run/branch_drops.bmb`, `tests/run/returns.bmb`, `tests/run/owned_arrays.bmb`, `tests/run/vec_tree.bmb`.

## 참조는 매개변수에만

참조 타입은 매개변수의 타입으로만 쓴다.

| 꼴 | 가리키는 것 |
| --- | --- |
| `&[T]`, `&mut [T]` | 배열, `Vec`, 슬라이스의 원소 |
| `&str` | 텍스트 |
| `&T`, `&mut T` | 스칼라(ADR-0054), 구조체·열거형, `Vec<T>`, `String`, `Box<T>`, 타입 매개변수 |

- 지역 변수의 타입, 반환 타입, 필드와 페이로드의 타입에 참조를 쓰면 E0403 오류다. 타입 인자로도 쓰지 못한다(E0403).
- 수명 표기(`'a`)는 E0208 오류다. 참조가 함수 밖으로 나가지 않으므로 필요가 없다.
- `&[i64; 3]`(배열의 참조), `&mut str`은 타입이 아니다(E0402). 배열은 `&[T]`로 넘긴다.
- 몸에서 `&T`·`&mut T` 매개변수는 `*x`로 읽고(스칼라), `x.f`로 필드를 읽으며, `&mut`이면 `*x = v`·`x.f = v`로 쓴다.
- 코퍼스: `tests/run/scalar_refs.bmb`, `tests/run/swap.bmb`, `tests/check/err/types.bmb`, `tests/check/err/data_types.bmb`, `tests/check/err/places.bmb`.

## 참조 인자

`&e`와 `&mut e`는 호출의 인자 자리에 바로 올 때만 쓴다. 다른 자리(`let`의 오른쪽, `*(&x)`, `Some(&p)`)는 E0409 오류다.

| 매개변수 | 인자 |
| --- | --- |
| `&[T]` | `&arr`, `v.as_slice()`, `&v[lo..hi]`(정수 원소), 받은 `&[T]`·`&mut [T]` 매개변수 |
| `&mut [T]` | `&mut arr`, `v.as_mut_slice()`, 받은 `&mut [T]` 매개변수 |
| `&str` | 리터럴, `s.as_str()`, `&s[a..b]`, 받은 `&str` 매개변수 |
| `&T` | `&e`(값이면 된다), `&*b`, 받은 `&T`·`&mut T` 매개변수 |
| `&mut T` | `&mut 자리`(변수, 필드 `p.x`, 원소 `a[0]`, `*b`), 받은 `&mut T` 매개변수 |

- `v.as_slice()`·`v.as_mut_slice()`·`s.as_str()`·문자열 리터럴도 인자 자리 밖에서는 E0409 오류다(리터럴은 비교의 피연산자로도 온다).
- `&mut v[lo..hi]`는 E0409 오류다. 부분 슬라이스는 읽기 전용이다.
- `Vec`이나 `String`을 슬라이스·`&str` 자리에 `&v`·`&s`로 넘기면 E0218 오류다. 참조 자리에 값을, 값 자리에 참조를 넘기거나 `&`와 `&mut`이 어긋나면 E0401 오류다.
- 메서드 호출 `x.f(..)`는 선언된 수신자에 따라 `&x`·`&mut x`·`x`를 넘긴다(ADR-0051). `x`가 이미 참조면 그대로 넘긴다.
- 코퍼스: `tests/run/methods.bmb`, `tests/check/err/types.bmb`, `tests/check/err/subslices.bmb`, `tests/check/err/vec.bmb`, `tests/check/err/strings.bmb`.

## 쓸 수 있는 자리

대입의 왼쪽, `&mut` 인자, 변경 메서드(`push`, `pop`, `truncate`, `push_str`, `copy_within`)의 수신자, 수신자가 `self: &mut T`인 메서드의 수신자는 쓸 수 있는 자리여야 한다. 아니면 E0408 오류다.

- 쓸 수 있는 자리는 `let mut` 지역, `&mut` 매개변수가 가리키는 값, 그리고 그 아래의 필드·원소·`*b` 경로다.
- 쓸 수 없는 것: `let` 지역, 값 매개변수(`mut` 매개변수는 E0210: `let mut` 지역에 복사한다), `&` 매개변수가 가리키는 값, `for`의 변수, 공유 참조 `match`의 바인딩.
- 코퍼스: `tests/run/fields.bmb`, `tests/run/box_places.bmb`, `tests/check/err/types.bmb`, `tests/check/err/places.bmb`, `tests/check/err/ranges.bmb`.

## 별칭

한 호출에서 `&mut`으로 넘긴 변수는 그 호출의 다른 인자에 나오지 못한다(E0410).

```
#[effects(pure)]
fn fill(a: &mut [i64], n: usize) -> () {}
#[effects(pure)]
fn same(a: &[i64], b: &[i64]) -> bool { a.len() == b.len() }
#[effects(pure)]
fn ok() -> bool {
    let mut x: [i64; 2] = [0, 0];
    fill(&mut x, x.len());
    same(&x, &x)
}
```

| 함께 넘기는 것 | 판정 |
| --- | --- |
| `&x`와 `&x` | 받는다 |
| `&mut x`와 `&x`, `&mut x`와 `&mut x` | E0410 |
| `&mut x`와 서로 다른 변수의 참조 | 받는다 |
| `&mut x`와 `x`의 이동(`keep(&mut w, w)`) | E0410 |
| `&mut x`와, 값으로 넘기는 인자 자리에서 `x`를 읽기만 하는 식 | 받는다(ADR-0024) |

- 읽기만 하는 식은 변수, 리터럴, 인덱스, 단항·이항 연산, 메서드 호출로만 된 식이다: `x.len()`, `x[0]`, `k + 1`. 호출, 블록, `if`가 들어가거나 필드를 읽는 식(`p.x`)은 E0410 오류다.
- 별칭은 변수 단위로 본다. 같은 변수의 다른 필드나 다른 원소도 겹친다고 본다.
- 코퍼스: `tests/check/err/types.bmb`, `tests/check/err/moves.bmb`, `tests/check/err/data_values.bmb`, `tests/run/swap.bmb`.

## 참조 `match`

- `&T` 매개변수에 대한 `match`의 바인딩은 페이로드를 가리키는 참조다. `*n`으로 읽고, 쓰지 못한다(E0408).
- `&mut T` 매개변수에 대한 `match`의 바인딩은 `*n = v`로 쓴다. 갈래 안에서 피검사 값을 쓰면(`*s = ..`) E0410 오류다: `match`가 그것을 빌리고 있다(ADR-0055).
- 참조 `match`의 바인딩에서 이동 타입의 값을 꺼내면 E0413 오류다.
- 코퍼스: `tests/run/mut_match.bmb`, `tests/check/err/mut_match.bmb`.

## 클로저의 캡처

- 바깥 지역을 쓰는 클로저는 `move`를 쓴다(E0262, ADR-0053). 캡처는 클로저를 만드는 지점에서 값 쓰임이다: 복사 타입은 복사되고 이동 타입은 옮겨 간다. 옮겨 간 지역을 뒤에서 쓰면 E0413 오류다.
- 캡처하는 지역의 타입은 적혀 있어야 한다(매개변수, 타입 있는 `let`). `for`의 변수처럼 타입을 적지 않은 이름의 캡처는 E0262 오류다.
- 경계가 호출의 방식을 정한다: `Fn`은 캡처를 읽기만 하고, `FnMut`는 캡처를 바꾸며, `FnOnce`는 한 번 부르면 함수 값이 이동한다(둘째 호출은 E0413).

```
#[effects(pure)]
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 { f(x) }
#[effects(alloc)]
fn caps(k: i64) -> i64 {
    let v: Vec<i64> = Vec::new();
    let a: i64 = apply::<_>(move |x: i64| -> i64 { x.wrapping_add(k) }, 1);
    apply::<_>(move |x: i64| -> i64 { if v.len() > 0 { x } else { a } }, k)
}
```

- 코퍼스: `tests/run/fn_values.bmb`, `tests/check/err/fn_values.bmb`.
