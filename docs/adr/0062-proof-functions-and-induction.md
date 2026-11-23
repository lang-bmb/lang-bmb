# ADR-0062: Proof functions (`proof fn`) and induction: lemmas written by the user

- Status: Accepted (approved by the maintainer 2026-10-28 "ADR-0059 to 0062 accepted")
- Date: 2026-10-28
- Related: ADR-0017 §4 (opaque `spec fn`, `#[uses]`), ADR-0018 §2 (compiler helper facts and `tests/lemmas.rs`), ADR-0023 (ground instance retry), ADR-0032·0043 (incremental keys), ADR-0049 (`return`·control flow), ADR-0058 (tracks, T1·T4), backlog T1-14, DESIGN §3 principles 4·6

## Confirmed at acceptance (2026-10-28)
- Implementation phase: ROADMAP M2-d (representation extension), backlog T1-18. P1~P6 of the Consequences section are the subtasks.
- Diagnostic numbers: E0256~E0258 are unused in `compiler/src/diag.rs` (checked 2026-10-28). They are used as is at implementation.
- Open question 1 (the hidden specification strength check and lemmas): as in §Consequences, the judge uses the `proof fn`s of the workspace as well. 2 (`forall` introduction) starts with lemma calls only. The Verus form `assert forall .. by { }` is opened with a new ADR when demand comes from the corpus or the pilot. 3 (normal-form position of `proof` blocks) is decided at implementation and written under "Confirmed at implementation".
- Correction of the evidence: among the motivations of the draft, the three failures of the BRB small model were due to the call budget, not lemmas (corrected in `bench/pilot/README.md`). The remaining motivations are the cases the judge left undecided (`vc-vv0039` etc.), the node count of binary-trees, and induction in library specifications.

## Confirmed at implementation (2026-10-28, backlog T1-18)
- **Open question 3 (normal-form position of `proof` blocks):** it stays as a statement where it was written. The order is not changed. The fact a block proves holds from that point on, and the block reads the execution state at that point, so the position is meaning. It is `proof {` … `}` indented like any other statement, with no `;` after it (the parser reads and discards a `;`. The fix of E0258 wraps the call statement as `proof { f(..); };` and `bmb fmt` removes it). A `let` in the block is visible only inside the block, so ghost values do not leak into executable code.
- **Argument range of ghost arithmetic (new diagnostic E0819):** per §2 ghost expressions are mathematical integers. Then the argument of `lemma(n - 1)` can be outside `usize`, while the lemma is proved assuming its parameter is a value of that type. Left as is, falsehood is proved (calling `proof fn natural(x: u32) ensures nonneg(x)` as `natural(x - 1)` yields `nonneg(-1)`: `below_zero` in `tests/verify/fail/proofs.bmb`). A call emits as an obligation that the arguments satisfy the facts of the parameter type (E0819, `Enc::type_facts`: range of integer leaves, length range, range of integer array elements). The lemma assumes exactly those facts about its parameters. The payload range of enum values is left out: that fact of `fresh_val` is one level only, so a structural induction that descends into the payload (`size_nonneg(&**t)`) cannot prove it for the payload of the payload. A lemma that needs it writes it in `requires`. The number is an unused E08 number (E0818·E0820·E0823 are taken by other ADRs) (AUTONOMY §4: new diagnostic codes are autonomous).
- **Boundary of the ghost context:** a third mode of the checker (`Mode::Ghost`). `if`, `match` and blocks contain ghost statements, and conditions, scrutinees, arguments, `let` values and `assert` are specification. E0257 is: executable function calls (including inside specification expressions), assignment, `let mut`, mutating methods (`push` etc.) and `std::mem`, `&mut` arguments and `match`, loops (hint: induct with a recursive `proof fn`), nested `proof` blocks. Calling a `proof fn` from a specification is E0258 (it has no value). `proof` became a reserved word (cannot be used as a unit or definition name). The move check treats a `proof` block as code that only reads (it moves nothing).
- **Judge (open question 1 of "Confirmed at acceptance"):** the check function built by the hidden specification strength check (`judge_target` in `bench/m1/harness.py`) calls only the target function. To use the workspace's lemmas it would have to be decided which lemma to call with which arguments, and that is not mechanically determined from the judge text now. The test `the_vv0039_equivalent_contract_is_proved_with_a_lemma` (`compiler/tests/verify.rs`) checks three cases with the judge text as is: the recursive contract is not decided in the judge (reproduces T1-14), it is proved when a `proof` block in the check function calls the agent's lemma, and when the agent proves the quantified contract directly with a lemma the current judge accepts it as is. Changing the judge is a change of the bench verdict (the re-freeze procedure of ADR-0021), so it is left as `T1-25`.
- **Decision on the judge (2026-11-14, approved by the maintainer, backlog `T1-25`):** the judge stays as it is. Open question 1 of "Confirmed at acceptance" (the judge uses the `proof fn`s of the workspace as well) is not done: changing the verdict would require re-freezing m1·m1b·BVB and the `bench/review`·`bench/intent` that contain them, and results before and after could not be compared. When the agent proves the quantified contract directly with a lemma the current judge accepts it, and the draft of that lemma is written by `bmb fix --induction` (ADR-0135). Undecided is counted separately from rejected.
- **Generics and units:** a generic `proof fn` is proved once with its parameters as sorts, like an executable function, and a lemma of another unit is called by full path (`pg` in `tests/verify/pass/proofs.bmb`). The parser does not yet accept a `proof fn` inside `impl` or a trait (no demand).
- **Measurement:** when `lib/utf8.bmb` was moved to the concatenation specification, five local workaround predicates (`covers`·`covered`·`good`·`ends_by`·`tail`) turned into one recursive `parses` and two lemmas, and the proof cost of `valid_up_to` went from rlimit 310,475 → 62,292, and `is_valid` from 8,924 → 15,107. `parse` of `numlist` went 195,540 → 229,686 (the per-case predicate `fits` became `pow10` and the lemma `pow10_grows`). Five gate detection power mutants (`v-lemma-requires-dropped`, `v-lemma-decrease-dropped`, `v-lemma-post-dropped`, `v-lemma-arg-range-dropped`, `k-proof-block-is-exec`) were added, and running only the targeted tests, all five are caught by `test verify` (failing corpus), `v-lemma-requires-dropped`·`v-lemma-post-dropped`·`k-proof-block-is-exec` are also caught by the specification fuzzer (the lemma functions of `test soundness_specs`), and `k-proof-block-is-exec` also by the error corpus (`test corpus`).

## Context

BMB has no lemmas written by the user (the surface rules of `compiler/README.md` have no `proof fn` or `lemma`). The prover knows only three things about a recursive `spec fn`.

- The definition axiom: one quantified defining equation (`verify.rs:4731` `Prog::axiom`).
- Ground instance retry: on `unknown`, the applications that appeared are unfolded up to 3 levels (ADR-0023, `verify.rs:5480` `ground_instances`).
- The contract of a call: the caller proves the callee's `requires` (`verify.rs:4133`, E0801) and assumes its `ensures`.

Facts that need induction do not come out of these three. These are facts such as "a predicate defined by recursion ⇔ a predicate written with a quantifier", "if it holds on a prefix it holds when extended by one", "joining two pieces joins the definition". In this session it showed up in three places.

1. **Pilot T1-14.** In BVB `vc-vv0039-max-of-list` the agent's contract `contains(lst, r)` means the same as the hidden contract `!(forall i. lst[i] != r)`. `contains` is defined by the recursive `contains_prefix`. But the hidden specification strength check ended with "solver could not decide" and was recorded as a failure. The implication needed induction on `contains_prefix`.
2. **T4 libraries.** The natural specification of UTF-8 validation (T4-02), the JSON parser (T4-05) and SHA-256 (T4-01) is "the input is a concatenation of correct pieces". This is written as a recursive `spec fn`, and extending the loop invariant by one needs an induction lemma.
3. **2K-06 `numlist`.** The digit-count invariant required "a predicate unfolded by hand per case" (`dev-docs/health/2K.md` practicality section, `dev-docs/measurements/m1.md` §6).

`tests/lemmas.rs` (ADR-0018 §2) is where the facts the compiler puts in the trusted base are proved with Z3. It is not a place for the lemmas of user programs.

## Precedents (JUDGMENT S1 order)

| Source | Approach | Check |
| --- | --- | --- |
| Rust | Not applicable (no proof) | — |
| Verus | `proof fn`: a ghost function with `requires`·`ensures`·`decreases`. The body calls other `proof fn`s (including recursively) and splits cases with `assert`·`if`·`match`. From executable code it is called inside a `proof { lemma(x); }` block, and it is erased in code generation. Induction is written as a recursive call (no automatic induction) | [induction](https://verus-lang.github.io/verus/guide/induction.html), [decreases](https://verus-lang.github.io/verus/guide/reference-decreases.html). The original did not open in this environment, so confirmed from search summaries (original not checked) |
| Dafny | `lemma` (ghost method). An automatic induction heuristic applies to lemmas with integer and datatype parameters (tuned with `{:induction}`) | Original not checked (did not come up in search) |
| Why3 | `lemma` (a theorem used like an axiom, proved separately), induction with `let lemma` functions | Original not checked |
| F*·Lean | Tactics, proof terms | Out of scope (a surface that is hard for AI to write, far from DESIGN §4 "a more verbose Rust") |

## Decision

### 1. Surface

```rust
#[uses(contains, contains_prefix)]
proof fn contains_has_index(a: &[i64], n: usize, r: i64)
    requires n <= a.len(), contains_prefix(a, n, r),
    ensures has_index_below(a, n, r),
    decreases n,
{
    if n > 0 && a[n - 1] != r {
        contains_has_index(a, n - 1, r);
    }
}

#[effects(pure)]
#[uses(contains, contains_prefix, ge_all)]
fn max_of_list(lst: &[i64]) -> (r: i64)
    requires non_empty(lst),
    ensures contains(lst, r), ge_all(lst, r),
{
    // ...
    proof { contains_has_index(lst, lst.len(), m); }
    m
}
```

- `proof fn` uses the Verus notation as is. It is a surface inside the training distribution and does not conflict with the ledger rule "make the optional required".
- The return type is only `()`, so it is not written. Same as Verus. The normal form makes this one omission an exception.
- No effect attribute is attached (E0256 rejects it). Like a `spec fn` it has no effects. The rule "effects required" for executable functions (ledger "effects") is about executable code.
- `requires` and `ensures` follow the same rules as executable functions (named predicate calls, E0501). A recursive `proof fn` and mutual recursion require `decreases` (the same check as E0507).
- **Where it is called**
  - A `proof { .. }` block inside an executable function. It appears only in statement position and yields no value.
  - The body of another `proof fn` (without a block).
  - A `proof` block inside a loop body is also allowed (a place that helps maintain the invariant).
- **What goes inside a `proof` block:** `proof fn` calls, `assert`, `if`·`match` (case splits), `let` using only specification expressions. Executable function calls, assignment and `&mut` arguments cannot appear (E0257: a ghost context cannot change or read execution state).
- The parameters of a `proof fn` are values and `&T` only. `&mut` is E0257. There is no way to send ghost state out to executable code.

### 2. Meaning and encoding

- **The proof of a `proof fn` itself is the same as for an executable function.** The body is symbolically executed and `ensures` is emitted as an obligation (`post_check`). `assert` and call premises are obligations. There are no overflow or index obligations. The body is specification expressions, and specification arithmetic is mathematical integers (ADR-0017).
- **A call is the contract mechanism as is (`verify.rs:4098` `call`).** It proves `requires` (E0801), proves the `decreases` decrease if inside a recursive component (E0809), and assumes `ensures`. There are no new trusted facts. The conclusion of a lemma is used only when its proof has passed the gate.
- **Induction is a recursive call.** Both structural induction and numeric induction are written as a recursive `proof fn` with `decreases`. Soundness is given by the same termination obligation as executable function recursion.
- **`#[uses]` stays as it is.** A lemma also declares the opaque predicates it unfolds (ADR-0017 §4). The side that calls the lemma uses only the conclusion, so the need to unfold goes down. This is the gain in proof stability (an extension of the ledger row "proof automation", which removes unrelated context).
- **Erasure:** code generation does not emit `proof fn`s or `proof` blocks. The run-time cost is 0. That there is no trace in the IR is checked by a test.

### 3. Map, incremental, cache

- A `proof fn` appears in map L0·L1 as a definition (`def_kind: "proof fn"`). L2 gets `proof_calls` edges (who relies on this lemma). The "proof dependency" view of DESIGN §7.2 comes from these edges.
- The incremental key (ADR-0032·0043) is the current rule that contains the callee contract, as is. If the body of a lemma changes but the contract stays, the calling side is not proved again (contract firewall).
- The hash (ADR-0019, proposed ADR-0059) is the same as for other definitions.

### 4. Diagnostics (numbers provisional, fixed in `compiler/src/diag.rs` at acceptance)

| Code | Meaning | Hint |
| --- | --- | --- |
| E0256 | Effect attribute or return type on a `proof fn` | Remove (`bmb fix`) |
| E0257 | Executable function call, assignment or `&mut` in a ghost context (`proof fn` body, `proof` block) | Use a specification expression, or move it outside the block |
| E0258 | A `proof fn` called from executable code outside a `proof` block | Wrap in `proof { .. }` (`bmb fix`) |

They do not overlap with E0254·E0255 (ADR-0060 draft) or E0815·E0818·E0820·E0823 (accepted and proposed ADRs).

### 5. Automatic induction by a tool (later)

- Lemmas that are mechanically determined are written by a tool (DESIGN principle 6). The "prefix extension" form and the "recursion ⇔ quantifier" form, which follow the definition of one recursive `spec fn`, are such examples.
- When `bmb fix` recognizes such a form in an obligation failure diagnostic (E0811 "solver could not decide"), it proposes a draft `proof fn` and the call site. The agent accepts it.
- This is a tool decision, not a language decision. This ADR fixes only the place that tool will produce into (user `proof fn`s).

## Rationale (JUDGMENT §10)

1. **Problem.** Facts that need induction cannot be expressed.
   - So a specification with the same meaning fails the verdict (T1-14), and the loop invariants of canonical specifications (a formal grammar written with recursive predicates) cannot be proved (T4).
   - A workaround appears in which a person unfolds per-case predicates by hand (2K-06).
2. **Cause and layer.** The language. The canonical solution (establishing inductive facts with lemmas) cannot be expressed (JUDGMENT §3 second row, §7 judgement question 1).
   - It cannot be fixed in the compiler layer (deeper ground instances, better triggers). Induction is outside the decidable range of first-order SMT.
   - Deeper ground instances are still only a finite unfolding (ADR-0023 Consequences section).
3. **Canonical solution and sources.**
   - The lemmas of verification languages: Verus `proof fn`, Dafny `lemma`, Why3 `lemma` (S1 b).
   - For a Rust-shaped surface and ghost erasure, Verus is the direct precedent.
4. **Comparison of candidates.**

   | Candidate | C1 | C2 | C3 | C4 | S1 Canonical | S2 Root cause | S3 Consistency | S4 Goal | S5 Evidence |
   | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
   | (a) Current: predicates unfolded by hand, ground instances | Pass | Pass | Pass | Pass | No (workaround) | Does not solve the class. Induction beyond depth 3 is impossible | — | Specifications get longer (tokens) and canonical specifications are abandoned | T1-14, 2K-06 |
   | **(b) User `proof fn` (Verus shape)** | Pass (no new trusted facts, termination obligation) | Pass | Pass | Pass (surface inherited, the same direction as the ledger "declare the proof context") | Verus·Dafny·Why3 | Every fact that needs induction | Reuses contracts, recursion and `decreases` as they are | Only the conclusion is used, so context shrinks (stability) | Precedents |
   | (c) Automatic induction only (Dafny heuristic, no user lemmas) | Pass | The heuristic must be deterministic | Pass | Pass | Part of Dafny | Only the forms the heuristic fits. If it fails the user has no way to act | Hidden automation (the ledger "proof automation" chose explicitness) | Writing cost 0, blocked on failure | — |
   | (d) (b) + later (c) as a tool | Same as (b) | Pass (the tool is deterministic) | Pass | Pass | (b)+(c) | Same as (b), and the tool fills in the mechanical forms | Same as principle 6 | Better than (b) | — |

5. **Recommendation and decisive criterion.** The language decision is (b), and for the tool §5 leaves the way to (d) open.
   - (a) fails on S1·S2.
   - Putting (c) into the language fails on S3. It brings in hidden automation, against the direction of the ledger row "proof automation" (explicit triggers, opaque definitions, declaring the facts to use). Also, if the heuristic fails the user has no surface to act on.
   - The gain of (c) is obtained through a tool (§5), not the language. The tool writes its result in the surface of (b).
6. **Impact (information).**
   - Lexer, parser, normal form (`proof fn`, `proof` blocks), checker (ghost context rules, E0256~E0258), encoder (proof of ghost functions; calls use the existing `call`), code generation (erasure), map (L1 `def_kind`, L2 `proof_calls`).
   - Fuzzer: put random lemmas into `soundness_specs`.
   - Gate detection power mutants: decrease obligation of a recursive lemma dropped, assignment allowed inside a `proof` block, lemma conclusion assumed without proof.
7. **Approvals needed.** Widening the language scope (AUTONOMY §4).
8. **Workarounds.** None. The hand unfolding of (a) migrates to the canonical form once this ADR is accepted (2K-06 `numlist`).

## Consequences

- **DESIGN §4.1 ledger:** no new row is needed. It widens "declare the facts to use per proof" of the row "proof automation" to lemmas, and buys that row's "what is gained" (proof stability) as is. Next to "definitions are opaque by default", add "inductive facts are stated explicitly with `proof fn`".
- **Hypothesis registry:**
  - The failure class of T1-14 ("a specification with the same meaning fails because the solver is undecided") changes from a judge defect to "could the agent have written a lemma". The judge (hidden specification strength check) also uses the agent's `proof fn`s from the workspace.
  - H-FAMILIAR (BVB): the Verus arm can already use `proof fn`. If the BMB arm lacks it, BMB is at a disadvantage on the same task because of the surface. It is a premise of a fair comparison.
- **What it opens:** the "concatenation of correct pieces" specifications of UTF-8 validation (T4-02) and the JSON parser (T4-05), the block iteration specification of SHA-256 (T4-01), the recursive specification tasks of BVB.
- **Proposed tasks (numbers from the backlog):**

  | Task | Check |
  | --- | --- |
  | P1 Surface and check: `proof fn`, `proof { }`, ghost context rules, E0256~E0258, normal form | Error corpus (a hint on every line), format fuzzer round trip |
  | P2 Encoding: proof of the `proof fn` body, call obligations and assumptions, recursive decrease | Passing corpus (recursion ⇔ quantifier, prefix extension, concatenation), failing corpus (counterexample for a false conclusion, decrease failure) |
  | P3 Erasure and map: removal in code generation, L1 `def_kind`, L2 `proof_calls`, incremental key audit | No-trace-in-IR test, `tests/incremental.rs` audit, determinism |
  | P4 Fuzzer and detection power | `soundness_specs` extension, the three mutants are caught by the gate |
  | P5 Practicality: prove the same-meaning specification of `vc-vv0039` with a lemma, migrate `numlist`, the concatenation specification of UTF-8 validation | 2 or more run programs (HEALTH §5) |
  | P6 Tool (§5): `bmb fix` that proposes an induction lemma draft on E0811 undecided | Determinism test, the proposed lemma is proved |

## Open questions

1. How the specification strength verdict (hidden check) treats a same-meaning specification that needs induction. If the agent submits the lemma with it, the verdict stays as it is. Reporting separately the cases the judge could not decide (`undecided` of `spec_rejections`) is already done.
2. Whether `forall` introduction (showing it for an arbitrary `i`) is the Verus form `assert forall .. by { }`, or lemma calls only.
3. Where a `proof` block goes in the normal form of executable code (statement order rule).
