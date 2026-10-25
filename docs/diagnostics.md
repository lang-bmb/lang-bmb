# 진단 코드

> 생성 파일: `bmb codes --md`가 `compiler/src/diag.rs`의 `CODES`에서 만든다. 손으로 고치지 않는다.
> 코드는 바꾸거나 재사용하지 않는다. 진단 형식은 `compiler/README.md`.

## E01xx 어휘·구문

| 코드 | 뜻 |
| --- | --- |
| E0101 | unexpected character |
| E0102 | unterminated string literal |
| E0103 | syntax error: unexpected token |
| E0104 | integer literal suffix not allowed; the type comes from context |
| E0105 | integer literal too large |
| E0106 | block comments are not allowed; use `//` |
| E0107 | nesting too deep |

## E02xx Rust 습관(수정 제안 포함)

| 코드 | 뜻 |
| --- | --- |
| E0201 | `let` requires an explicit type |
| E0202 | macros are not allowed |
| E0203 | `return` is not allowed; end the block with a tail expression |
| E0204 | `for` and `loop` are not allowed; use `while` with `invariant` and `decreases` |
| E0205 | `as` casts are not allowed |
| E0206 | the `?` operator is not allowed |
| E0207 | construct not supported in milestone 1 |
| E0208 | lifetime annotations are not allowed |
| E0209 | function return type is required (write `-> ()` for unit) |
| E0210 | `mut` parameters are not allowed |
| E0211 | comparison operators cannot be chained |
| E0212 | `T::from(x)` / `x.into()` / `x.try_into()`: convert with `T::try_from(x).unwrap()` |
| E0213 | `T::try_from(x).expect(..)` is written `.unwrap()` |
| E0218 | a `Vec` is passed where a slice is expected: write `v.as_slice()` |
| E0219 | `drop(v)` and `mem::take` / `replace` / `swap` are not used |
| E0220 | `#[derive(..)]` is not used |
| E0221 | built-in variants are written bare (`Some`, not `Option::Some`) |
| E0222 | field shorthand is not allowed (`x: x`) |
| E0223 | struct update syntax `..` is not allowed |
| E0224 | a wildcard arm is not allowed |
| E0225 | `if let` is not allowed; use `match` |
| E0226 | `while let`, `let .. else` and pattern `let` are not allowed |
| E0227 | method calls are not allowed; call by path (`T::f(&x)`) |
| E0228 | the receiver needs its type (`self: &T`) |
| E0229 | `.unwrap()` / `.expect()` are not allowed; `match` both cases |
| E0230 | destructuring `let` is not allowed |
| E0231 | `(*p).x` is written `p.x` |
| E0232 | pattern form not allowed (guard, or, literal, nested, struct) |
| E0233 | tuples, tuple structs and struct-like variants are not allowed |

## E03xx 이름

| 코드 | 뜻 |
| --- | --- |
| E0301 | unknown name |
| E0302 | duplicate definition |
| E0303 | name already bound in this function (shadowing is not allowed) |
| E0304 | unknown function |
| E0305 | unknown type |
| E0306 | unknown field |
| E0307 | unknown variant |
| E0308 | `impl` block not in the unit of its type, or a second one |
| E0309 | a built-in name cannot be defined |
| E0310 | `self` names exactly the receiver |

## E04xx 타입·소유권

| 코드 | 뜻 |
| --- | --- |
| E0401 | type mismatch |
| E0402 | unsupported type |
| E0403 | references are only allowed in parameter position |
| E0404 | unknown method (no method lookup; only built-in methods exist) |
| E0405 | wrong number of arguments |
| E0406 | integer literal out of range for its type |
| E0407 | value is not indexable |
| E0408 | assignment to an immutable place |
| E0409 | `&` / `&mut` may only appear directly as a call argument |
| E0410 | a `&mut` argument aliases another argument of the same call |
| E0411 | cannot negate an unsigned value |
| E0412 | `T::try_from(x)` takes an integer and is used as `.unwrap()` or `.ok()` |
| E0413 | use of a moved `Vec` |
| E0414 | a `Vec` is moved on some paths and not on others |
| E0420 | malformed data type: wrong type arguments, or no field or variant |
| E0421 | a type contains itself |
| E0422 | struct literal fields: not a struct, a field given twice, or a field missing |
| E0423 | `match` needs an enum value |
| E0424 | the type of `None` / `Ok` / `Err` is not known here |
| E0425 | `==` on data types is not executable (compare the fields) |
| E0426 | `match` arms: one per variant, in declaration order |
| E0427 | a pattern names the wrong number of payload values |

## E05xx 명세

| 코드 | 뜻 |
| --- | --- |
| E0501 | contract clause must be a call to a named predicate (`spec fn ... -> bool`) |
| E0502 | vacuous predicate used in a contract |
| E0503 | specification-only construct used in executable code |
| E0504 | executable function called in a specification |
| E0505 | spec function called in executable code |
| E0506 | `while` requires `decreases` |
| E0507 | recursive function requires `decreases` |
| E0513 | conversions are not used in specifications (integer types mix there) |
| E0514 | `==` is not defined for `Vec` |
| E0508 | `old(..)` is only allowed in `ensures`, loop invariants and `assert` on a `&mut` parameter |
| E0509 | `#[uses(..)]` entry is not a spec function |
| E0510 | `#[trigger]` is only allowed inside `forall` |
| E0511 | `forall` requires at least one `#[trigger]` |
| E0512 | the triggers of a `forall` must together mention every bound variable |

## E06xx 효과·속성

| 코드 | 뜻 |
| --- | --- |
| E0601 | function requires an `#[effects(..)]` attribute |
| E0602 | unknown effect |
| E0603 | `pure` function calls a function with effect `io` |
| E0604 | spec functions take no `#[effects(..)]` attribute |
| E0605 | unknown or misplaced attribute |

## E07xx 프로그램 구조

| 코드 | 뜻 |
| --- | --- |
| E0701 | `main` must be `#[effects(io)] fn main() -> ()` |
| E0702 | more than one `main` |
| E0703 | items must be declared inside a `mod` |
| E0704 | `mod` requires a `#[summary("..")]` attribute |

## E08xx 증명 게이트

| 코드 | 뜻 |
| --- | --- |
| E0801 | precondition of a call may not hold |
| E0802 | postcondition may not hold |
| E0803 | arithmetic may overflow |
| E0804 | divisor may be zero |
| E0805 | index may be out of bounds |
| E0806 | loop invariant may not hold on entry |
| E0807 | loop invariant may not be preserved |
| E0808 | loop `decreases` may not decrease or may be negative |
| E0809 | recursive call may not decrease the measure |
| E0810 | assertion may not hold |
| E0811 | solver could not decide (resource limit or incompleteness) |
| E0812 | proof gate cannot run (solver missing, wrong version or failed) |
| E0813 | recursive spec function may not terminate |
| E0814 | construct not supported by the proof gate |
| E0815 | integer conversion may be out of range |
| E0816 | a `Vec` may exceed its maximum length |

## W09xx 계약 조정 린트(경고, `bmb lint`만)

| 코드 | 뜻 |
| --- | --- |
| W0901 | preconditions are unsatisfiable together: the function can never be called and its proofs are vacuous |
| W0902 | precondition is not needed by this function's proof |
| W0903 | postcondition is not used by any caller's proof |
| W0904 | the specification does not distinguish this change to the body (the mutant still verifies) |
| W0905 | a point of a verified function is unreachable in its proof: what follows is proved vacuously |
