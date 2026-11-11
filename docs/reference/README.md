# BMB 언어 레퍼런스 (초안)

> 상태: 초안(백로그 5A-01). 기술 대상은 동결 지문(`dev-docs/health/surface.json`)이 고정하는 받아들이는 언어다.
> 정본의 순서: 컴파일러의 판정(코퍼스가 고정한다) → 이 문서. 둘이 다르면 이 문서가 틀린 것이고, 고친다.
> 설계의 이유는 `docs/DESIGN.md`와 ADR(`dev-docs/adr/`)에 있다. 여기에는 무엇이 프로그램이고 무엇이 아닌지만 적는다.

## 범위

- **넣는 것:** 문법, 정적 의미(이름·타입·소유권·효과), 명세의 뜻, 증명 의무. 절마다 그 규칙을 어기면 나오는 진단 코드와 그 규칙을 보이는 코퍼스 파일을 적는다.
- **넣지 않는 것:** 튜토리얼, 도구(`bmb lint`·`weaken`·`uses`·`fix`, 맵, MCP: `compiler/README.md`), 표준 라이브러리의 API(`lib/std/`의 정의 요약과 맵 L1), 코드 생성의 세부.
- `W`로 시작하는 진단은 `bmb lint`와 `bmb weaken`만 내는 경고라 언어의 판정이 아니다. 목록은 `docs/diagnostics.md`.

## 장

| 장 | 내용 |
| --- | --- |
| [01 어휘와 구문](01-lexical-and-syntax.md) | 토큰, 리터럴, 주석과 요약, 구문의 뼈대 |
| [02 단위·항목·이름](02-units-items-names.md) | 단위(`mod`), 함수·타입·trait·상수 항목, 속성, 이름 찾기 |
| [03 타입과 값](03-types-and-values.md) | 정수·`f64`·`bool`·`char`, 배열·슬라이스, 구조체·열거형, 제네릭·trait, 소유 타입 |
| [04 식·문장·제어 흐름](04-expressions-and-control.md) | 식과 문장, `if`·`match`·루프, 받지 않는 Rust 습관과 그 대체 |
| [05 소유권과 참조](05-ownership.md) | 이동, 빌림, 바깥으로 나가지 않는 참조 |
| [06 계약과 명세](06-contracts-and-specs.md) | `requires`·`ensures`·불변식·감소식, `spec fn`·`proof fn`, 수량자 |
| [07 증명 게이트](07-proof-gate.md) | 의무의 종류, 증명 문맥(`#[uses]`), 판정과 반례 |
| [08 효과와 탈출구](08-effects-and-escape-hatches.md) | `#[effects]`, `#[external_body]`·`assume`와 근거 |

## 읽는 법

- 예시는 코퍼스 파일을 가리킨다. `tests/run/`은 실행 출력까지 고정된 프로그램, `tests/verify/pass`·`fail`은 증명 판정, `tests/check/err`는 검사 오류와 그 진단(`.jsonl`)이다.
- 진단 코드는 바뀌지 않고 재사용되지 않는다. 은퇴한 코드는 `docs/diagnostics.md` 끝에 있다.
- 게이트(`scripts/reference.py check`)가 확인하는 것: 언어의 진단 코드가 모두 어느 장에 나오는가, 장이 가리키는 코퍼스 파일이 있는가, 절마다 코드나 코퍼스가 하나 이상 있는가.

## 초안의 한계

장의 서술은 `compiler/README.md`의 규칙표, 컴파일러 소스, 코퍼스에서 옮겼고, 예시와 의심스러운 규칙은 컴파일러를 돌려 확인했다. 돌려 보지 못하고 문서·소스만 근거로 적은 것이 남아 있다(백로그 `5A-03`이 목록을 갖는다): 해제 순서(05장), 연산자 우선순위 전체와 `&&`·`||`의 단락 평가(04장), 메서드 찾기 순서(04장), `emit-llvm`이 지우고 남기는 검사(07장), 근거 해시의 입력과 타입 불변식의 자리 목록(06·08장), 내장 메서드의 전체 목록(03장). 적합성 시험(5A-02)이 절마다 시험을 붙이면서 이 목록을 지운다.

## 진단 코드 색인

<!-- reference:index begin (scripts/reference.py index --write; do not edit by hand) -->

| 코드 | 뜻 | 장 |
| --- | --- | --- |
| E0101 | unexpected character | [01](01-lexical-and-syntax.md) |
| E0102 | unterminated string literal | [01](01-lexical-and-syntax.md) |
| E0103 | syntax error: unexpected token | [01](01-lexical-and-syntax.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0104 | integer literal suffix not allowed; the type comes from context | [01](01-lexical-and-syntax.md), [03](03-types-and-values.md) |
| E0105 | integer literal too large | [01](01-lexical-and-syntax.md) |
| E0106 | block comments are not allowed; use `//` | [01](01-lexical-and-syntax.md) |
| E0107 | nesting too deep | [01](01-lexical-and-syntax.md), [04](04-expressions-and-control.md) |
| E0108 | malformed integer literal: a digit outside its base, or a base prefix without digits | [01](01-lexical-and-syntax.md) |
| E0109 | malformed float literal: digits on both sides of the point (`1.0`, `0.5`) and after an exponent `e` are required | [01](01-lexical-and-syntax.md) |
| E0201 | `let` requires an explicit type | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0202 | macros are not allowed | [04](04-expressions-and-control.md) |
| E0205 | `as` converts between integer types only (to `f64`: `f64::from(x)`) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0207 | construct not supported in milestone 1 | [02](02-units-items-names.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0208 | lifetime annotations are not allowed | [01](01-lexical-and-syntax.md), [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0209 | function return type is required (write `-> ()` for unit) | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0210 | `mut` parameters are not allowed | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0211 | comparison operators cannot be chained | [04](04-expressions-and-control.md) |
| E0212 | `T::from(x)` / `x.into()` / `x.try_into()`: convert with `x as T` | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0213 | `T::try_from(x).expect(..)` is written `.unwrap()` | [04](04-expressions-and-control.md) |
| E0214 | `use` is not used: names are written in full | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0215 | `unsafe` is not used: a body that cannot be proved is `#[external_body]` | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0216 | Verus's `#[verifier::external_body]`: BMB writes `#[external_body]` | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0217 | Verus's `admit()`: BMB writes `assume(p(..))` with a rationale | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0218 | a `Vec` or `String` is passed where a slice or `&str` is expected: write `v.as_slice()` or `s.as_str()` | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0219 | `drop(v)` and `mem::take` are not used | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0220 | `#[derive(..)]` is not used | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0221 | built-in variants are written bare (`Some`, not `Option::Some`) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0222 | field shorthand is not allowed (`x: x`) | [04](04-expressions-and-control.md) |
| E0223 | struct update syntax `..` is not allowed | [04](04-expressions-and-control.md) |
| E0224 | a wildcard arm is not allowed | [04](04-expressions-and-control.md) |
| E0225 | `if let` is not allowed; use `match` | [04](04-expressions-and-control.md) |
| E0226 | `while let`, `let .. else` and pattern `let` are not allowed | [04](04-expressions-and-control.md) |
| E0228 | the receiver needs its type (`self: &T`) | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0229 | `.unwrap()` / `.expect()` are not allowed; `match` both cases | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0230 | destructuring `let` is not allowed | [04](04-expressions-and-control.md) |
| E0231 | `(*p).x` is written `p.x` | [04](04-expressions-and-control.md) |
| E0232 | pattern form not allowed (guard, or, literal, nested, struct) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0233 | tuples, tuple structs and struct-like variants are not allowed | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0235 | a blanket impl `impl<T> Trait for T` is not used | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0236 | trait objects `dyn Trait` are not used | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0237 | a call of a generic function names its type arguments: `f::<T>(..)` | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0238 | `Rc`, `RefCell`, `Cell` and `Arc` are not used: values have one owner | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0239 | an owned value is moved out from behind a `&mut`: use `std::mem::replace` | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0240 | a `where` clause is not used: bounds go next to the type parameter | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0241 | `?` whose `None`/`Err` the function cannot return (another error type, or not an `Option`/`Result` function) | [04](04-expressions-and-control.md) |
| E0242 | `crate::`, `super::` or `self::` path: a path names a unit from the root | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0250 | `break` or `continue` outside a loop | [04](04-expressions-and-control.md) |
| E0251 | `break` with a value outside a `loop` | [04](04-expressions-and-control.md) |
| E0252 | `for` over something other than an integer range | [04](04-expressions-and-control.md) |
| E0253 | `decreases` on a `for` loop (its measure is automatic) | [04](04-expressions-and-control.md) |
| E0254 | `#[invariant(p)]`: `p` is not a spec function `(x: &T) -> bool` over the type's parameters, or states nothing | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0255 | more than one `#[invariant(..)]` on a type: join the conditions with `&&` in one predicate | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0256 | a `proof fn` takes no effect attribute and no return type | [02](02-units-items-names.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0257 | ghost code (a `proof fn` body or a `proof { .. }` block) cannot run executable code: no executable call, assignment, `&mut` or loop | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0258 | a `proof fn` is called only from ghost code: a `proof { .. }` block or another `proof fn` | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0259 | a `const`'s value is not a constant expression: literals, other `const`s, operators, `if`, `match`, the wrapping and saturating methods, `as`, arrays, structs, variants, fields and elements (ADR-0052 §3) | [04](04-expressions-and-control.md) |
| E0260 | a `const`'s evaluation failed: overflow, division by zero, a shift of the width or more, a conversion out of range, an index past the end (ADR-0052 §4) | [04](04-expressions-and-control.md) |
| E0261 | a `static` item: an immutable one is a `const`, global mutable state is passed as an argument (ADR-0052 §7) | [04](04-expressions-and-control.md) |
| E0262 | a closure that captures a local is written `move`: captures move into it (ADR-0053 §2) | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0263 | a closure's parameters and result have their types written (ADR-0053 §2) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0264 | a function pointer type, a returned closure, or a closure that is not the argument of an `Fn`-bounded parameter (ADR-0053 §1, §3) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0265 | an argument-position `impl Fn(..)`: the form is a type parameter `F: Fn(..)` (ADR-0053 §1) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0301 | unknown name | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0302 | duplicate definition | [02](02-units-items-names.md) |
| E0303 | a local name hides a visible one (a parameter, an outer local, the result), or is bound elsewhere in the function with another type (ADR-0095 A3) | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0304 | unknown function | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0305 | unknown type or trait | [02](02-units-items-names.md) |
| E0306 | unknown field | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0307 | unknown variant | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0308 | `impl` block not in the unit of its type, or a second one with the same bounds (one block per list of bounds, ADR-0097) | [02](02-units-items-names.md) |
| E0309 | a built-in name cannot be defined | [02](02-units-items-names.md), [03](03-types-and-values.md) |
| E0310 | `self` names exactly the receiver | [02](02-units-items-names.md) |
| E0311 | two impls of a trait for types that can be the same | [02](02-units-items-names.md), [03](03-types-and-values.md) |
| E0312 | a trait impl in neither the unit of the trait nor the unit of the type | [02](02-units-items-names.md) |
| E0313 | a method call names a method of more than one trait; call by path | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0314 | a method call names a function that has no receiver or type parameters of its own; call by path | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0315 | a trait impl misses a method of the trait, or has one the trait lacks | [02](02-units-items-names.md) |
| E0316 | a trait impl's function does not match the trait's signature | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0317 | a `break` or `continue` names a label no loop around it has | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0318 | `const`s that refer to each other (ADR-0052 §3) | [02](02-units-items-names.md) |
| E0401 | type mismatch | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [05](05-ownership.md), [06](06-contracts-and-specs.md) |
| E0402 | unsupported type | [03](03-types-and-values.md), [05](05-ownership.md), [06](06-contracts-and-specs.md) |
| E0403 | references are only allowed in parameter position | [03](03-types-and-values.md), [05](05-ownership.md) |
| E0404 | unknown method (no method lookup; only built-in methods exist) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0405 | wrong number of arguments | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0406 | integer literal out of range for its type | [03](03-types-and-values.md) |
| E0407 | value is not indexable | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0408 | assignment to an immutable place | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0409 | `&` / `&mut` may only appear directly as a call argument | [03](03-types-and-values.md), [05](05-ownership.md) |
| E0410 | a `&mut` argument aliases another argument of the same call | [05](05-ownership.md) |
| E0411 | cannot negate an unsigned value | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0412 | `T::try_from(x)` takes an integer and is used as `.unwrap()` or `.ok()` | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0413 | use of a moved value, or a move out of a place | [05](05-ownership.md) |
| E0414 | a value that moves is moved on some paths and not on others | [05](05-ownership.md) |
| E0420 | malformed data type: wrong type arguments, or no field or variant | [03](03-types-and-values.md) |
| E0421 | a type contains itself | [03](03-types-and-values.md) |
| E0422 | struct literal fields: not a struct, a field given twice, or a field missing | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0423 | `match` needs an enum value | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0424 | the type of `None` / `Ok` / `Err` is not known here | [03](03-types-and-values.md) |
| E0425 | `==` on data types is not executable (compare the fields) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0426 | `match` arms: one per variant, in declaration order | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [07](07-proof-gate.md) |
| E0427 | a pattern names the wrong number of payload values | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0428 | a type argument does not satisfy a bound of its parameter | [03](03-types-and-values.md), [05](05-ownership.md) |
| E0429 | wrong number of type arguments in a call | [03](03-types-and-values.md) |
| E0430 | a recursive generic call wraps a type parameter in a larger type | [03](03-types-and-values.md) |
| E0431 | `==` on a recursive type that holds a `Vec` (in a specification) | [03](03-types-and-values.md) |
| E0432 | a value of a recursive struct is built in a specification | [03](03-types-and-values.md) |
| E0433 | a value is used whole after one of its fields was moved | [05](05-ownership.md) |
| E0434 | struct literal fields must be given in declaration order (the order of evaluation) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0435 | a range `lo..hi` outside the source argument of `copy_within` and the bounds of a `&str` argument `&s[lo..hi]` | [03](03-types-and-values.md) |
| E0436 | a `std::process::Child` built or read by its fields (it comes from `std::process::spawn`) | [03](03-types-and-values.md) |
| E0437 | a type a `const` cannot have: a `const` holds a value type, an integer, `bool` or `char`, or an array, struct or enum of them (ADR-0052 §2) | [03](03-types-and-values.md) |
| E0501 | a contract clause with a quantifier must be a call to a named predicate (`spec fn ... -> bool`); one without is an expression (ADR-0095 A2) | [06](06-contracts-and-specs.md) |
| E0502 | vacuous predicate used in a contract | [06](06-contracts-and-specs.md) |
| E0503 | specification-only construct used in executable code | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0504 | executable function called in a specification | [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0505 | spec function called in executable code | [06](06-contracts-and-specs.md) |
| E0506 | `while` requires `decreases` | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0507 | recursive function requires `decreases` | [06](06-contracts-and-specs.md) |
| E0508 | `old(..)` is only allowed in `ensures`, loop invariants and `assert` on a `&mut` parameter | [06](06-contracts-and-specs.md) |
| E0509 | `#[uses(..)]` entry is not a spec function | [06](06-contracts-and-specs.md), [07](07-proof-gate.md), [08](08-effects-and-escape-hatches.md) |
| E0510 | `#[trigger]` is only allowed inside `forall` or `exists` | [06](06-contracts-and-specs.md) |
| E0511 | `forall` and `exists` require at least one `#[trigger]` | [06](06-contracts-and-specs.md) |
| E0512 | the triggers of a `forall` or `exists` must together mention every bound variable | [06](06-contracts-and-specs.md) |
| E0513 | conversions are not used in specifications (integer types mix there) | [03](03-types-and-values.md), [06](06-contracts-and-specs.md) |
| E0514 | `==` is not defined for `Vec` | [03](03-types-and-values.md), [06](06-contracts-and-specs.md) |
| E0515 | `assert(..) by (bit_vector)` takes integer, `f64` and `bool` terms only (no quantifier, `old` or data value; an element, field or length that names a `let` of the assertion) | [06](06-contracts-and-specs.md) |
| E0516 | unknown prover in `assert(..) by (..)` (the one prover is `bit_vector`) | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0601 | function requires an `#[effects(..)]` attribute | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0602 | unknown effect | [08](08-effects-and-escape-hatches.md) |
| E0603 | `pure` function calls a function with effect `io` | [08](08-effects-and-escape-hatches.md) |
| E0604 | spec functions take no `#[effects(..)]` attribute | [02](02-units-items-names.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0605 | unknown or misplaced attribute | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0606 | the body needs an effect the function does not declare | [08](08-effects-and-escape-hatches.md) |
| E0607 | `pure` listed with another effect, or an effect listed twice | [08](08-effects-and-escape-hatches.md) |
| E0608 | a function value whose effects the generic function that calls it does not declare (ADR-0053 §5) | [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0701 | `main` must be `#[effects(io)] fn main() -> ()` | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0702 | more than one `main` | [02](02-units-items-names.md) |
| E0703 | items must be declared inside a `mod` | [01](01-lexical-and-syntax.md), [02](02-units-items-names.md) |
| E0704 | `mod` requires a `#[summary("..")]` attribute | [01](01-lexical-and-syntax.md), [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0801 | precondition of a call may not hold | [07](07-proof-gate.md) |
| E0802 | postcondition may not hold | [07](07-proof-gate.md) |
| E0803 | arithmetic may overflow | [07](07-proof-gate.md) |
| E0804 | divisor may be zero | [07](07-proof-gate.md) |
| E0805 | index may be out of bounds | [07](07-proof-gate.md) |
| E0806 | loop invariant may not hold on entry | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0807 | loop invariant may not be preserved | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0808 | loop `decreases` may not decrease or may be negative | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0809 | recursive call may not decrease the measure | [07](07-proof-gate.md) |
| E0810 | assertion may not hold | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0811 | solver could not decide (resource limit or incompleteness) | [07](07-proof-gate.md) |
| E0812 | proof gate cannot run (solver missing, wrong version or failed) | [07](07-proof-gate.md) |
| E0813 | recursive spec function may not terminate | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0815 | integer conversion may be out of range (to `f64`: a 64-bit value may be above 2^53 and inexact) | [07](07-proof-gate.md) |
| E0816 | a `Vec` may exceed its maximum length | [07](07-proof-gate.md) |
| E0817 | a trait impl's function does not refine the trait's contract | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0818 | a `by (bit_vector)` sub-proof may not hold, or its assertion has no bit-vector translation | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0819 | an argument of a `proof fn` call may be outside its parameter's type (ghost arithmetic is mathematical, ADR-0062 §2) | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0820 | a string position may not be a character boundary (`&s[a..b]` needs `s.is_char_boundary(a)`, `s.is_char_boundary(b)` and `a <= b`, ADR-0050 §2) | [07](07-proof-gate.md) |
| E0823 | a type invariant may not hold where a value leaves the function's hands: a call argument, a return, a `&mut` parameter at exit, a loop head (ADR-0060 §2) | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E1001 | an escape (`assume`, `#[external_body]`) without a `#[rationale(..)]` before it | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E1002 | a malformed `#[rationale(..)]`: `kind` outside the list, no explanation, or not before an escape | [01](01-lexical-and-syntax.md), [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E1003 | `assume` takes one named predicate call, in executable code | [08](08-effects-and-escape-hatches.md) |
| E1004 | `#[external_body]` on a spec fn, a proof fn or `main` | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |

<!-- reference:index end -->
