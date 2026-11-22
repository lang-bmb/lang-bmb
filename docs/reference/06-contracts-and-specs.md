# 06 Contracts and specifications

This chapter covers the syntax for writing specifications and its static rules: a function's `requires` and `ensures` and the result name, a loop's `invariant` and `decreases`, `assert`, `old(..)`, the scope and the integers of specification expressions, quantifiers and triggers, `spec fn`, `proof fn` and `proof { }`, type invariants, trait contracts, and the contracts of function values. The diagnostics here are the E05xx that the checker (`bmb check`) issues. The proof obligations that specifications create and their failures (E08xx) are in [07 Proof gate](07-proof-gate.md).

## Contract clauses

- A function's contract is the clauses after the signature and before the body: `requires c1, c2,` `ensures c3,` `decreases d,`. The order is fixed as `requires`, `ensures`, `decreases`, and breaking it is a syntax error (E0103, `order` in `tests/check/err/specs.bmb`).
- One clause is a specification expression of type `bool`. Clauses separated by commas must all hold.
- A clause that holds a quantifier (`forall` or `exists`) must be a single call of a named predicate (`spec fn .. -> bool`). Otherwise it is an error (E0501, ADR-0095 A2). A clause without a quantifier writes the expression as it is (`tests/verify/pass/inline_contracts.bmb`).
- If a clause is a call of a predicate whose body is `true`, it is an error (E0502, `vacuous` in `tests/check/err/specs.bmb`).
- To refer to the result in `ensures`, give the return type a name: `-> (r: T)`. Using a name such as `result` without naming it is an unknown-name error (E0301).
- `requires` sees the parameters, and `ensures` sees the parameters and the result name. A `&mut` parameter in `ensures` is the value after the function ends, and the value at entry is written with `old(..)` ("`old`" below).

## Specification context and executable context

The specification context is the contract clauses, a loop's `invariant` and `decreases`, the condition of `assert` and `assume`, and the body of a `spec fn`. The rest of a function body is the executable context. The body of a `proof fn` and a `proof { }` block are the ghost context, and the expressions inside them are specification expressions ("`proof fn` and `proof` blocks" below).

| Rule | Diagnostic | Corpus |
| --- | --- | --- |
| `forall`, `exists`, `==>`, `old(..)` and `#[trigger]` are not used in the executable context | E0503 | `tests/check/err/specs.bmb` |
| `let mut`, assignment, loops, `break`, `return`, `?` and `assert` are not used in the specification context | E0503 | `tests/check/err/spec_mode.bmb` |
| The specification context does not call executable functions | E0504 | `calls_exec` in `tests/check/err/specs.bmb` |
| The executable context does not call a `spec fn` | E0505 | `calls_spec` in `tests/check/err/specs.bmb` |

What comes in a specification expression is literals, names, arithmetic, comparison and logical operations, `==>`, indexing, fields and `.len()`, `if` and `match`, a `let` that binds a value, `spec fn` calls, quantifiers, and `old(..)`. A specification also reads moved values (ADR-0098, `tests/verify/pass/moved_specs.bmb`).

## Integers in specifications

- There is no separate `int` type. Specification expressions use the names of the machine integer types, but the arithmetic is read as mathematical integers (ADR-0017). So specification expressions have no proof obligations for overflow, division by zero or index range. `requires a + b <= 255` (with `a`, `b` of type `u8`) means exactly that the sum is at most 255.
- In specifications and ghost code an integer type widens to a type that holds the range of the value (ADR-0099, `tests/verify/pass/spec_widening.bmb`). Different integer types are mixed without conversion.
- In a specification a conversion `x as T` that narrows the range is an error (E0513), and using a wide value in a position where a narrow type is expected (the result of a `spec fn`, a `let` with a written type) is an error (E0401, `tests/check/err/spec_widening.bmb`, `tests/check/err/conversions.bmb`). The arguments of a `spec fn` are the exception: a value of another integer type can be passed, and for a value outside the range of the parameter the value of that call is not determined (ADR-0022). So `small(n)`, which passes the `u64` value 256 to `small(b: u8)`, says nothing. A widening `as` is accepted as an input alias equal to the value itself ("Input aliases" in `compiler/README.md`).
- `Vec` has no `==` or `!=` (E0514, `tests/check/err/vec.bmb`). Compare the length and the elements with a predicate over `v.as_slice()` (ADR-0031).

## Quantifiers and triggers

- The forms are `forall|i: T, j: U| expr` and `exists|i: T| expr`. The rules of the two quantifiers are the same (ADR-0079, `tests/verify/pass/exists.bmb`).
- The type of a bound variable is an integer. Any other type is an error (E0402).
- The body of a quantifier must have at least one term marked with `#[trigger]`. Without one it is an error (E0511). The triggers of one quantifier form a single multi-pattern, and every bound variable must appear in some trigger. A missing variable is an error (E0512, `triggers` in `tests/check/err/specs.bmb`, `tests/check/err/exists.bmb`).
- A `#[trigger]` in a specification expression outside a quantifier is an error (E0510, `bare_trigger` in `tests/check/err/specs.bmb`).
- Quantifiers come in the body of a `spec fn`, in invariants and in `assert`. In a contract clause they come only as a call of a predicate that holds them (E0501).

## `spec fn`

- `spec fn name(params) -> T { spec expr }` is a function called only from specifications (E0505). It has no effect attribute (E0604 in [08](08-effects-and-escape-hatches.md)).
- If it recurses, `decreases` is required (E0507). The proof obligation that the measure decreases is in chapter 07 (E0813, `contains_prefix` in `tests/verify/pass/proofs.bmb`, `tests/verify/fail/contracts.bmb`).
- **Definitions are opaque by default.** A proof sees a `spec fn` only by name, and unfolds its definition only when it is in the context of that proof (ADR-0017 §4). The context is the union of the default context and `#[uses(..)]`. The default context is the `spec fn`s called by the definition's own clauses, invariants, `assert`s and `proof` code and the `spec fn`s called by the contracts of its callees; it does not go into the `spec fn`s that those bodies call in turn (ADR-0095 A1, `tests/verify/pass/default_context.bmb`). The detailed rules are in chapter 07, "Proof context".
- `#[uses(a, unit::b, <T as Tr>::m)]` is written before a function and accepts only names of `spec fn`s. The name of an executable function, a name that does not exist and an argument that is not a name are errors (E0509, `bad_uses` in `tests/check/err/specs.bmb`, `tests/check/err/spec_mode.bmb`, `tests/verify/pass/uses_impl_path.bmb`).

## Loop `invariant` and `decreases`

- `while cond invariant p1, p2, decreases d, { body }`. The clauses come between the condition and the body. An invariant clause is a `bool` specification expression and can hold a quantifier directly.
- `while` and `loop` require `decreases`. Without it it is an error (E0506, `no_decreases` in `tests/check/err/specs.bmb`, `tests/check/err/loop_exits.bmb`). A range `for i in lo..hi` does not write `decreases` (`tests/verify/pass/ranges.bmb`).
- There is one exception: a `loop` in a function that declares the `nondet` effect is accepted without `decreases`. The termination of that loop is not proved (ADR-0094).
- All that the code after a loop knows is the invariant and the condition under which the loop was left. The obligations of entry, preservation and decrease are in chapter 07 (E0806, E0807, E0808, `tests/verify/pass/loops.bmb`, `tests/verify/fail/loops.bmb`).

## Recursion and `decreases`

- A recursive function (mutual recursion included) requires `decreases`. This is the same for executable functions, `spec fn` and `proof fn`. Without it it is an error (E0507, `rec`, `ping` and `pong` in `tests/check/err/specs.bmb`, `tests/check/err/proofs.bmb`).
- The measure is an integer expression or a value of a recursive enum (ADR-0047 §3). An integer measure must get smaller at every call and not go below 0, and an enum measure must be a part of the caller's value (`tests/verify/pass/recursion.bmb`, `tests/verify/pass/recursive.bmb`).
- The declaration of a trait method does not write `decreases` (E0207).

## `assert`

- `assert(e);` is a statement of the executable context and of the ghost context. `e` is a specification expression and can hold a quantifier. It proves `e` at that position (chapter 07, E0810) and uses `e` as a fact from then on. It produces no executable code. It is not used inside a specification context (E0503).
- `assert(e) by (bit_vector);` proves `e` with a separate bit-vector query (ADR-0048 §4). Only integer, `f64` and `bool` terms come in `e`. Quantifiers, `old`, data values, and elements, fields and lengths that refer to a `let` inside the assertion are errors (E0515). Other elements, fields, lengths and values behind references (`s[0]`, `r.x`, `*r`, `s.len()`) are accepted as variables of the sub-proof. A prover name other than `bit_vector` and `assert(..) by { .. }` are errors (E0516, `tests/check/err/bit_vector.bmb`, `tests/verify/pass/bit_vector.bmb`). This sub-proof sees only the variables of `e` and does not see the facts of the path. Its failure is in chapter 07 (E0818).

## `old`

- `old(x)` is the value of the `&mut` parameter `x` at function entry. It comes in `ensures`, loop invariants, `assert` and `proof` blocks.
- If it is in any other specification position (`requires` and so on) or `x` is not a `&mut` parameter, it is an error (E0508, `old_on_shared` in `tests/check/err/specs.bmb`). If it is in the executable context it is an error (E0503).
- A scalar reference is written `*old(x)`, a slice `old(a)[i]` and `old(a).len()`, and a receiver `old(self)` (`bump` in `tests/verify/pass/inline_contracts.bmb`, `tests/verify/pass/trait_contracts.bmb`).

## `proof fn` and `proof` blocks

- `proof fn name(params) requires .., ensures .., decreases .., { .. }` is a lemma (ADR-0062). It writes no return type and no effect attribute (E0256). Its parameters are values and `&T`. The rules for contract clauses are the same as for executable functions (E0501, E0507). It accepts `#[uses(..)]`.
- The ghost context is the body of a `proof fn` and a `proof { .. }` block in executable code (in statement position). What comes there is `proof fn` calls, `assert`, `if` and `match` that split cases, and `let` of specification expressions. Calls of executable functions, assignment, `let mut`, `&mut`, loops and nesting of `proof` blocks are errors (E0257).
- A `proof fn` is called only from the ghost context. Calling it from executable code or from a specification is an error (E0258, `tests/check/err/proofs.bmb`).
- The arithmetic of ghost code is mathematical integers, so there are no overflow or index obligations. Instead, it is an obligation that the arguments of a `proof fn` call are within the parameter types (chapter 07, E0819).
- A call proves the `requires` and gives the `ensures` as a fact from then on. Induction is a recursive call with `decreases` (`tests/verify/pass/proofs.bmb`, `tests/run/max_list.bmb`). Ghost code does not remain in the executable.

## Type invariants

- `#[invariant(p)]` before a struct or enum. `p` is a `spec fn` that takes the type as `x: &T` and returns `bool`, and its body is not `true` (E0254). There is one per type (E0255, `tests/check/err/invariants.bmb`). ADR-0060.
- An executable function assumes the invariant for its parameters, the results of calls, `&mut` arguments after a call, and locals at a loop head. At the positions where a value leaves the hands of the function (call arguments, return, `&mut` parameters at exit, loop heads) the invariant is an obligation (chapter 07, E0823). Inside the function it may be broken for a while.
- The predicate `p` is opaque like any other `spec fn`. A proof that needs the definition writes `#[uses(p)]` (`tests/verify/pass/invariants.bmb`, `tests/run/ring.bmb`).
- In a `spec fn`, in values inside a specification, in a `proof fn` and in a `proof` block there is neither assumption nor obligation. For a lemma to use the invariant, write it as `requires p(x)`.

## Trait contracts and impl refinement

- The declaration of a trait method has `requires` and `ensures`. A trait can declare a bodiless `spec fn m(self: &Self) -> T;` and each impl writes its definition (ADR-0046 §4, `tests/verify/pass/trait_contracts.bmb`).
- A call through a bound (`Self` is a type parameter) uses the contract of the trait. A call on a concrete type uses the contract of that impl function.
- A function of an impl writes its own contract, and that contract must refine the contract of the trait: the preconditions of the trait imply each precondition of the impl, and the preconditions of the trait together with the postconditions of the impl imply each postcondition of the trait. The failure of this obligation is in chapter 07 (E0817, `tests/verify/fail/trait_contracts.bmb`).

## Contracts of function values

- For a parameter `f` with the bound `F: Fn(A) -> R`, a specification writes `f.requires(a)` (the precondition) and `f.ensures(a, r)` (the postcondition). A call `f(a)` proves `f.requires(a)` and assumes `f.ensures(a, r)` (ADR-0053, `tests/verify/pass/closure_context.bmb`, `tests/verify/fail/fn_values.bmb`).
- When a function item is passed as a value, the `requires` and `ensures` of that function are the contract of the value. A closure writes its contract in place: `|x: T| -> (r: R) requires P, ensures Q, { body }` (`tests/run/fn_values.bmb`).
- The `spec fn`s called by the clauses a closure writes are in the default context of the closure, and the closure inherits the `#[uses(..)]` of the function that wrote it (ADR-0114).
- The rule on the effects of a passed function (E0608) is in chapter 08.

Contract lints that begin with `W` (vacuous precondition, weak specification and so on) are not verdicts of the language. The list is in `docs/diagnostics.md` and the tool in `compiler/README.md`.
