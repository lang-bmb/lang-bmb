# ADR-0056: Proof encoding of recursion through structs only

- Status: Accepted (2026-10-27, Accepted (autonomous): AUTONOMY §4 "implementation-level decision within the scope of an existing ADR", backlog 2O-03d)
- Date: 2026-10-27
- Related: ADR-0027 §1·§3 (leaf decomposition of structs, enum datatypes), ADR-0047 §3 (encoding of recursive types, "bounds and stability are measured at implementation"), measurement `dev-docs/measurements/z3-nested-datatypes`, backlog 2O-03d

## Context
`struct Node { v: i64, kids: Vec<Node> }` (the canonical Rust rose tree) contains itself through structs only. A struct is a bundle of leaves (ADR-0027 §1), so the leaves grow without end; up to 2O-03c this was blocked with E0421 and an enum had to be inserted. ADR-0047 §3 said recursive types are SMT datatypes, but a struct has one constructor, and Z3 does not accept a datatype whose only constructor contains an array of itself (not well-founded).

Measurement (`z3-nested-datatypes` README section 2O-03d, the queries exactly as the prover emitted them, Z3 4.15.4):
- Datatype with an added placeholder constructor: no answer on a small satisfiable query (20 second limit). Changing constructor order and shape makes the result vary per query, and in one case Z3 terminated abnormally.
- Uninterpreted sort and selector functions: proofs finish at once, but a satisfiable query with a quantified spec fn gives `unknown` (rlimit 5,000,314, the limit).
- The same query with the sort as `Int`: `sat` (rlimit 93,293).

## Decision
1. **Handle.** The value of a struct that contains itself through structs only (via fields, `Vec`, arrays, `Box`, with no enum: `check::struct_cycle`) is an `Int` handle. The sort name is the struct's (`(define-sort |m::Node| () Int)`) and is declared before all datatypes. For each field leaf a selector function (`|m::Node::Node.0|` etc., the same naming rule as enum payloads) is declared after the datatypes.
2. **Making a value is ground facts.** A struct expression and a field write (`n.v = x`, `n.kids[i].v = x`, `n.kids.push(c)`) make a fresh constant and assert the values of its selectors as equations (`Enc::mk_struct`). There is no constructor function and no quantified axiom, so the query stays quantifier-free. The range and length facts of fields are added as ground facts for each selector application that appears in the path condition and goal of an obligation (`Prog::struct_facts`, excluding terms inside quantifiers). A term on the path refers to a value on that path, and an earlier obligation checked that field. Value definitions (the selector equations of a fresh constant) are not scanned: that would assume the field facts of a value made later or on another path, which the obligation checking that field is supposed to prove (a defect found by the ownership fuzzer during implementation: the overflow of `Node { v: 3 * x, .. }` was proved).
3. **Restrictions in specifications.** Making a value of this struct in a specification is E0432 (there is no term to use in the axiom of the definition: speak through the fields). If the recursive type contains a `Vec`, specification `==` is E0431 (an elementwise comparison of a value of undetermined depth cannot be unfolded: this also blocks the defect where the enum `==` of 2O-03c made the encoder not terminate).
4. **Termination.** `decreases n` accepts this struct too. Structural decrease is the selector chain of 2O-03c (element steps and range conditions) as is, and a struct constructor has no discriminant. The reason the chain is a strict part is not the SMT datatype but that program values are finite (values come only from construction and there are no cyclic references, ADR-0047 §1).
5. **Other data types stay as they are.** Non-recursive structs keep leaf decomposition, and enums (including recursive ones) keep datatypes. ADR-0047 open question 1 (unification) stays open: this measurement showed only that recursive datatypes nested in arrays are unstable in Z3 4.15.4.
6. **Code generation does not change.** `Vec` breaks the layout, so a recursive struct is an ordinary struct, and the drop glue functions call each other (2O-02).

## Rationale (alternatives considered)
| Alternative | Judgement |
| --- | --- |
| Datatype + placeholder constructor | Satisfiable queries do not finish in Z3 4.15.4 (a 360 second wait where a counterexample should come out) |
| Uninterpreted sort + constructor function and quantified axioms | Satisfiable queries are `unknown`, no counterexample |
| Uninterpreted sort + ground facts | With a quantified spec fn the model search runs to the resource limit (5M for one failure) |
| **`Int` handle + ground facts (accepted)** | Every measured query answers immediately. There is no arithmetic on handles, so the meaning is the same except for the cardinality of the sort, and the encoding does not rely on cardinality |
| Keep E0421 (make the user insert an enum) | Blocks the canonical Rust form. Friction (`m1.md` §6) |

## Consequences
- Corpus: `tests/verify/pass/rec_structs` (mutually recursive structs, generic `Rose<T>`, writes two levels down, recursive spec fn), `tests/verify/fail/rec_structs` (7 counterexamples), `tests/check/err/rec_structs` (E0431·E0432·E0401), `tests/run/rose` (valgrind clean).
- The fuzzer `soundness_owned` runs rose tree and `Vec` tree operations (80 batches, 0 mismatches). The path restriction of decision 2 above is the fix of the defect that fuzzer found, and the gate mutation `v-struct-facts-from-definitions` catches a revert.
- A counterexample shows a recursive struct input by one level of fields (`n = Node { v: 0, kids: [Node { .. }] (len 1) }`).
- Equality of two recursive values cannot be stated in a specification (E0431). If needed, write it as a recursive spec fn.
