# 08 Effects and escape hatches

This chapter covers a function's effect declaration `#[effects(..)]` and its check, the range of attributes an item accepts, and the two escape hatches that accept a fact without proof, `assume` and `#[external_body]`, together with their rationale `#[rationale(..)]`. The diagnostics here are E06xx (effects and attributes) and E10xx (escape hatches). Proof obligations and verdicts are in [07 Proof gate](07-proof-gate.md).

## The set of effects

An effect is a set of atomic effects, and `pure` is the name of the empty set (ADR-0030 §1). There are four atomic effects.

| Effect | Meaning | Source |
| --- | --- | --- |
| `alloc` | Heap allocation. Aborts if memory runs short | `Vec::new`, `Vec::with_capacity`, `push`, `clone` (copying a value that holds memory), `Box::new` and so on |
| `io` | Interaction with the outside | Output builtins such as `print_i64` and the system builtins (ADR-0091) |
| `nondet` | Reads a value that can differ from run to run even for the same program and the same arguments | Command-line arguments, standard input, file reads, environment variables, the clock |
| `panic` | May end in a trap during execution | A definition with an escape hatch: an `assume` statement, `#[external_body]` (ADR-0029) |

- Effects are orthogonal to each other. `io` does not imply any other effect. Input builtins have `io` and `nondet` together (for example, `std::io::read_line` is `alloc, io, nondet`).
- An abort from running out of memory belongs to `alloc`, not `panic`. Deallocation is not an effect.
- The corpus that shows the effect rules is `tests/check/err/effects.bmb` and its expected diagnostics `tests/check/err/effects.jsonl`.

## The `#[effects(..)]` declaration

- An executable function must have `#[effects(..)]`. Without it, it is an error (E0601, `tests/check/err/items.bmb`). The same holds for the declaration of a trait method and for a function of an impl.
- The argument is either `pure` alone or the names of atomic effects. The normal form is in name order: `#[effects(alloc, io)]`.
- An unknown name, an argument that is not an identifier, and the empty list `#[effects()]` are errors (E0602, `unknown` and `none` in `tests/check/err/effects.bmb`).
- Writing `pure` together with another effect, or writing one effect twice, is an error (E0607, `two_effects`, `twice`, `pure_too`). The fix of the diagnostic is the normal form.
- Writing `#[effects(..)]` twice on one function is an error (E0605, `tests/check/err/attrs.bmb`).
- `main` is `#[effects(io)] fn main() -> ()`, and other effects can be written together with `io`. Without `io` it is an error (E0701, chapter 02).

## Declaration and body

- The **required effects** of a function are the union of the effects of the body's own operations (allocation, builtin calls, `assume`) and the **declared** effects of the executable functions the body calls. The declaration must contain the required effects. If an effect is missing, one E0606 diagnostic is reported on that function's attribute, and the message gives the first source for each missing effect: `` `both` needs `alloc` (`Box::new`) and `io` (a call of `print_i64`) but declares `#[effects(pure)]` ``.
- If a `pure` function calls an `io` function, an E0603 diagnostic is reported at that call site (`prints` in `tests/check/err/effects.bmb`).
- A declaration wider than what is required is not an error. The declaration is the interface (`wider`). A caller looks at the callee's declaration, not its body. So when a callee's body changes and its effects grow, the error is reported on the callee, not on the caller (ADR-0030 §3).
- In mutual recursion the rule is also the same for each direct call (`ping`, `pong`).
- A function of a trait impl cannot have an effect that the trait method does not declare (E0316, `traits` in `tests/check/err/effects.bmb`). The effect of a call through a bound is the declaration of the trait method.
- The body of an `#[external_body]` function is checked by the same rule too. Effects are not something that is taken on trust.
- A `loop` in a function that declares `nondet` is accepted without `decreases` (ADR-0094, chapter 06).

## Effects of specifications and ghost code

- A `spec fn` has no effects and does not write an effect attribute. Writing one is an error (E0604, `spec_with_effect` in `tests/check/err/effects.bmb`).
- A `proof fn` does not write an effect attribute either (E0256, `tests/check/err/proofs.bmb`). Ghost code does not remain in the executable.
- A specification context (a contract clause, an invariant, the condition of an `assert`) and a `proof { }` block add nothing to the required effects of the enclosing function, because they cannot call an executable function (E0504 and E0257 in chapter 06).

## Effects of function values

- Inside a generic function `g` with the bound `F: Fn(A) -> R`, the effect of a call `f(a)` is the effect that `g` declares (ADR-0053 §5).
- So the effects of a function or closure passed to `g` must be within the declaration of `g`. If what is passed is a function, an E0608 diagnostic is reported on that argument (if it is a closure, E0603 is reported on that call in the closure body: the effect of a closure is the declaration of the receiving side `F`): `` `shout` has effect `io`, which `fve::apply` does not declare for its `F` `` (`tests/check/err/fn_values.bmb`). The fix is to declare that effect on `g` or to pass a function that does not have that effect.
- The forms that pass are in `tests/run/fn_values.bmb`.

## Attributes an item accepts

The list of attributes is closed. An attribute that does not belong in that place and an unknown attribute are errors (E0605), and the fix of the diagnostic is to remove that attribute (`tests/check/err/attrs.bmb`, `tests/check/err/habits.bmb`, `tests/check/err/summaries.bmb`).

| Place | Attributes accepted |
| --- | --- |
| Unit (`mod`) | `#[summary("..")]` (required) |
| Function | `#[summary("..")]`, `#[effects(..)]`, `#[uses(..)]`, `#[external_body]`, `#[rationale(..)]` |
| Struct, enum | `#[summary("..")]`, `#[invariant(p)]` |
| trait, `const` | `#[summary("..")]` |
| Statement | Only `#[rationale(..)]` before `assume(..)` (anything else is E1002) |
| Term of a specification expression | `#[trigger]` (chapter 06) |

- Writing `#[summary(..)]` twice on one definition is an error (E0605). An argument that is not a single string is an error (E0704, chapter 02).
- Rust's `#[derive(..)]` is handled separately by a Rust-habit diagnostic (E0220), and `#[inline]`, `#[repr(..)]` and the like are unknown attributes (E0605).
- The argument rules of `#[uses(..)]` are in chapter 06 (E0509), and the rules of `#[invariant(p)]` are in chapter 06 (E0254, E0255).

## Escape hatch: `assume`

- `assume(p(..));` is a statement of executable code. From that point on, `p(..)` is accepted as a fact without proof (ADR-0029).
- The argument is a single call of a named predicate (`spec fn .. -> bool`). Any other expression (`assume(x < 10)`) is an error (E1003). An `assume` inside a specification or ghost code (`proof fn`, `proof { }`) is also an error (E1003). A ghost fact is proved with `assert` or a `proof fn`.
- There must be a `#[rationale(..)]` immediately before it. Without one it is an error (E1001).
- A function that contains `assume` must declare `panic` (E0606, `missing_panic` in `tests/check/err/escapes.bmb`). A function that calls that function also declares `panic`, by the call rule.

## Escape hatch: `#[external_body]`

- `#[external_body]` accepts the contract of an executable function without proving its body. A caller proves the `requires` and assumes the `ensures`, as for any other function.
- It is not written on a `spec fn`, a `proof fn` or `main` (E1004, `external_spec` in `tests/check/err/escapes.bmb`).
- The function's attributes must include `#[rationale(..)]` with it. Without one it is an error (E1001, `external_without`).
- Conversely, a `#[rationale(..)]` on a function without `#[external_body]` is an error (E1002, `lone` in `tests/check/err/attr_places.bmb`).
- That function is a source of `panic`. Its body goes through the type, ownership and effect checks as usual.
- The Rust and Verus forms are not accepted: `unsafe` (E0215), `#[verifier::external_body]` (E0216), `admit()` (E0217). They are in chapters 02 and 04.

## Rationale: `#[rationale(..)]`

- The form is `#[rationale(kind = K, "description")]`. `K` is one of `nonlinear`, `solver_limit`, `boundary`, `performance`, and the description is a single non-empty string.
- A wrong format is an error (E1002): a `kind` outside the list, a missing or repeated `kind`, a missing description, an empty description or more than one description, an unknown key (`bad_kind` and `no_text` in `tests/check/err/escapes.bmb`).
- The only attribute that comes before a statement is `#[rationale(..)]` before `assume(..)`. A `#[rationale(..)]` before another statement and any other attribute before a statement are errors (E1002, `not_before_assume`, `tests/check/err/summaries.bmb`).
- `hash = ".."` is the hash of the code the rationale is bound to, and a tool writes it (ADR-0006, ADR-0029 §3): `#[rationale(kind = boundary, hash = "c8c486f5ddfa4a41", "the outside world")]` (`tests/run/escapes.bmb`). The hash is computed from the normal form of the definition without the rationale and the text of the `spec fn`s that definition calls. So editing only the description text leaves the hash as it is, and editing the code changes it.
- A rationale whose `hash` is missing or differs from the current value is unconfirmed. Unconfirmed is not an error of the language and does not block `build`. The warning that reports it and the tool that writes the hash are in `compiler/README.md` and `docs/diagnostics.md`.

## Escape hatches and trust levels

- A program with escape hatches first has every definition proved without trusted facts: neither the fact of an `assume` nor the `ensures` of an unproved callee is used. Only the definitions that fail there and can reach an escape hatch are proved again with the trusted facts added (ADR-0029 §2).
- The status line of `bmb verify` records the result as `trust`. A definition proved without trusted facts has no key, a definition proved with trusted facts has `"trust":"assumed"`, and an `#[external_body]` function has 0 obligations and `"trust":"external"`. The `status` is `verified` for all three. A caller proved by relying on the `ensures` of a function that is `assumed` or `external` is also `assumed`.
- A trusted fact is not grounds for removing a runtime check (ADR-0029 §4). A definition that is not proved without trusted facts, and every definition it calls, are compiled with their checks left in. So a false `assume` can end in a trap, not in undefined behavior, and this is the meaning of the `panic` effect.
- Escape hatches do not block `build`. A program with no escape hatch is proved once and has no `trust` key.
- Examples fixed down to their execution output are `tests/run/escapes.bmb` and `tests/run/collatz.bmb` (which accepts with `assume` the upper bound of a loop whose termination cannot be proved).

The warning to narrow effects and the warning that reports an unnecessary `assume` are lints. They are not verdicts of the language and are in `compiler/README.md`.
