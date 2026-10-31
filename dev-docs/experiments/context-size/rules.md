# 프롬프트 언어 규칙 절: 진단이 알려 주는 것

생성: `python3 dev-docs/experiments/context-size/rules.py`(모델 호출 없음). 해석은 `README.md` 2D-28 절.

| 규칙 줄 | 탐침 | 명령 | 진단 | 메시지 | 고칠 길(`fix`) |
| --- | --- | --- | --- | --- | --- |
| units | item outside `mod` | check | E0703 | items must be declared inside a `mod` | wrap the item in `#[summary("..")] mod name { .. }` |
| units | `mod` without summary | check | E0704 | unit `p` has no summary | add `#[summary("one line")]` above `mod` |
| units | other unit without path | check | E0304 | unknown function `g` | no `g` in unit `p`; other units are reached by full path `unit::g` |
| units | `crate::` path | check | E0103 | expected name, found `crate` |  |
| units | `use` | check | E0214 | `use q::g;` is not used: write `q::g` where it is used | delete the `use`; its names are rewritten to full paths (치환) |
| units | `pub fn` | check | E0207 | `pub` is not used; every definition of a unit is part of its surface | remove `pub` (치환) |
| units | `struct` | check | 없음 | | |
| units | `enum` and `match` | check | 없음 | | |
| units | generics | check | 없음 | | |
| units | closure | check | E0207 | closures are not supported in milestone 1 |  |
| units | closure | check | E0301 | unknown name `c` |  |
| units | `assert!` | check | E0202 | macro `assert!` is not allowed | use `assert(cond);` (a proof obligation, not a runtime check) |
| units | `vec!` | check | E0202 | macro `vec!` is not allowed | a fixed array is `[a, b]` or `[v; N]`; a `Vec` is `Vec::with_capacity(n)` and `push` |
| types | `i32` | check | 없음 | | |
| types | returned reference | check | E0403 | a reference cannot be a return type | references are second-class: pass them as arguments only (ADR-0015) |
| types | `&mut` alias | check | E0410 | `a` is passed as `&mut` and also used by another argument |  |
| let/effects | `let` without type | check | E0201 | `let x` requires an explicit type | write `x: i64` (치환) |
| let/effects | no return type | check | E0209 | function return type is required | add `-> ()` (치환) |
| let/effects | no effect | check | E0601 | `f` has no `#[effects(..)]` attribute | add `#[effects(pure)]` above the function (치환) |
| let/effects | `pure` prints | check | E0603 | `print_i64` has effect `io`, but `p::f` is `pure` | declare `#[effects(io)]` on `p::f`, or move the call out of it |
| no return.. | `return` | check | 없음 | | |
| no return.. | `for` over a range | check | 없음 | | |
| no return.. | `loop` and `break` | check | E0506 | `loop` has no `decreases` clause | add `decreases <expr>,` before the loop body |
| no return.. | `while` with `continue` | check | 없음 | | |
| no return.. | `as` | check | E0205 | `as` casts are not allowed | convert with a range obligation (ADR-0028) (치환) |
| no return.. | `?` | check | E0241 | `?` on `Option<i64>` returns its `None` from a function returning `i64` | `?` needs the function to return `Option<..>`; otherwise `match` the value |
| no return.. | `.unwrap()` | check | E0229 | `.unwrap()` is not allowed | `match` both cases: `match x { None => .., Some(v) => v }` |
| arithmetic | overflow | verify | E0803 | `+` may overflow `i64` in `p::f`; counterexample: x = 9223372036854775807 |  |
| arithmetic | division | verify | E0803 | `/` may overflow `i64` in `p::f`; counterexample: x = -9223372036854775808, y = -1 |  |
| arithmetic | division | verify | E0804 | divisor may be zero in `p::f`; counterexample: x = -9223372036854775808, y = 0 |  |
| arithmetic | indexing | verify | E0805 | index may be out of bounds in `p::f`; counterexample: a = [] (len 0), i = 0 |  |
| contracts | expression clause | check | E0501 | contract clause is not a named predicate | move the condition into `spec fn name(..) -> bool { .. }` and call it here |
| contracts | `ensures` before `requires` | check | E0103 | clauses must appear in the order `requires`, `ensures`, `decreases` |  |
| contracts | unnamed result | check | E0301 | unknown name `result` | name the result: `-> (r: T)` and use `r` |
| contracts | `old` on a value | check | E0508 | `old(x)` is only allowed in `ensures`, loop invariants and `assert`, on a `&mut` parameter `x` |  |
| opaque | body needed, no `#[uses]` | verify | E0802 | postcondition `pos` may not hold in `p::f` | the proof sees `pos` only by name; if the failure depends on its definition, add to #[uses(pos)] |
| loops | `while` without `decreases` | check | E0506 | `while` has no `decreases` clause | add `decreases <expr>,` before the loop body |
| loops | `while` without `invariant` | verify | E0802 | postcondition `is3` may not hold in `p::f` |  |
| loops | recursion without `decreases` | check | E0507 | `p::f` is recursive and has no `decreases` clause | add `decreases <expr>,` after `ensures` |
| loops | `forall` without trigger | check | E0511 | `forall` has no `#[trigger]` | mark a term that mentions every bound variable, e.g. `#[trigger] a[i]` |
| loops | trigger misses a variable | check | E0512 | no trigger mentions bound variable `j` |  |
| loops | mixed integer types in code | check | E0401 | `+` needs operands of one type, found `u64` and `usize` | convert the `u64` operand: `usize::try_from(x).unwrap()` |
