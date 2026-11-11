# 07 증명 게이트

이 장은 검사를 통과한 프로그램에 증명 게이트(`bmb verify`)가 하는 판정을 다룬다: 정의마다 생기는 증명 의무의 종류, 호출자가 피호출자에 대해 가정하는 것, 증명 문맥(`#[uses(..)]`와 기본 문맥), 판정(증명됨, 반례가 있는 실패, 미결정), 결정론, 그리고 `build`와 `emit-llvm`이 판정을 쓰는 방식. 여기의 진단은 E08xx다. 명세를 적는 구문은 [06 계약과 명세](06-contracts-and-specs.md), 탈출구가 판정에 주는 신뢰 등급은 [08 효과와 탈출구](08-effects-and-escape-hatches.md)에 있다.

## 의무의 단위

- 증명의 단위는 정의다. 실행 함수와 `proof fn`은 몸을 기호 실행해 의무를 만들고, 재귀하는 `spec fn`은 종료 의무를 갖는다. 제네릭 정의는 타입 매개변수를 정해지지 않은 타입으로 두고 한 번 증명한다(ADR-0046, `tests/verify/pass/generics.bmb`).
- 의무마다 깨끗한 해법기 상태에서 따로 푼다. 가정은 그 자리까지의 경로에서 얻은 사실이다: 함수의 `requires`, 매개변수 타입의 범위, 지나온 조건, 앞선 `assert`, 호출의 `ensures`, 루프의 불변식.
- `bmb verify`는 진단 뒤에 정의마다 상태 줄을 낸다: `{"def":..,"status":"verified"|"failed","obligations":n,"failed":k,"rlimit":..,"rlimit_max":..}`. 의무가 없는 정의는 `obligations`가 0인 `verified`다(`tests/verify/fail/loops.jsonl`의 끝줄들).

## 의무의 종류

| 의무의 종류 | 언제 생기는가 | 실패 진단 |
| --- | --- | --- |
| 호출의 전제 | 실행 함수·`proof fn`·함수 값(`f(a)`)·경계를 거친 trait 메서드를 부르는 자리마다 피호출자의 `requires` 절 각각 | E0801 |
| 사후 조건 | 함수의 모든 출구(꼬리 식, `return`, `?`)에서 `ensures` 절 각각. `proof fn`의 `ensures`도 같다 | E0802 |
| 넘침 | 실행 코드의 `+`·`-`·`*`·단항 `-`, `i64::MIN / -1` 꼴의 나눗셈, 폭을 벗어난 시프트 양 | E0803 |
| 0 나눗셈 | 실행 코드의 `/`·`%`의 제수 | E0804 |
| 인덱스·범위 | `a[i]`, 부분 슬라이스 `&a[lo..hi]`, `copy_within`의 범위 | E0805 |
| 불변식의 진입 | 루프에 들어가는 자리에서 `invariant` 절 각각 | E0806 |
| 불변식의 유지 | 루프 몸의 끝(`continue` 포함)에서 `invariant` 절 각각 | E0807 |
| 루프의 감소 | 루프 몸의 끝에서 `decreases` 값이 줄고 음수가 아님 | E0808 |
| 재귀 호출의 감소 | 같은 재귀 성분의 실행 함수·`proof fn`을 부르는 자리에서 측도가 줄어듦 | E0809 |
| 단언 | `assert(e);` | E0810 |
| `spec fn`의 종료 | 재귀하는 `spec fn`의 몸 안의 재귀 호출에서 측도가 줄어듦 | E0813 |
| 변환의 범위 | 실행 코드의 `x as T`에서 값이 `T` 안에 있음. `f64`로는 64비트 정수가 2^53 이하여서 정확함 | E0815 |
| `Vec`·`String`의 최대 길이 | `push`·`with_capacity`·`push_str` 등 길이를 늘리는 연산 | E0816 |
| trait 계약의 정제 | trait impl의 함수마다: trait의 전제가 impl의 전제를, trait 전제와 impl 결과가 trait의 결과를 함의 | E0817 |
| 비트벡터 하위 증명 | `assert(e) by (bit_vector);`의 따로 된 질의 | E0818 |
| 유령 인자의 범위 | `proof fn` 호출의 인자가 매개변수 타입 안에 있음 | E0819 |
| 문자 경계 | 문자열 범위 `&s[a..b]`의 두 위치가 문자 경계이고 `a <= b` | E0820 |
| 타입 불변식 | `#[invariant(p)]`가 붙은 타입의 값이 떠나는 자리: 호출 인자, 반환, 출구의 `&mut` 매개변수, 루프 머리 | E0823 |

- 코퍼스(코드: 파일): E0801·E0802·E0809·E0810·E0813: `tests/verify/fail/contracts.bmb`. E0803·E0804: `tests/verify/fail/arith.bmb`, `tests/verify/fail/widths.bmb`. E0805: `tests/verify/fail/bounds.bmb`, `tests/verify/fail/subslices.bmb`. E0806·E0807·E0808: `tests/verify/fail/loops.bmb`. E0815: `tests/verify/fail/conversions.bmb`. E0816: `tests/verify/fail/vecs.bmb`. E0817: `tests/verify/fail/trait_contracts.bmb`. E0818: `tests/verify/fail/bit_vector.bmb`. E0819: `tests/verify/fail/proofs.bmb`. E0820: `tests/verify/fail/strings.bmb`. E0823: `tests/verify/fail/invariants.bmb`. 각 파일 옆의 `.jsonl`이 기대 진단이다.
- 명세 식과 유령 코드에는 넘침·0 나눗셈·인덱스 의무가 없다. 산술이 수학적 정수이기 때문이다(ADR-0017). 감싸는 산술(`wrapping_add` 등)과 검사 산술에도 넘침 의무가 없다(`tests/verify/pass/arith_methods.bmb`).
- `match`의 가지가 모든 변형을 덮는가는 증명 의무가 아니다. 검사기가 판정한다(E0426, 04장).
- 경로 조건에 이미 있는 사실은 의무로 내지 않는다. 타입 불변식이 있는 값을 옮기기만 하는 코드에는 E0823 의무가 없다(`tests/verify/pass/invariants.bmb`).

## 호출자가 가정하는 것

호출은 피호출자의 `requires`를 증명하고(E0801) `ensures`만 가정한다(ADR-0017). 피호출자의 몸은 보지 않는다. 계약이 없는 함수의 결과에 대해 호출자가 아는 것은 타입의 범위와 타입 불변식뿐이다. `&mut` 인자는 호출 뒤에 `ensures`가 말한 것만 남는다.

```
#[summary("호출자는 계약만 본다")]
mod m {
    #[effects(pure)]
    fn one() -> i64 { 1 }

    #[effects(pure)]
    fn two() -> (r: i64)
        ensures r == 2,
    {
        one() + one()
    }
}
```

`bmb verify`는 `two`에 E0802(`postcondition `r == 2` may not hold in `m::two``)와 E0803(`` `+` may overflow `i64` ``)을 낸다. `one`에 `-> (r: i64) ensures r == 1,`을 쓰면 둘 다 증명된다.

같은 규칙이 루프에도 선다: 루프 뒤의 코드는 불변식과 루프를 나온 조건만 안다. 경계를 거친 호출은 trait의 계약을, 구체 타입의 호출은 그 impl 함수의 계약을, 함수 값의 호출은 `f.requires`·`f.ensures`를 쓴다(`tests/verify/pass/trait_contracts.bmb`, `tests/verify/fail/fn_values.bmb`).

## 증명 문맥

`spec fn`의 정의는 증명 문맥에 있을 때만 펼쳐지고, 없으면 해법기는 그 함수를 이름으로만 본다(ADR-0017 §4). 한 정의의 문맥은 다음 둘의 합이다.

- **기본 문맥(ADR-0095 A1).** 정의 자신의 계약 절·루프 불변식·`assert`·`proof` 코드가 부르는 `spec fn`, 그리고 피호출자의 `requires`·`ensures`가 부르는 `spec fn`. 그 `spec fn`의 몸이 다시 부르는 `spec fn`으로는 들어가지 않는다(`tests/verify/pass/default_context.bmb`).
- **`#[uses(..)]`에 쓴 이름.** 더하기만 하고 전이되지 않는다: `#[uses(a)]`는 `a`의 몸을 펼칠 뿐 그 몸이 부르는 `spec fn`을 펼치지 않는다. 다른 단위의 것은 `unit::name`, trait impl의 것은 `<T as Tr>::name`으로 쓴다(`tests/verify/pass/uses_impl_path.bmb`). `spec fn`이 아닌 이름은 검사 오류다(06장, E0509).
- **클로저(ADR-0114).** 클로저가 적은 절이 부르는 `spec fn`은 그 클로저의 기본 문맥에 들고, 클로저는 그것을 쓴 함수의 `#[uses(..)]`를 물려받는다(`tests/verify/pass/closure_context.bmb`). 함수 항목을 값으로 넘긴 경우에는 이 규칙이 없다(그 함수 자신의 계약을 따른다).
- 타입 불변식의 술어도 같은 규칙을 따른다. 정의가 필요하면 `#[uses(p)]`에 쓴다.

```
#[summary("증명 문맥")]
mod m {
    spec fn small(n: u64) -> bool { n < 10 }
    spec fn bounded(n: u64) -> bool { small(n) && n > 0 }

    #[effects(pure)]
    fn f(n: u64) -> (r: u64)
        requires bounded(n),
        ensures r < 20,
    {
        n + n
    }
}
```

`bounded`는 `f`의 절이 부르므로 펼쳐지지만 `small`은 그 몸 안에 있어 이름뿐이다. `bmb verify`는 E0802(`counterexample: n = 10`)와 E0803 진단을 내고, 진단의 `fix`가 `#[uses(small)]`을 가리킨다. `f` 앞에 `#[uses(small)]`을 쓰면 `verified`다. `#[uses]`를 채우고 지우는 도구는 `compiler/README.md`에 있다.

## 판정

의무 하나의 판정은 셋 중 하나다.

- **증명됨.** 해법기가 부정을 `unsat`으로 답한 경우뿐이다.
- **반례가 있는 실패.** `sat`이면 그 의무의 코드(E0801~E0823)로 진단을 내고, 메시지 끝에 입력 값을 적는다: `counterexample: a = -9223372036854775808, b = -1`. 보일 입력이 없으면(정의 자신의 값이나 호출·루프가 열어 둔 값에서 실패) 반례 없이 낸다. 불투명한 `spec fn`이 걸려 있으면 `fix`가 그 이름을 댄다.
- **미결정.** 해법기가 `unknown`으로 답하거나 자원 한도를 넘으면 결정하지 못한 것으로 보고한다(E0811). 진단은 원래 의무의 코드를 `"obligation":"E0802"`처럼 함께 담고, 해법기의 사유를 적는다. 확인되지 않은 반례가 있으면 `unconfirmed counterexample`로 적는다(`tests/verify/fail/nonlinear.bmb`, `tests/verify/fail/proofs.jsonl`).
- **게이트가 돌지 못함.** 해법기가 없거나 고정 버전이 아니거나 오류로 끝나면 그 사실을 보고한다(E0812). 이 코드는 코퍼스에 고정된 사례가 없다(환경에 달린 진단).

미결정(E0811)과 게이트가 돌지 못한 경우(E0812)는 실패다(`docs/DESIGN.md` §6: unknown, timeout, 솔버 부재는 모두 실패). 정의의 `status`는 의무가 하나라도 증명되지 않으면 `failed`다.

```
#[summary("반례")]
mod m {
    #[effects(pure)]
    fn div(a: i64, b: i64) -> i64 { a / b }
}
```

`bmb verify`의 출력은 E0803(`` `/` may overflow `i64` in `m::div`; counterexample: a = -9223372036854775808, b = -1``), E0804(`divisor may be zero in `m::div`; counterexample: a = -9223372036854775808, b = 0`), 그리고 `{"def":"m::div","status":"failed","obligations":2,"failed":2,..}`다.

## 결정론

- 해법기는 고정 버전의 Z3이고 의무마다 자원 한도 `rlimit`이 같다. 두 값은 `scripts/toolchain.env`에 있고 컴파일러에 함께 빌드된다. 난수 시드는 0으로 고정이다. 판정은 시간이 아니라 `rlimit`으로 끊는다.
- 의무마다 해법기 상태를 초기화하고 옵션을 포함한 기반 전체를 다시 보낸다. 그래서 한 정의의 판정은 다른 정의가 먼저 증명되었는지에 달려 있지 않다.
- 진단은 위치 순으로 정렬되고 중복이 제거된다. 같은 입력이면 같은 바이트다. 상태 줄의 `rlimit` 값도 실패 코퍼스(`tests/verify/fail/arith.jsonl` 등)에 고정되어 있다.
- 질의의 기호는 처음 나온 순서의 정준 이름이라, 정의나 지역의 이름만 바꾼 프로그램의 질의 텍스트는 같다(ADR-0063).

## `build`와 `emit-llvm`

- `bmb build`는 모든 정의가 `verified`일 때만 실행 파일을 만든다. 하나라도 실패하면 `build refused: the program is not fully proved`로 끝나고 파일을 쓰지 않는다(종료 코드 1). 미결정(E0811)과 게이트가 돌지 못한 경우(E0812)도 여기에 든다.
- 탈출구(`assume`, `#[external_body]`)에 기대어 `verified`가 된 정의는 상태 줄에 `"trust":"assumed"`나 `"external"`가 붙고 `build`를 막지 않는다. 그 정의와 그것이 부르는 정의는 런타임 검사를 지우지 않는다(08장, `tests/run/escapes.bmb`).
- `bmb emit-llvm`은 증명된 의무의 검사를 지우고, 증명되지 않은 의무는 런타임 검사(trap)로 남긴 IR을 쓴다. 실패가 있으면 진단을 내고 종료 코드는 1이다. 증명은 호출자가 세운 `requires`와 피호출자가 증명한 `ensures`를 가정하므로, 실패한 정의와 호출로 이어진 정의들(부르는 쪽과 불리는 쪽 모두)은 증명된 의무의 검사도 남긴다(`compiler/src/lib.rs`의 `usable_proofs`, 백로그 1B-31).
- `bmb emit-llvm --all-checks`는 증명을 돌리지 않고 모든 의무를 런타임 검사로 남긴다. 계약을 믿지 않으므로 증명되지 않는 프로그램도 그대로 실행된다. E08xx 진단과 상태 줄은 나오지 않는다(`compiler/README.md`의 명령 목록).
- 증명된 의무만이 검사를 지운다(`docs/DESIGN.md` §3 원칙 9). 실행 출력까지 고정된 프로그램은 `tests/run/`에 있다(`tests/run/collatz.bmb`).

증명 캐시, 증분 증명, `--def`로 고른 정의만 증명하기, 시드를 바꿔 보는 안정성 검사, 계약 린트는 도구다. `compiler/README.md`에 있다.
