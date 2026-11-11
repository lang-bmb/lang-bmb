# 06 계약과 명세

이 장은 명세를 적는 구문과 그 정적 규칙을 다룬다: 함수의 `requires`·`ensures`와 결과 이름, 루프의 `invariant`·`decreases`, `assert`, `old(..)`, 명세 식의 범위와 정수, 수량자와 트리거, `spec fn`·`proof fn`·`proof { }`, 타입 불변식, trait 계약, 함수 값의 계약. 여기의 진단은 검사기(`bmb check`)가 내는 E05xx다. 명세가 만드는 증명 의무와 그 실패(E08xx)는 [07 증명 게이트](07-proof-gate.md)에 있다.

## 계약 절

- 함수의 계약은 시그니처 뒤, 몸 앞의 절이다: `requires c1, c2,` `ensures c3,` `decreases d,`. 순서는 `requires`, `ensures`, `decreases`로 고정이고 어기면 구문 오류다(E0103, `tests/check/err/specs.bmb`의 `order`).
- 절 하나는 `bool` 타입의 명세 식이다. 쉼표로 나눈 절들은 모두 성립해야 한다.
- 수량자(`forall`·`exists`)를 담는 절은 이름 붙은 술어(`spec fn .. -> bool`)의 호출 하나여야 한다. 아니면 오류다(E0501, ADR-0095 A2). 수량자가 없는 절은 식을 그대로 쓴다(`tests/verify/pass/inline_contracts.bmb`).
- 절이 본문이 `true`인 술어의 호출이면 오류다(E0502, `tests/check/err/specs.bmb`의 `vacuous`).
- `ensures`에서 결과를 가리키려면 반환 타입에 이름을 붙인다: `-> (r: T)`. 이름을 붙이지 않고 `result` 같은 이름을 쓰면 모르는 이름 오류다(E0301).
- `requires`는 매개변수를, `ensures`는 매개변수와 결과 이름을 본다. `&mut` 매개변수는 `ensures`에서 함수가 끝난 뒤의 값이고, 진입 시점의 값은 `old(..)`로 쓴다(아래 "`old`").

## 명세 문맥과 실행 문맥

명세 문맥은 계약 절, 루프의 `invariant`·`decreases`, `assert`·`assume`의 조건, `spec fn`의 몸이다. 그 밖의 함수 몸은 실행 문맥이다. `proof fn`의 몸과 `proof { }` 블록은 유령 문맥이고 그 안의 식은 명세 식이다(아래 "`proof fn`과 `proof` 블록").

| 규칙 | 진단 | 코퍼스 |
| --- | --- | --- |
| `forall`·`exists`·`==>`·`old(..)`·`#[trigger]`는 실행 문맥에 쓰지 않는다 | E0503 | `tests/check/err/specs.bmb` |
| 명세 문맥에는 `let mut`·대입·루프·`break`·`return`·`?`·`assert`를 쓰지 않는다 | E0503 | `tests/check/err/spec_mode.bmb` |
| 명세 문맥은 실행 함수를 부르지 않는다 | E0504 | `tests/check/err/specs.bmb`의 `calls_exec` |
| 실행 문맥은 `spec fn`을 부르지 않는다 | E0505 | `tests/check/err/specs.bmb`의 `calls_spec` |

명세 식에 오는 것은 리터럴, 이름, 산술·비교·논리 연산, `==>`, 인덱스·필드·`.len()`, `if`·`match`, 값을 묶는 `let`, `spec fn` 호출, 수량자, `old(..)`다. 명세는 옮겨진 값도 읽는다(ADR-0098, `tests/verify/pass/moved_specs.bmb`).

## 명세의 정수

- 별도의 `int` 타입은 없다. 명세 식은 기계 정수 타입의 이름을 쓰되 산술은 수학적 정수로 읽는다(ADR-0017). 그래서 명세 식에는 넘침·0 나눗셈·인덱스 범위의 증명 의무가 없다. `requires a + b <= 255`(`a`, `b`가 `u8`)는 합이 255 이하라는 뜻 그대로다.
- 명세와 유령 코드에서 정수 타입은 값의 범위를 담는 타입으로 넓어진다(ADR-0099, `tests/verify/pass/spec_widening.bmb`). 서로 다른 정수 타입을 변환 없이 섞어 쓴다.
- 명세에서 범위를 좁히는 변환 `x as T`는 오류이고(E0513), 좁은 타입이 기대되는 자리(`spec fn`의 결과, 타입을 적은 `let`)에 넓은 값을 쓰면 오류다(E0401, `tests/check/err/spec_widening.bmb`, `tests/check/err/conversions.bmb`). `spec fn`의 인자는 예외다: 다른 정수 타입의 값을 넘길 수 있고, 매개변수의 범위 밖 값에서는 그 호출의 값이 정해지지 않는다(ADR-0022). 그래서 `small(b: u8)`에 `u64` 값 256을 넘긴 `small(n)`은 아무것도 말하지 않는다. 넓히는 `as`는 값 그대로와 같은 입력 별칭으로 받는다(`compiler/README.md`의 "입력 별칭").
- `Vec`에는 `==`·`!=`가 없다(E0514, `tests/check/err/vec.bmb`). 길이와 원소를 `v.as_slice()` 위의 술어로 비교한다(ADR-0031).

## 수량자와 트리거

- 꼴은 `forall|i: T, j: U| 식`과 `exists|i: T| 식`이다. 두 수량자의 규칙은 같다(ADR-0079, `tests/verify/pass/exists.bmb`).
- 묶는 변수의 타입은 정수다. 다른 타입이면 오류다(E0402).
- 수량자의 몸에는 `#[trigger]`로 표시한 항이 하나 이상 있어야 한다. 없으면 오류다(E0511). 한 수량자의 트리거들은 하나의 다중 패턴을 이루고, 묶는 변수는 모두 어느 트리거에 나와야 한다. 빠진 변수가 있으면 오류다(E0512, `tests/check/err/specs.bmb`의 `triggers`, `tests/check/err/exists.bmb`).
- `#[trigger]`가 수량자 밖의 명세 식에 있으면 오류다(E0510, `tests/check/err/specs.bmb`의 `bare_trigger`).
- 수량자는 `spec fn`의 몸, 불변식, `assert`에 온다. 계약 절에는 그것을 담은 술어의 호출로만 온다(E0501).

## `spec fn`

- `spec fn 이름(매개변수) -> T { 명세 식 }`은 명세에서만 부르는 함수다(E0505). 효과 속성을 갖지 않는다([08](08-effects-and-escape-hatches.md)의 E0604).
- 재귀하면 `decreases`가 필수다(E0507). 그 측도가 줄어든다는 증명 의무는 07장에 있다(E0813, `tests/verify/pass/proofs.bmb`의 `contains_prefix`, `tests/verify/fail/contracts.bmb`).
- **정의는 기본으로 불투명하다.** 증명은 `spec fn`을 이름으로만 보고, 정의는 그 증명의 문맥에 들어 있을 때만 펼친다(ADR-0017 §4). 문맥은 기본 문맥과 `#[uses(..)]`의 합이다. 기본 문맥은 정의 자신의 절·불변식·`assert`·`proof` 코드가 부르는 `spec fn`과 피호출자의 계약이 부르는 `spec fn`이고, 그 몸이 다시 부르는 `spec fn`으로는 들어가지 않는다(ADR-0095 A1, `tests/verify/pass/default_context.bmb`). 자세한 규칙은 07장 "증명 문맥"에 있다.
- `#[uses(a, unit::b, <T as Tr>::m)]`는 함수 앞에 쓰고 `spec fn`의 이름만 받는다. 실행 함수의 이름, 없는 이름, 이름이 아닌 인자는 오류다(E0509, `tests/check/err/specs.bmb`의 `bad_uses`, `tests/check/err/spec_mode.bmb`, `tests/verify/pass/uses_impl_path.bmb`).

## 루프의 `invariant`와 `decreases`

- `while 조건 invariant p1, p2, decreases d, { 몸 }`. 절은 조건과 몸 사이에 온다. 불변식 절은 `bool` 명세 식이고 수량자를 직접 담을 수 있다.
- `while`과 `loop`에는 `decreases`가 필수다. 없으면 오류다(E0506, `tests/check/err/specs.bmb`의 `no_decreases`, `tests/check/err/loop_exits.bmb`). 범위 `for i in lo..hi`는 `decreases`를 쓰지 않는다(`tests/verify/pass/ranges.bmb`).
- 예외는 하나다: `nondet` 효과를 선언한 함수의 `loop`는 `decreases` 없이 받는다. 그 루프의 종료는 증명되지 않는다(ADR-0094).
- 루프 뒤의 코드가 아는 것은 불변식과 루프를 나온 조건뿐이다. 진입·유지·감소의 의무는 07장에 있다(E0806·E0807·E0808, `tests/verify/pass/loops.bmb`, `tests/verify/fail/loops.bmb`).

## 재귀와 `decreases`

- 재귀하는 함수(상호 재귀 포함)는 `decreases`가 필수다. 실행 함수, `spec fn`, `proof fn` 모두 같다. 없으면 오류다(E0507, `tests/check/err/specs.bmb`의 `rec`·`ping`·`pong`, `tests/check/err/proofs.bmb`).
- 측도는 정수 식이거나 재귀 열거형의 값이다(ADR-0047 §3). 정수 측도는 호출마다 작아지고 0 아래로 내려가지 않아야 하고, 열거형 측도는 호출자의 값의 일부여야 한다(`tests/verify/pass/recursion.bmb`, `tests/verify/pass/recursive.bmb`).
- trait 메서드의 선언에는 `decreases`를 쓰지 않는다(E0207).

## `assert`

- `assert(e);`는 실행 문맥과 유령 문맥의 문장이다. `e`는 명세 식이고 수량자를 담을 수 있다. 그 자리에서 `e`를 증명하고(07장 E0810) 그 뒤로 `e`를 사실로 쓴다. 실행 코드를 만들지 않는다. 명세 문맥 안에는 쓰지 않는다(E0503).
- `assert(e) by (bit_vector);`는 `e`를 따로 된 비트벡터 질의로 증명한다(ADR-0048 §4). `e`에는 정수·`f64`·`bool` 항만 온다. 수량자, `old`, 자료 값, 그리고 단언 안의 `let`을 부르는 원소·필드·길이는 오류다(E0515). 그 밖의 원소·필드·길이·참조 뒤의 값(`s[0]`, `r.x`, `*r`, `s.len()`)은 하위 증명의 변수로 받는다. `bit_vector` 말고 다른 증명기 이름과 `assert(..) by { .. }`는 오류다(E0516, `tests/check/err/bit_vector.bmb`, `tests/verify/pass/bit_vector.bmb`). 이 하위 증명은 `e`의 변수만 보고 경로의 사실을 보지 않는다. 그 실패는 07장에 있다(E0818).

## `old`

- `old(x)`는 `&mut` 매개변수 `x`의 함수 진입 시점 값이다. `ensures`, 루프 불변식, `assert`, `proof` 블록에 온다.
- 그 밖의 명세 자리(`requires` 등)에 있거나 `x`가 `&mut` 매개변수가 아니면 오류다(E0508, `tests/check/err/specs.bmb`의 `old_on_shared`). 실행 문맥에 있으면 오류다(E0503).
- 스칼라 참조는 `*old(x)`, 슬라이스는 `old(a)[i]`·`old(a).len()`, 수신자는 `old(self)`로 쓴다(`tests/verify/pass/inline_contracts.bmb`의 `bump`, `tests/verify/pass/trait_contracts.bmb`).

## `proof fn`과 `proof` 블록

- `proof fn 이름(매개변수) requires .., ensures .., decreases .., { .. }`는 보조 정리다(ADR-0062). 반환 타입과 효과 속성을 쓰지 않는다(E0256). 매개변수는 값과 `&T`다. 계약 절의 규칙은 실행 함수와 같다(E0501, E0507). `#[uses(..)]`를 받는다.
- 유령 문맥은 `proof fn`의 몸과 실행 코드의 `proof { .. }` 블록(문장 자리)이다. 오는 것은 `proof fn` 호출, `assert`, 경우를 나누는 `if`·`match`, 명세 식의 `let`이다. 실행 함수 호출, 대입, `let mut`, `&mut`, 루프, `proof` 블록의 중첩은 오류다(E0257).
- `proof fn`은 유령 문맥에서만 부른다. 실행 코드나 명세에서 부르면 오류다(E0258, `tests/check/err/proofs.bmb`).
- 유령 코드의 산술은 수학적 정수라 넘침·인덱스 의무가 없다. 대신 `proof fn` 호출의 인자가 매개변수 타입 안에 있음이 의무다(07장 E0819).
- 호출은 `requires`를 증명하고 `ensures`를 그 뒤의 사실로 준다. 귀납은 `decreases`가 있는 재귀 호출이다(`tests/verify/pass/proofs.bmb`, `tests/run/max_list.bmb`). 유령 코드는 실행 파일에 남지 않는다.

## 타입 불변식

- 구조체·열거형 앞의 `#[invariant(p)]`. `p`는 그 타입을 `x: &T`로 받아 `bool`을 돌려주는 `spec fn`이고 본문이 `true`가 아니다(E0254). 타입마다 하나다(E0255, `tests/check/err/invariants.bmb`). ADR-0060.
- 실행 함수는 매개변수, 호출의 결과, 호출 뒤의 `&mut` 인자, 루프 머리의 지역에 대해 불변식을 가정한다. 값이 함수의 손을 떠나는 자리(호출 인자, 반환, 출구의 `&mut` 매개변수, 루프 머리)에서 불변식이 의무다(07장 E0823). 함수 안에서는 잠시 깨도 된다.
- 술어 `p`도 다른 `spec fn`처럼 불투명하다. 정의가 필요한 증명은 `#[uses(p)]`를 쓴다(`tests/verify/pass/invariants.bmb`, `tests/run/ring.bmb`).
- `spec fn`, 명세 안의 값, `proof fn`, `proof` 블록에는 가정도 의무도 없다. 보조 정리가 불변식을 쓰려면 `requires p(x)`로 적는다.

## trait의 계약과 impl의 정제

- trait 메서드의 선언은 `requires`·`ensures`를 갖는다. trait는 몸 없는 `spec fn m(self: &Self) -> T;`를 선언할 수 있고 impl마다 그 정의를 쓴다(ADR-0046 §4, `tests/verify/pass/trait_contracts.bmb`).
- 경계를 거친 호출(`Self`가 타입 매개변수)은 trait의 계약을 쓴다. 구체 타입의 호출은 그 impl 함수의 계약을 쓴다.
- impl의 함수는 자기 계약을 쓰고, 그 계약은 trait의 계약을 정제해야 한다: trait의 전제가 impl의 각 전제를 함의하고, trait의 전제와 impl의 결과가 trait의 각 결과를 함의한다. 이 의무의 실패는 07장에 있다(E0817, `tests/verify/fail/trait_contracts.bmb`).

## 함수 값의 계약

- 경계 `F: Fn(A) -> R`인 매개변수 `f`에 대해 명세는 `f.requires(a)`(전제)와 `f.ensures(a, r)`(결과)를 쓴다. `f(a)` 호출은 `f.requires(a)`를 증명하고 `f.ensures(a, r)`를 가정한다(ADR-0053, `tests/verify/pass/closure_context.bmb`, `tests/verify/fail/fn_values.bmb`).
- 함수 항목을 값으로 넘기면 그 함수의 `requires`·`ensures`가 값의 계약이다. 클로저는 계약을 제자리에 쓴다: `|x: T| -> (r: R) requires P, ensures Q, { 몸 }`(`tests/run/fn_values.bmb`).
- 클로저가 적은 절이 부르는 `spec fn`은 클로저의 기본 문맥에 들고, 클로저는 그것을 쓴 함수의 `#[uses(..)]`를 물려받는다(ADR-0114).
- 넘기는 함수의 효과에 대한 규칙(E0608)은 08장에 있다.

`W`로 시작하는 계약 린트(공허한 전제, 약한 명세 등)는 언어의 판정이 아니다. 목록은 `docs/diagnostics.md`, 도구는 `compiler/README.md`에 있다.
