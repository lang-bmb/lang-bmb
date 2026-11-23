# ADR-0060: Type invariants: data invariants proved at boundaries instead of visibility

- Status: Accepted (approved by the maintainer 2026-10-28 "ADR-0059 to 0062 accepted")
- Date: 2026-10-28
- Related: `dev-docs/reviews/direction-2026-10-28.md` §3·§6.2 (H-ENCAP), ADR-0015 (second-class references), ADR-0017 §4 (opaque `spec fn`), ADR-0026·0027 (data types), ADR-0032·0043 (incremental keys), ADR-0046 (generics), ADR-0047 (owned types), ADR-0055 (reference `match`), DESIGN.md principle 7, §4.1

## Confirmed at acceptance (2026-10-28)
- Implementation phase: ROADMAP M2-c (identity and collaboration), backlog T1-11. Proposed tasks 1~6 of the Consequences section are the subtasks.
- Diagnostic numbers: E0254·E0255·E0823 are unused in `compiler/src/diag.rs` (checked 2026-10-28). They are used as is at implementation.
- Open question 1 (the collaboration share of visibility) goes to a new ADR when measurement shows the cost. 2 (cost of deep validity) is measured at implementation with V4·V7, and if expensive it is compared with the shallow validity option and written under "Confirmed at implementation". 3 (invariants left broken across a loop) starts with the current option (validity at the loop head) and is reopened when a real program gets blocked.

## Confirmed at implementation (2026-10-28, backlog T1-11)
- Scope: proposed tasks 1~5 of the Consequences section are implemented. 6 (BRB extension, H-ENCAP measurement) was raised to the measurement track as `T0-09`. Code paths and tests are in the "type invariants" entry of `compiler/README.md` and the backlog row T1-11.
- §1 Surface: `invariant` is the keyword of the loop clause. The attribute name is read separately (parser `attrs`). The shape of the predicate was fixed as "the parameters of the type in the same order, bounds within those of the type" (so that the instance of a generic predicate is determined directly from the type arguments). A predicate whose body is `true` is also E0254. The E0502 of the "named predicate rule" is a diagnostic of contract clauses, so its meaning was not widened.
- §1 Identity (ADR-0059): the attribute writes the predicate by `id`, like `#[uses]`. The predicate takes `&T`, so the type and the predicate are one recursive component. Changing the predicate body changes the `id` of the type, and changing only the predicate name leaves it (`invariant_edits_reencode_where_values_pass` in `tests/incremental.rs`; the name fuzzer `renames_keep_identities_and_edits_change_them` passes). The transaction `rename` also fixes the attribute.
- §2 Proof functions (ADR-0062): validity is not assumed for the parameters of a `proof fn`, and the arguments of its calls have no obligation. `proof` blocks are the same. Ghost code is a specification context and this ADR puts no assumptions in specification contexts. The symmetry also holds. A ghost argument can be an arbitrary specification value, so an assumption would create an obligation at every call. A lemma that needs the invariant writes `requires p(x)`.
- §2 Encoding details:
  - Recursive types get no quantified definition. The definition of the function `|T.'valid|` enters only as two levels of ground instances of the applications the query contains (sound for the same reason as the ground instances of ADR-0023). With a quantified definition, the matching loop uses up the rlimit on every satisfiable query, so queries that should give a counterexample become expensive and uncertain. The unfolding of the inside of an element quantification (`Vec<Tree>`) is not reached by these two levels. No place in the corpus needs it yet.
  - A fact already in the path condition is not emitted as an obligation (the same rule as E0819). The bound variable of an element quantification is named by nesting depth, so that the same value becomes the same text. The first implementation used a global number, so even generic code that only moves values (`fill_short`) got three obligations.
  - A value written with a constructor (`None`, `Some(x)`) is unfolded directly to that variant.
  - Parameter validity creates obligations in generic code too. The result `a[i]` of `get<T>(a: &[T], i)` is a real obligation that comes from the element quantification. In the existing failing corpus the obligation counts grew: `gen::get` 1 → 2, `gen::fill_short` 6 → 7, `gen::max_first` 1 → 2. Verdicts did not change.
  - W0901 (satisfiability) uses the same entry assumptions.
- Existing defect fixed during implementation: the value of the `None`/`Err` returned by `?` was built with the written result type (`f.ret.ty`). So a result that writes a type inside the unit by its short name (`Result<i64, Span>`) became an undeclared constructor symbol (E0812). Now the result type fixed by the checker is used. It had not shown up because no existing corpus had a postcondition reading that value.
- Metric measurement: `S2` and `S4` of `scripts/health.py` counted only `compiler/src/*.rs`. This skipped submodules (`src/verify/invariants.rs`) and could miss the trust boundary (external processes), so it was fixed to `src/**`.
- **Open question 2 (cost of deep validity): not expensive. The shallow validity option was not compared.**
  - V4 7,034,750 → 7,737,006 (+10.0%). Most of it is new corpus: `sorted_set` 433,767, `verify/pass/invariants` 94,531, `ring` 65,713, total 594,011. The `generic_stack` migration is 217,860 → 219,503 (+0.8%). The remaining roughly 107k (1.5% of the baseline) is the cost of parameter validity in existing generic code.
  - V5 0.0153 and V7 0.0 (0 unstable obligations) are unchanged.
  - The expensive axes of deep validity are small too. `Vec` element quantification (`first_width`, whole definition 6,594) and recursive unfolding (`two` 7,252, `push_front` 2,964) are all under 10,000 per definition.
  - The most expensive definition is `insert` of `sorted_set` at 230,496 (largest single obligation 61,494). This cost is the proof of the sortedness invariant itself, and it is the same proof when written as a contract.
- **Open question 3 (invariants left broken across a loop): a real program got blocked. Reopened (`T1-26`; a change of meaning, so acceptance needs approval).**
  - Shifting in place over `self.items` in `insert` of the strictly sorted set (`tests/run/sorted_set`) leaves two equal values between iterations, so validity at the loop head gives E0823. In Rust this is canonical code that would be written with `copy_within`. The reproduction is the same program with the loop changed to `self.items[j] = self.items[j - 1]`.
  - For now it shifts over a copy (`let mut a = self.items`) and assigns once at the end (workaround, `T1-26`).
  - With non-strict sorting (`<=`) the in-place shift keeps the invariant between iterations too.
  - The loops of the ring buffer and `generic_stack` were not blocked.
- `generic_stack` migration: contract clauses 7 → 6 (the loop invariant `fits::<u8>(&st)` is gone). In `some` the piece `s.len <= 8` is gone, leaving `0 < s.len`. In return, the `#[uses]` of `new`, `push` and `pop` each gained one `fits` (because the body re-establishes the invariant). The file is 3,893 → 3,924 bytes (including the changed summary text). The price of fewer repeated contracts is more `#[uses]` entries. The clauses that shrink are on the loop side and the "in range" premises.

## Context
- BMB has no visibility. There is no `pub` and definitions of other units are called by full path (`compiler/README.md` "units"). Any function of any unit can read and write struct fields.
- So the invariant of a data structure is not a property of the type. It is only the contracts of the functions that declare it.
  - Example: `Stack<T>` of `tests/run/generic_stack.bmb` rewrites `len <= 8` in every function, in the `requires`/`ensures` of `new`, `push` and `pop` (`room`, `some`, `pushed`, `popped`).
  - A new function that does not write these contracts can make and return a value with `s.len = 100`. A function that receives that value can assume nothing.
- Rust solves this with visibility (private fields + constructors). The invariant itself is kept by comments and tests.
- The question of the direction discussion (H-ENCAP) is this. In a language accessed only through tools and with a proof gate, is invariant protection done by "who can write" (visibility) or by "is the written result right" (proof)?
- An existing precedent: the range of integer types is BMB's built-in type invariant.
  - The encoder assumes the range of the type every time it makes a fresh symbol (`fresh_val` at `verify.rs:2143` puts `range(t, &c)` into the path condition).
  - It emits an obligation for every operation that can leave the range (E0803).
  - User types have no such mechanism.

## Precedents (JUDGMENT S1 order)

| Source | Approach | Note |
| --- | --- | --- |
| Rust | Private fields + changes only through constructors and methods. Invariants are comments and tests | Protection is by "who". Whether it is right is not proved |
| Verus | A `spec fn` marked `#[verifier::type_invariant]`. It must hold after a constructor expression, after a field assignment, and after a call that takes `&mut` of the value. The user side takes the assumption out with `use_type_invariant(&x)`. Temporarily breaking the invariant is not supported (worked around with a swap) | [Reference](https://verus-lang.github.io/verus/guide/reference-type-invariants.html), [container example](https://verus-lang.github.io/verus/guide/container_bst_type_invariant.html). The original did not open in this environment, so this was confirmed from search summaries (original not checked). Whether visibility is required could not be confirmed |
| Creusot | Type invariants are assumed and checked at function boundaries | [Guide](https://creusot-rs.github.io/creusot/guide/type_invariants.html), original not checked |
| Dafny | A conventional predicate `Valid()` is written in the `requires`/`ensures` of every method, and the ghost field `Repr` sets the frame | A convention, not a language mechanism. The same shape as BMB's current state (option a) |
| Ada 2012/SPARK | `Type_Invariant` attaches only to private types and is checked at the boundary of the package's public operations | [ARM 7.3.2](https://docs.adacore.com/live/wave/arm12/html/arm12/arm12-7-3-2.html). A design tied to visibility |
| Spec#/Boogie | Ownership methodology. Each object has a "valid/open" state (pack/unpack) | Needed because of aliasing through references stored in the heap. Another object can point into my internals and change them |

**Observation.** There are two reasons visibility or pack/unpack is needed for soundness.

1. A design that puts the checks only at the boundary of public operations (Ada).
2. Aliasing through stored references (Boogie).

BMB has neither.

- References exist only in argument position and are not stored or returned (ADR-0015).
- So the only thing that can change a value is the function that owns it, or the one function that received the unique `&mut`.
- Every way that function lets the value out is a boundary visible in the text (call argument, return, the function end for a `&mut` parameter).

## Decision

### 1. Surface
```rust
#[invariant(fits)]
struct Stack<T: Copy> { items: [T; 8], len: usize }

spec fn fits<T: Copy>(s: &Stack<T>) -> bool { s.len <= 8 }
```
- The attribute `#[invariant(p)]` attaches to structs and enums.
  - `p` is a `spec fn` that takes that one type (`x: &T`) and returns `bool`, and follows the named predicate rule (rejected if the body is `true`).
  - For a generic type, `p` takes the same type parameters.
  - There is one attribute per type. Several conditions are joined with `&&` in one predicate (one normal form).
- **The attribute is on the type side.** Verus puts the mark on the predicate side. A type definition goes into every slice that uses the type and into the map L1 type surface, so the invariant is seen together with the type (principle 5, DESIGN §7.2 type capability view).
- **The predicate is opaque like any other `spec fn` (ADR-0017 §4).** The assumption comes in as the atom `fits(x)`. Using its content needs `#[uses(fits)]` (`bmb uses` fills it in).

### 2. Meaning: a new value is assumed valid, and a value being forgotten must be valid
The encoder builds the deep validity `valid_T(x)` of type `T`.

- It combines the invariant of `T` with the deep validity of all fields, payloads, `Box` contents and `Vec` elements (quantified, trigger on `select`).
- For a type with no invariant that part is `true`.
- For recursive types it enters as an opaque definition and unfolding axioms, like a recursive `spec fn` (ADR-0023 ground instances).

| Point | Assumption | Obligation |
| --- | --- | --- |
| Function entry: parameters (value, `&`, `&mut`) | `valid_T(p)` | — |
| Call: arguments (value, `&`, `&mut`) | — | `valid_T(arg)` (E0823) |
| After a call: the result, the new value of `&mut` argument places | `valid_T` | — |
| Every exit of a function (tail, `return`, `?`): the result and the current value of `&mut` parameters | — | `valid_T` (E0823, the same place as the postcondition check `post_check`) |
| Locals havocked at a loop head | `valid_T` at the head | `valid_T` on entry and at the end of every iteration (E0823, implied invariant) |

- This widens what integer ranges do (assume at a fresh symbol) to user predicates. The difference is that the obligation is put not on operations but on **the points where a value is forgotten**.
- Inside a function body the invariant may be broken temporarily.
  - Example: `self.len = self.len + 1` followed by `self.items[..] = x`.
  - Locals and `&mut` parameters are visible only to that function, so this is sound (ADR-0015).
  - The "empty it temporarily with a swap" that Verus uses as a workaround is not needed.
- Writes through `std::mem::replace`/`swap`, field and element assignment, and the `&mut` bindings of `match` (ADR-0055) create no obligation. The result is checked at one of the boundaries above.
- No assumption attaches to `spec fn`s and values inside specifications (same as Verus). Contracts are computed over assumed parameters, so `requires fits(s)` does not have to be written again.
- Inside a generic function the validity of `T` is the uninterpreted predicate `valid_T`. It becomes assumption and obligation at the same points, and in code that only moves values it is closed by the frame.
- There is no run-time cost. If an obligation is not proved there is no binary (principle 4).

### 3. Incremental keys and the map
- The incremental key already contains the definition lines of the types a definition reaches (`incremental.rs:719`~`728`, `type` rows). The invariant attribute goes into that line, and the body hash of the predicate is added to the key (ADR-0032 "everything the encoding reads").
- Changing an invariant changes the meaning of every definition that uses `T`. `impact` widens a type definition change to every definition in which `T` appears.
- The type surface of map L1 carries the invariant name.

### 4. Diagnostics (numbers provisional, fixed in `compiler/src/diag.rs` at acceptance)
| Code | Meaning | Hint |
| --- | --- | --- |
| E0254 | The `p` of `#[invariant(p)]` does not exist, is not a `spec fn`, or does not have the shape `(x: &T) -> bool` | The right signature |
| E0255 | Two or more `#[invariant]` on one type | Combine into one predicate (`&&`) |
| E0823 | The invariant of `T` may not hold at a boundary (argument, return, `&mut` exit, loop) | Field values in the counterexample. Which boundary is in the message |

## Rationale (JUDGMENT §10)
1. **Problem:** a data invariant is not a property of the type. Nothing stops a function that omits the contract from making a broken value, and the functions that keep it repeat the same contracts (`generic_stack`).
2. **Cause and layer:** the language. The canonical solution (an abstract data type with an invariant) cannot be expressed (JUDGMENT §3 second row).
3. **Canonical solution and sources:** Rust uses visibility. Verification languages (Verus, Creusot, Ada/SPARK) use type invariants, some together with visibility.
4. **Comparison of candidates:**

| Option | C1~C4 | Evaluation |
| --- | --- | --- |
| (a) Current: the invariant per function as `requires`/`ensures` (Dafny `Valid()` convention) | Pass | No type-level guarantee (S2 fails). Tokens grow through repeated contracts (S4), and an omission shows up only as a proof failure |
| (b) Visibility (`pub`/private) | Pass. But tension with C4: gives meaning to units and files (principle 7) | Rust standard (S1-a). But it only reduces "who writes"; "is it right" is still tests or (a). The agent has to write accessor workarounds |
| (c) **Type invariants proved at boundaries (recommended)** | Pass | Verification language standard (S1-b). Solves the whole class (S2). The same mechanism as integer ranges, with no second concept of visibility (S3). No repeated contracts (S4) |
| (d) (b)+(c) | Pass. But the C4 tension is the same as (b) | The most common combination in verification languages (Ada, probably Verus). The share visibility has in that combination is "put the checks only at the public boundary" and "aliasing", and BMB has neither (observation above) |

5. **Recommendation and decisive criterion:** (c).
   - On S1, (d) is the closest to the verification language precedents.
   - But what the visibility of (d) does for soundness is already done in BMB by second-class references (ADR-0015). So (c) wins on S3 (economy of concepts) and C4 (principle 7: units carry no meaning).
6. **Impact (information):**
   - The parser, checker, encoder (`fresh_val`, `call`, `post_check`, loop havoc), incremental keys, map L1 and `impact` change.
   - The repeated contracts of the existing corpus (the `room`/`fits` family of `generic_stack`) are not a target of `bmb fix` but of manual migration. The meaning is different (a contract is a promise of the function, an invariant is a promise of the type).
7. **Approvals needed:** it widens the language scope (AUTONOMY §4), so acceptance needs approval.

**What (c) loses (honestly):** it does not replace the collaboration share of visibility (K), namely hiding the representation to keep a stable API. If agents lean directly on internal fields, there are more places to change when the representation changes. Proof makes this cost safe but does not reduce it. The contract firewall and `impact` only show the scope. If this share becomes a problem in measurement, a separate mechanism (a lint for direct field access, etc.) is decided separately. Open question 1.

## Consequences
- **Row to add to the DESIGN §4.1 ledger:**

  | Area | Convenience Rust·Verus allow | BMB | What is gained |
  | --- | --- | --- | --- |
  | Protection | Visibility (`pub`/private) keeps data invariants at the module boundary. Invariants are comments and tests | No visibility. An invariant is a named predicate attached to the type and is proved at every boundary | The invariant stays in the map (L1 type surface), and the gate keeps it whichever definition makes the edit. Repeated contracts removed. Units carry no meaning (principle 7) |

- **H-ENCAP measurement:**
  - **Needs a model (BRB extension):** make data structure modification tasks (sorted set, ring buffer, interval list, min-heap). Each task has a hidden invariant verdict. Compare the rate of "pass & hidden invariant violated" between the BMB arm (type invariants) and the Rust arm (private + tests). Refutation condition: a violation passes the gate in the BMB arm.
  - **Deterministic part:** the number of contract clauses and tokens removed from the corpus by migrating to invariants.
- **Proposed tasks (IDs from the backlog):**
  1. Check and surface: parsing and normal form of `#[invariant(p)]`, E0254·E0255, map L1 display, predicate included in slices.
  2. Encoding: generation of `valid_T` (deep validity, recursion·`Vec`·`Box`, generics opaque), assumption points (the `fresh_val` family), obligation points (call arguments, exits, loop havoc), E0823 and counterexamples.
  3. Incremental keys and `impact`: the attribute in the type line, the predicate body hash. Checked with the `incremental` audit test.
  4. Corpus: `verify/pass`·`fail` (counterexample per boundary)·`check/err`·`run` (ring buffer, sorted set). Migrate `generic_stack` and report the reduced contract count.
  5. Fuzzer and detection power: put random invariants into `soundness_data`. Add gate detection power mutants (`&mut` exit obligation dropped, loop entry obligation dropped, argument obligation dropped).
  6. BRB extension tasks (H-ENCAP). Runs are on the measurement track (T0).

## Open questions
1. If the collaboration share of visibility (hiding the representation) shows up as a cost in measurement, which mechanism to use.
2. Proof cost of deep validity: `Vec<S>` element quantification and unfolding of recursive types. Measured with V4·V7, and if expensive, compared with the option of shallow validity + assumption on extraction.
3. Whether code that leaves an invariant broken across a loop (keeping a broken state between iterations) is really needed. The current option requires validity at the loop head.
