# 08 효과와 탈출구

이 장은 함수의 효과 선언 `#[effects(..)]`와 그 검사, 항목이 받는 속성의 범위, 그리고 증명 없이 사실을 받아들이는 두 탈출구 `assume`과 `#[external_body]`를 그 근거 `#[rationale(..)]`와 함께 다룬다. 여기의 진단은 E06xx(효과와 속성)와 E10xx(탈출구)다. 증명 의무와 판정은 [07 증명 게이트](07-proof-gate.md)에 있다.

## 효과의 집합

효과는 원자 효과의 집합이고, `pure`는 빈 집합의 이름이다(ADR-0030 §1). 원자 효과는 넷이다.

| 효과 | 뜻 | 원천 |
| --- | --- | --- |
| `alloc` | 힙 할당. 메모리가 모자라면 중단한다 | `Vec::new`·`Vec::with_capacity`·`push`·`Vec`의 `clone`·`Box::new` 등 |
| `io` | 바깥과의 상호작용 | `print_i64` 같은 출력 내장과 시스템 내장(ADR-0091) |
| `nondet` | 같은 프로그램과 같은 인자에서도 실행마다 다를 수 있는 값을 읽는다 | 명령줄 인자, 표준 입력, 파일 읽기, 환경 변수, 시계 |
| `panic` | 실행 중 trap으로 끝날 수 있다 | 탈출구가 있는 정의: `assume` 문장, `#[external_body]`(ADR-0029) |

- 효과는 서로 직교한다. `io`는 다른 효과를 함의하지 않는다. 입력 내장은 `io`와 `nondet`을 함께 갖는다(예: `std::io::read_line`은 `alloc, io, nondet`).
- 메모리 부족에 따른 중단은 `panic`이 아니라 `alloc`에 든다. 해제는 효과가 아니다.
- 효과 규칙을 보이는 코퍼스는 `tests/check/err/effects.bmb`와 그 기대 진단 `tests/check/err/effects.jsonl`이다.

## `#[effects(..)]` 선언

- 실행 함수는 `#[effects(..)]`가 필수다. 없으면 오류다(E0601, `tests/check/err/items.bmb`). trait 메서드의 선언과 impl의 함수도 같다.
- 인자는 `pure` 하나이거나 원자 효과의 이름들이다. 정규형은 이름순이다: `#[effects(alloc, io)]`.
- 모르는 이름, 식별자가 아닌 인자, 빈 목록 `#[effects()]`는 오류다(E0602, `tests/check/err/effects.bmb`의 `unknown`·`none`).
- `pure`를 다른 효과와 함께 쓰거나 한 효과를 두 번 쓰면 오류다(E0607, `two_effects`·`twice`·`pure_too`). 진단의 수정은 정규형이다.
- `#[effects(..)]`를 한 함수에 두 번 쓰면 오류다(E0605, `tests/check/err/attrs.bmb`).
- `main`은 `#[effects(io)] fn main() -> ()`이고 `io`와 함께 다른 효과를 쓸 수 있다. `io`가 없으면 오류다(E0701, 02장).

## 선언과 몸의 관계

- 함수의 **필요 효과**는 몸 자신의 연산의 효과(할당, 내장 호출, `assume`)와 몸이 부르는 실행 함수의 **선언** 효과의 합이다. 선언은 필요 효과를 담아야 한다. 빠진 효과가 있으면 그 함수의 속성에 E0606 진단이 하나 나오고, 메시지는 빠진 효과마다 첫 원천을 적는다: `` `both` needs `alloc` (`Box::new`) and `io` (a call of `print_i64`) but declares `#[effects(pure)]` ``.
- `pure` 함수가 `io` 함수를 부르면 그 호출 자리에 E0603 진단이 나온다(`tests/check/err/effects.bmb`의 `prints`).
- 선언이 필요보다 넓은 것은 오류가 아니다. 선언은 인터페이스다(`wider`). 호출자는 피호출자의 몸이 아니라 선언을 본다. 그래서 피호출자의 몸이 바뀌어 효과가 늘면 오류는 호출자가 아니라 피호출자에 난다(ADR-0030 §3).
- 상호 재귀에서도 규칙은 직접 호출마다 같다(`ping`·`pong`).
- trait impl의 함수는 trait 메서드가 선언하지 않은 효과를 갖지 못한다(E0316, `tests/check/err/effects.bmb`의 `traits`). 경계를 거친 호출의 효과는 trait 메서드의 선언이다.
- `#[external_body]` 함수의 몸도 같은 규칙으로 검사한다. 효과는 신뢰의 대상이 아니다.
- `nondet`을 선언한 함수의 `loop`는 `decreases` 없이 받는다(ADR-0094, 06장).

## 명세와 유령 코드의 효과

- `spec fn`은 효과가 없고 효과 속성을 쓰지 않는다. 쓰면 오류다(E0604, `tests/check/err/effects.bmb`의 `spec_with_effect`).
- `proof fn`도 효과 속성을 쓰지 않는다(E0256, `tests/check/err/proofs.bmb`). 유령 코드는 실행 파일에 남지 않는다.
- 명세 문맥(계약 절, 불변식, `assert`의 조건)과 `proof { }` 블록은 둘러싼 함수의 필요 효과에 더하는 것이 없다. 실행 함수를 부를 수 없기 때문이다(06장의 E0504, E0257).

## 함수 값의 효과

- 경계 `F: Fn(A) -> R`를 가진 제네릭 함수 `g` 안에서 `f(a)` 호출의 효과는 `g`가 선언한 효과다(ADR-0053 §5).
- 그래서 `g`에 넘기는 함수나 클로저의 효과는 `g`의 선언 안에 있어야 한다. 넘기는 것이 함수이면 그 인자에 E0608 진단이 나온다(클로저이면 클로저 몸의 그 호출에 E0603이 나온다: 클로저의 효과는 받는 쪽 `F`의 선언이다): `` `shout` has effect `io`, which `fve::apply` does not declare for its `F` ``(`tests/check/err/fn_values.bmb`). 수정은 `g`에 그 효과를 선언하거나 그 효과가 없는 함수를 넘기는 것이다.
- 통과하는 꼴은 `tests/run/fn_values.bmb`에 있다.

## 항목이 받는 속성

속성의 목록은 닫혀 있다. 그 자리에 오지 않는 속성과 모르는 속성은 오류이고(E0605), 진단의 수정은 그 속성을 지우는 것이다(`tests/check/err/attrs.bmb`, `tests/check/err/habits.bmb`, `tests/check/err/summaries.bmb`).

| 자리 | 받는 속성 |
| --- | --- |
| 단위(`mod`) | `#[summary("..")]`(필수) |
| 함수 | `#[summary("..")]`, `#[effects(..)]`, `#[uses(..)]`, `#[external_body]`, `#[rationale(..)]` |
| 구조체·열거형 | `#[summary("..")]`, `#[invariant(p)]` |
| trait, `const` | `#[summary("..")]` |
| 문장 | `assume(..)` 앞의 `#[rationale(..)]`뿐(그 밖은 E1002) |
| 명세 식의 항 | `#[trigger]`(06장) |

- `#[summary(..)]`를 한 정의에 두 번 쓰면 오류다(E0605). 인자가 문자열 하나가 아닌 것은 오류다(E0704, 02장).
- Rust의 `#[derive(..)]`는 Rust 습관의 진단으로 따로 받고(E0220), `#[inline]`·`#[repr(..)]` 등은 모르는 속성이다(E0605).
- `#[uses(..)]`의 인자 규칙은 06장(E0509), `#[invariant(p)]`의 규칙은 06장(E0254, E0255)에 있다.

## 탈출구: `assume`

- `assume(p(..));`는 실행 코드의 문장이다. 그 자리부터 `p(..)`를 증명 없이 사실로 받는다(ADR-0029).
- 인자는 이름 붙은 술어(`spec fn .. -> bool`) 호출 하나다. 그 밖의 식(`assume(x < 10)`)은 오류다(E1003). 명세나 유령 코드(`proof fn`, `proof { }`) 안의 `assume`도 오류다(E1003). 유령 사실은 `assert`나 `proof fn`으로 증명한다.
- 바로 앞에 `#[rationale(..)]`가 있어야 한다. 없으면 오류다(E1001).
- `assume`이 있는 함수는 `panic`을 선언해야 한다(E0606, `tests/check/err/escapes.bmb`의 `missing_panic`). 그 함수를 부르는 함수도 호출 규칙에 따라 `panic`을 선언한다.

## 탈출구: `#[external_body]`

- `#[external_body]`는 실행 함수의 몸을 증명하지 않고 그 계약을 받아들인다. 호출자는 여느 함수처럼 `requires`를 증명하고 `ensures`를 가정한다.
- `spec fn`, `proof fn`, `main`에는 쓰지 않는다(E1004, `tests/check/err/escapes.bmb`의 `external_spec`).
- 함수의 속성에 `#[rationale(..)]`가 함께 있어야 한다. 없으면 오류다(E1001, `external_without`).
- 거꾸로 `#[external_body]`가 없는 함수의 `#[rationale(..)]`는 오류다(E1002, `tests/check/err/attr_places.bmb`의 `lone`).
- 그 함수는 `panic`의 원천이다. 몸은 타입·소유권·효과 검사를 그대로 받는다.
- Rust·Verus의 꼴은 받지 않는다: `unsafe`(E0215), `#[verifier::external_body]`(E0216), `admit()`(E0217). 02장과 04장에 있다.

## 근거: `#[rationale(..)]`

- 꼴은 `#[rationale(kind = K, "설명")]`이다. `K`는 `nonlinear`, `solver_limit`, `boundary`, `performance` 중 하나이고, 설명은 비어 있지 않은 문자열 하나다.
- 형식이 틀리면 오류다(E1002): 목록 밖의 `kind`, `kind`의 누락이나 반복, 설명의 누락·빈 설명·둘 이상의 설명, 모르는 키(`tests/check/err/escapes.bmb`의 `bad_kind`·`no_text`).
- 문장 앞에 오는 속성은 `assume(..)` 앞의 `#[rationale(..)]`뿐이다. 다른 문장 앞의 `#[rationale(..)]`와 문장 앞의 다른 속성은 오류다(E1002, `not_before_assume`, `tests/check/err/summaries.bmb`).
- `hash = ".."`는 근거가 묶인 코드의 해시이고 도구가 쓴다(ADR-0006, ADR-0029 §3): `#[rationale(kind = boundary, hash = "c8c486f5ddfa4a41", "the outside world")]`(`tests/run/escapes.bmb`). 해시는 근거를 뺀 정의의 정규형과 그 정의가 부르는 `spec fn`들의 텍스트에서 계산한다. 그래서 설명 문구만 고치면 해시가 그대로이고, 코드를 고치면 달라진다.
- `hash`가 없거나 현재 값과 다른 근거는 미확인이다. 미확인은 언어의 오류가 아니고 `build`를 막지 않는다. 그것을 알리는 경고와 해시를 쓰는 도구는 `compiler/README.md`와 `docs/diagnostics.md`에 있다.

## 탈출구와 증명의 신뢰 등급

- 탈출구가 있는 프로그램은 먼저 모든 정의를 신뢰 사실 없이 증명한다: `assume`의 사실도, 증명되지 않은 피호출자의 `ensures`도 쓰지 않는다. 거기서 실패하고 탈출구에 닿을 수 있는 정의만 신뢰 사실을 넣어 다시 증명한다(ADR-0029 §2).
- `bmb verify`의 상태 줄은 그 결과를 `trust`로 적는다. 신뢰 사실 없이 증명된 정의는 키가 없고, 신뢰 사실로 증명된 정의는 `"trust":"assumed"`, `#[external_body]` 함수는 의무가 0이고 `"trust":"external"`이다. `status`는 셋 모두 `verified`다. `assumed`나 `external`인 함수의 `ensures`에 기대어 증명된 호출자도 `assumed`다.
- 신뢰 사실은 런타임 검사를 지우는 근거가 아니다(ADR-0029 §4). 신뢰 사실 없이 증명되지 않은 정의와 그것이 부르는 모든 정의는 검사를 남긴 채 컴파일된다. 그래서 거짓 `assume`은 정의되지 않은 동작이 아니라 trap으로 끝날 수 있고, 이것이 `panic` 효과의 뜻이다.
- 탈출구는 `build`를 막지 않는다. 탈출구가 없는 프로그램의 증명은 한 번이고 `trust` 키가 없다.
- 실행 출력까지 고정된 예는 `tests/run/escapes.bmb`와 `tests/run/collatz.bmb`(종료를 증명할 수 없는 루프의 상한을 `assume`으로 받는다)이다.

효과를 좁히라는 경고와 필요 없는 `assume`을 알리는 경고는 린트다. 언어의 판정이 아니며 `compiler/README.md`에 있다.
